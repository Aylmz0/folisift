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
    pub status: CoverageStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<String>,
    #[serde(default = "default_confidence")]
    pub confidence: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoverageManifest {
    pub schema_version: u32,
    pub rows: Vec<ManifestRow>,
}

impl CoverageManifest {
    pub fn build(inventory: &Inventory, markdown: &str) -> Self {
        let headings = heading_index(markdown);
        let covers = covers_index(markdown);
        let mut rows = Vec::new();
        for source in &inventory.sources {
            let source_id = source.filename.trim_end_matches(".pdf");
            for (number, title, page) in candidates(source) {
                let heading = if number.is_empty() {
                    title.clone()
                } else {
                    format!("{number} {title}")
                };
                let normalized = normalize(&heading);
                let key = format!("{source_id}:{number}");
                let guide_heading = headings
                    .iter()
                    .find(|candidate| {
                        normalize(candidate) == normalize(&title)
                            || normalize(candidate) == normalized
                            || normalize(candidate).starts_with(&format!("{} ", normalize(&number)))
                    })
                    .cloned();
                let covered = covers.contains(&key) || guide_heading.is_some();
                rows.push(ManifestRow {
                    source: source_id.to_string(),
                    heading,
                    normalized_heading: normalized,
                    page,
                    guide_heading,
                    status: if covered {
                        CoverageStatus::Covered
                    } else {
                        CoverageStatus::Missing
                    },
                    evidence: None,
                    confidence: 1.0,
                });
            }
        }
        Self {
            schema_version: MANIFEST_SCHEMA_VERSION,
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
        let mut missing = 0;
        let mut missing_from_gate = Vec::new();
        let mut empty_sections = Vec::new();
        for row in &self.rows {
            total += 1;
            let present = covers.contains(&format!("{}:{}", row.source, row_number(&row.heading)))
                || headings.iter().any(|heading| {
                    row.guide_heading
                        .as_deref()
                        .is_some_and(|guide| normalize(guide) == normalize(heading))
                });
            let covered = match row.status {
                CoverageStatus::Covered => present,
                CoverageStatus::Merged | CoverageStatus::Excluded => true,
                CoverageStatus::Missing => false,
            };
            if covered {
                claimed += 1;
            } else {
                missing += 1;
                missing_from_gate.push(row.heading.clone());
            }
            let number = row_number(&row.heading);
            let chars = body_chars.get(&number).copied().unwrap_or(0);
            let empty = covered && chars < crate::coverage::MIN_SECTION_CHARS;
            if empty {
                empty_sections.push(row.heading.clone());
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
            canonical_total: total,
            claimed,
            gate_total: claimed,
            missing,
            missing_from_gate,
            empty_sections,
            weak_sections: Vec::new(),
            unknown_claims: Vec::new(),
            duplicate_claims: Vec::new(),
            duplicate_sections: Vec::new(),
            audit_missing: false,
            sources: source_rows,
        }
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

fn candidates(source: &SourceInventory) -> Vec<(String, String, usize)> {
    let mut rows: BTreeMap<String, (String, String, usize)> = BTreeMap::new();
    for section in &source.sections {
        let page = source
            .headings
            .iter()
            .find(|heading| heading.heading.starts_with(&section.number))
            .map_or(0, |heading| heading.page);
        rows.entry(section.number.clone()).or_insert((
            section.number.clone(),
            section.title.clone(),
            page,
        ));
    }
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
        let title = heading[number.len()..].trim().to_string();
        rows.entry(number.to_string()).or_insert((
            number.to_string(),
            title,
            heading_page(source, heading),
        ));
    }
    rows.into_values().collect()
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
    value
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn heading_index(markdown: &str) -> Vec<String> {
    markdown
        .lines()
        .filter_map(|line| {
            let rest = line.strip_prefix("## ")?;
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
    fn manifest_round_trips_json() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("manifest.json");
        let manifest = CoverageManifest::build(&inventory(), "");
        manifest.write_json(&path).expect("write");
        assert_eq!(CoverageManifest::read_json(&path).expect("read"), manifest);
    }
}
