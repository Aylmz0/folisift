//! Reviewed coverage manifest.
//!
//! The manifest is the authoritative list of source headings a guide claims to
//! cover. The legacy coverage gate remains a fallback for guides without one.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::coverage::{CoverageReport, SectionResult, SourceCoverage};
use crate::error::{Error, Result, io};
use crate::inventory::{Inventory, SourceInventory};

pub const MANIFEST_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoverageStatus {
    Covered,
    Missing,
    Merged,
    Excluded,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ManifestRow {
    pub source: String,
    pub heading: String,
    pub normalized_heading: String,
    /// 1-based source page; 0 means the source did not expose a page mapping.
    pub page: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guide_heading: Option<String>,
    /// Exact `##`/`###` guide headings that cover this source row.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub guide_sections: Vec<String>,
    pub status: CoverageStatus,
    /// How the row entered the manifest: heading, toc, inline, review, manual.
    #[serde(default = "default_origin")]
    pub origin: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<String>,
    #[serde(default = "default_confidence")]
    pub confidence: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoverageManifest {
    pub schema_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reviewed_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reviewed_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical_total: Option<usize>,
    pub rows: Vec<ManifestRow>,
}

impl CoverageManifest {
    pub fn build(inventory: &Inventory, markdown: &str) -> Self {
        let headings = heading_index(markdown);
        let covers = covers_index(markdown);
        let mut rows = Vec::new();
        for source in &inventory.sources {
            let source_id = source.filename.trim_end_matches(".pdf");
            for (number, title, page, origin) in candidates(source) {
                let heading = if number.is_empty() {
                    title.clone()
                } else {
                    format!("{number} {title}")
                };
                let normalized = normalize(&heading);
                let key = format!("{source_id}:{number}");
                let guide_heading = guide_heading_for(&headings, &number, &title);
                let covered = covers.contains(&key) || guide_heading.is_some();
                rows.push(ManifestRow {
                    source: source_id.to_string(),
                    heading,
                    normalized_heading: normalized,
                    page,
                    guide_heading: guide_heading.clone(),
                    guide_sections: guide_heading.into_iter().collect(),
                    status: if covered {
                        CoverageStatus::Covered
                    } else {
                        CoverageStatus::Missing
                    },
                    origin,
                    evidence: None,
                    confidence: 1.0,
                });
            }
        }
        let canonical_total = rows.len();
        Self {
            schema_version: MANIFEST_SCHEMA_VERSION,
            reviewed_at: None,
            reviewed_by: None,
            canonical_total: Some(canonical_total),
            rows,
        }
    }

    pub fn report(&self, inventory: &Inventory, markdown: &str) -> CoverageReport {
        let headings = heading_index(markdown);
        let covers = covers_index(markdown);
        let body_chars = crate::coverage::guide_body_chars(markdown);
        let mut by_source: BTreeMap<String, Vec<SectionResult>> = BTreeMap::new();
        let mut total = 0;
        let mut claimed = 0;
        let mut excluded = 0;
        let mut merged = 0;
        let mut missing = 0;
        let mut missing_from_gate = Vec::new();
        let mut empty_sections = Vec::new();
        let mut weak_sections = Vec::new();
        for row in &self.rows {
            total += 1;
            let number = row_number(&row.heading);
            let parent = number.split('.').next().unwrap_or(&number).to_string();
            let present = covers.contains(&format!("{}:{number}", row.source))
                || row.guide_sections.iter().any(|guide| {
                    headings
                        .iter()
                        .any(|heading| normalize(guide) == normalize(heading))
                })
                || row.guide_heading.as_deref().is_some_and(|guide| {
                    headings
                        .iter()
                        .any(|heading| normalize(guide) == normalize(heading))
                });
            let mapped_body_chars = row
                .guide_sections
                .iter()
                .chain(row.guide_heading.iter())
                .filter_map(|guide| body_chars.get(&normalize(guide)))
                .max()
                .copied()
                .or_else(|| body_chars.get(&number).copied())
                .or_else(|| body_chars.get(&parent).copied())
                .unwrap_or(0);
            let covered = match row.status {
                CoverageStatus::Covered => present,
                CoverageStatus::Merged | CoverageStatus::Excluded => true,
                CoverageStatus::Missing => false,
            };
            let counted = match row.status {
                CoverageStatus::Covered => present,
                CoverageStatus::Merged => true,
                CoverageStatus::Excluded | CoverageStatus::Missing => false,
            };
            if counted {
                claimed += 1;
            }
            if matches!(row.status, CoverageStatus::Excluded) {
                excluded += 1;
            }
            if matches!(row.status, CoverageStatus::Merged) {
                merged += 1;
            }
            if !covered {
                missing += 1;
                missing_from_gate.push(row.heading.clone());
            }
            let chars = mapped_body_chars;
            // Empty means no substantive body at all. A short body is weak,
            // not empty; the old 400-character floor conflated the two.
            let empty = covered && !matches!(row.status, CoverageStatus::Excluded) && chars == 0;
            if empty {
                empty_sections.push(row.heading.clone());
            } else if covered
                && !matches!(row.status, CoverageStatus::Excluded)
                && chars < crate::coverage::MIN_SECTION_CHARS
            {
                weak_sections.push(row.heading.clone());
            }
            by_source
                .entry(row.source.clone())
                .or_default()
                .push(SectionResult {
                    number: number.clone(),
                    title: display_title(&row.heading, &number),
                    page: row.page,
                    score: if covered { 1.0 } else { 0.0 },
                    matched: row.guide_heading.clone().unwrap_or_default(),
                    body_chars: chars,
                    body_scope: row
                        .guide_heading
                        .clone()
                        .or_else(|| row.guide_sections.first().cloned())
                        .unwrap_or_else(|| parent.clone()),
                    empty,
                    ok: covered && !empty,
                });
        }
        let source_rows = inventory
            .sources
            .iter()
            .map(|source| {
                let id = source.filename.trim_end_matches(".pdf");
                SourceCoverage {
                    filename: source.filename.clone(),
                    pages: source.pages,
                    low_text_pages: source.low_text_pages.len(),
                    sections: by_source.remove(id).unwrap_or_default(),
                }
            })
            .collect::<Vec<_>>();
        CoverageReport {
            passed: missing == 0 && empty_sections.is_empty(),
            threshold: 1.0,
            total,
            // For a reviewed manifest the manifest rows are the canonical set.
            // Legacy canonical sections are reported by check_coverage instead.
            canonical_total: total,
            claimed,
            excluded,
            merged,
            gate_total: claimed,
            missing,
            missing_from_gate,
            empty_sections,
            weak_sections,
            unknown_claims: Vec::new(),
            duplicate_claims: Vec::new(),
            duplicate_sections: Vec::new(),
            audit_missing: false,
            weak_section_chars: crate::coverage::WEAK_SECTION_CHARS,
            sources: source_rows,
        }
    }

    /// A review file is eligible when no canonical row is still missing.
    /// Review stamps are optional for legacy files and advisory for new ones.
    pub fn review_eligible(&self) -> bool {
        self.rows
            .iter()
            .all(|row| !matches!(row.status, CoverageStatus::Missing))
    }

    pub fn write_json(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| io(parent, error))?;
        }
        let json = serde_json::to_string_pretty(self).map_err(|error| Error::Parse {
            path: path.to_path_buf(),
            message: error.to_string(),
        })?;
        std::fs::write(path, format!("{json}\n")).map_err(|error| io(path, error))
    }

    pub fn read_json(path: &Path) -> Result<Self> {
        let bytes = std::fs::read(path).map_err(|error| io(path, error))?;
        serde_json::from_slice(&bytes).map_err(|error| Error::Parse {
            path: path.to_path_buf(),
            message: error.to_string(),
        })
    }
}

fn default_confidence() -> f32 {
    1.0
}

fn guide_heading_for(headings: &[String], number: &str, title: &str) -> Option<String> {
    let normalized = normalize(title);
    let normalized_number = normalize(number);
    if let Some(exact) = headings.iter().find(|candidate| {
        normalize(candidate) == normalized
            || normalize(candidate) == normalized_number
            || normalize(candidate).starts_with(&format!("{normalized_number} "))
    }) {
        return Some(exact.clone());
    }
    // Generated manifests are skeletons: a subheading is covered when the
    // guide has its parent section. The reviewed manifest still wins for
    // precise mapping and evidence.
    let mut parts = number.split('.').collect::<Vec<_>>();
    while parts.len() > 1 {
        parts.pop();
        let parent = parts.join(".");
        let normalized_parent = normalize(&parent);
        if let Some(found) = headings.iter().find(|candidate| {
            normalize(candidate) == normalized_parent
                || normalize(candidate).starts_with(&format!("{normalized_parent} "))
        }) {
            return Some(found.clone());
        }
    }
    None
}

fn candidates(source: &SourceInventory) -> Vec<(String, String, usize, String)> {
    let mut rows: BTreeMap<String, (String, String, usize, String)> = BTreeMap::new();
    // Real heading blocks are the cleanest source: they never contain body
    // prose. The TOC/section list is a fallback for rows the layout missed.
    for heading in &source.headings {
        let heading = heading.heading.trim();
        let Some(number) = heading.split_whitespace().next() else {
            continue;
        };
        let number = number.trim_end_matches('.');
        if !number
            .chars()
            .all(|character| character.is_ascii_digit() || character == '.')
        {
            continue;
        }
        let title = clean_heading_title(heading[number.len()..].trim());
        if !title_like(&title) {
            continue;
        }
        rows.entry(number.to_string()).or_insert((
            number.to_string(),
            title,
            heading_page(source, heading),
            "heading".to_string(),
        ));
    }
    for section in &source.sections {
        if rows.contains_key(&section.number) || !title_like(&section.title) {
            continue;
        }
        let page = source
            .headings
            .iter()
            .find(|heading| heading.heading.starts_with(&section.number))
            .map_or(0, |heading| heading.page);
        rows.entry(section.number.clone()).or_insert((
            section.number.clone(),
            section.title.clone(),
            page,
            "toc".to_string(),
        ));
    }
    rows.into_values().collect()
}

fn clean_heading_title(value: &str) -> String {
    const CONNECTORS: &[&str] = &["ve", "veya", "ile", "için", "ve/veya"];
    let words = value.split_whitespace().collect::<Vec<_>>();
    if words.len() > 8 {
        for index in 3..words.len().saturating_sub(1) {
            let current = words[index];
            let next = words[index + 1];
            if CONNECTORS.contains(&current.to_lowercase().as_str()) {
                continue;
            }
            let current_upper = current.chars().next().is_some_and(char::is_uppercase);
            let next_lower = next.chars().next().is_some_and(char::is_lowercase);
            if (current_upper && next_lower)
                || current.chars().next().is_some_and(char::is_lowercase)
            {
                return words[..index].join(" ");
            }
        }
    }
    value.to_string()
}

fn title_like(value: &str) -> bool {
    let value = value.trim();
    let words = value.split_whitespace().count();
    !value.is_empty()
        && value.chars().count() <= 160
        && words <= 24
        && !value.ends_with("...")
        && !value.ends_with(',')
        && !value.ends_with(';')
        && !(value.contains(',') && words > 8)
        && !matches!(
            value
                .split_whitespace()
                .next()
                .unwrap_or_default()
                .to_lowercase()
                .as_str(),
            "ve" | "veya" | "ile" | "için" | "daha" | "gibi"
        )
}

fn default_origin() -> String {
    "review".to_string()
}

fn heading_page(source: &SourceInventory, heading: &str) -> usize {
    source
        .headings
        .iter()
        .find(|candidate| candidate.heading.trim() == heading)
        .map_or(0, |candidate| candidate.page)
}

fn display_title(heading: &str, number: &str) -> String {
    heading
        .strip_prefix(number)
        .map(str::trim)
        .filter(|title| !title.is_empty())
        .unwrap_or(heading)
        .to_string()
}

fn row_number(heading: &str) -> String {
    heading
        .split_whitespace()
        .next()
        .unwrap_or(heading)
        .trim_end_matches('.')
        .to_string()
}

fn normalize(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for character in value.chars() {
        let character = match character {
            'ı' | 'İ' | 'I' => 'i',
            other => other.to_lowercase().next().unwrap_or(other),
        };
        if character.is_alphanumeric() {
            out.push(character);
        } else {
            out.push(' ');
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn heading_index(markdown: &str) -> Vec<String> {
    markdown
        .lines()
        .filter_map(|line| {
            let rest = line
                .strip_prefix("## ")
                .or_else(|| line.strip_prefix("### "))?;
            Some(rest.trim().to_string())
        })
        .collect()
}

fn covers_index(markdown: &str) -> BTreeSet<String> {
    static COVERS_RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
        Regex::new(r"<!--\s*covers:\s*([A-Za-z0-9_.-]+:[0-9.]+)\s*-->").expect("valid covers regex")
    });
    COVERS_RE
        .captures_iter(markdown)
        .map(|capture| capture[1].to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{Inventory, SourceInventory, Totals};

    fn inventory() -> Inventory {
        Inventory {
            generated_at: "now".to_string(),
            low_text_threshold: 120,
            sources: vec![SourceInventory {
                path: "PD2.pdf".to_string(),
                filename: "PD2.pdf".to_string(),
                title: "PD2".to_string(),
                pages: 10,
                chars: 0,
                words: 0,
                low_text_threshold: 120,
                source_digest: None,
                low_text_pages: Vec::new(),
                headings: vec![],
                layout: None,
                sections: vec![crate::inventory::CanonicalSection {
                    number: "2.1".to_string(),
                    title: "Giriş".to_string(),
                    page: 1,
                    origin: "toc".to_string(),
                }],
                page_map: vec![],
            }],
            totals: Totals {
                pages: 10,
                words: 0,
                low_text_pages: 0,
            },
        }
    }

    #[test]
    fn review_rows_fall_back_to_the_parent_chapter_body() {
        let inventory = inventory();
        let body = format!(
            "## 1. Parent chapter\n\n{}\n\n## 2. Other\n\n{}\n",
            "anlamlı gövde metni ".repeat(40),
            "başka gövde ".repeat(40)
        );
        let mut manifest = CoverageManifest::build(&inventory, &body);
        manifest.rows[0].guide_heading = Some("1. Parent chapter".to_string());
        manifest.rows[0].guide_sections = vec!["1. Parent chapter".to_string()];
        manifest.rows[0].status = CoverageStatus::Covered;
        let report = manifest.report(&inventory, &body);
        assert!(report.passed, "{report:#?}");
        assert!(report.empty_sections.is_empty(), "{report:#?}");
        assert!(report.sources[0].sections[0].body_chars > 0);
        assert!(manifest.review_eligible());
        manifest.rows[0].status = CoverageStatus::Missing;
        assert!(!manifest.review_eligible());
    }

    #[test]
    fn guide_body_chars_indexes_h2_and_h3() {
        let chars = crate::coverage::guide_body_chars(
            "## 1. Chapter\n\nchapter body\n\n### 1.1 Sub\n\nsub body\n",
        );
        assert!(chars.get("1").is_some_and(|value| *value > 0));
        assert!(chars.get("1.1").is_some_and(|value| *value > 0));
        assert!(chars.contains_key("1 chapter"));
    }

    #[test]
    fn manifest_matches_a_heading_and_fails_a_missing_row() {
        let inventory = inventory();
        let body = format!("## 2.1 Giriş\n\n{}\n", "anlamlı gövde metni ".repeat(40));
        let manifest = CoverageManifest::build(&inventory, &body);
        assert_eq!(manifest.rows.len(), 1);
        assert_eq!(manifest.rows[0].status, CoverageStatus::Covered);
        let report = manifest.report(&inventory, &body);
        assert!(report.passed, "{report:#?}");
        assert!(report.sources[0].sections[0].body_chars >= crate::coverage::MIN_SECTION_CHARS);
        let report = manifest.report(&inventory, "");
        assert!(!report.passed);
        assert_eq!(report.missing, 1);
    }

    #[test]
    fn manifest_keeps_scan_headings_beyond_canonical_sections() {
        let mut inventory = inventory();
        inventory.sources[0].headings = (1..=141)
            .map(|index| crate::inventory::Heading {
                heading: format!("{}.{index} Konu {index}", index % 7 + 1),
                page: index,
                level: Some(1),
                block_id: None,
            })
            .collect();
        let manifest = CoverageManifest::build(&inventory, "");
        assert_eq!(manifest.rows.len(), 141);
    }

    #[test]
    fn excluded_rows_are_counted_and_do_not_fail_the_report() {
        let inventory = inventory();
        let body = format!("## 2.1 Giriş\n\n{}\n", "anlamlı gövde metni ".repeat(40));
        let mut manifest = CoverageManifest::build(&inventory, &body);
        manifest.rows[0].status = CoverageStatus::Excluded;
        manifest.rows[0].guide_heading = None;
        manifest.rows[0].guide_sections.clear();
        let report = manifest.report(&inventory, &body);
        assert!(report.passed, "{report:#?}");
        assert_eq!(report.total, 1);
        assert_eq!(report.claimed, 0);
        assert_eq!(report.excluded, 1);
        assert_eq!(report.missing, 0);
        assert!(manifest.review_eligible());
        manifest.rows[0].status = CoverageStatus::Merged;
        let report = manifest.report(&inventory, &body);
        assert!(report.passed, "{report:#?}");
        assert_eq!(report.claimed, 1);
        assert_eq!(report.merged, 1);
    }

    #[test]
    fn manifest_round_trips_json() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("manifest.json");
        let manifest = CoverageManifest::build(&inventory(), "");
        manifest.write_json(&path).expect("write");
        assert_eq!(CoverageManifest::read_json(&path).expect("read"), manifest);
    }
}
