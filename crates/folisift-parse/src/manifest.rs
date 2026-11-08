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
use crate::heading::{
    clean_heading_title, is_section_number, parse_numbered_heading, recover_inline_headings,
    scan_table_headings, title_like,
};
use crate::inventory::{Inventory, SourceInventory};
use crate::layout::{LayoutBlockKind, SourceLayout};
use crate::source_cache::PageText;

pub const MANIFEST_SCHEMA_VERSION: u32 = 1;

/// Where a generated coverage row came from. Lower rank wins.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadingEvidence {
    /// Explicit heading block, recovered inline heading or table-cell heading.
    Heading,
    /// Numbered heading evidence found in generic layout text.
    Layout,
    /// Source topic list / TOC fallback.
    Toc,
    /// Raw page text, used only when the source has no layout artifact.
    Text,
}

impl HeadingEvidence {
    fn rank(self) -> u8 {
        match self {
            Self::Heading => 0,
            Self::Layout => 1,
            Self::Toc => 2,
            Self::Text => 3,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Heading => "heading",
            Self::Layout => "layout",
            Self::Toc => "toc",
            Self::Text => "text",
        }
    }
}

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
    /// Required when the row is intentionally out of scope.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
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
        Self::build_with_sources(inventory, markdown, &BTreeMap::new(), &BTreeMap::new())
    }

    /// Build a generated manifest with cached source page text. The text
    /// supplies numbered headings the layout classifier missed; it never
    /// overrides a cleaner layout/TOC row.
    pub fn build_with_texts(
        inventory: &Inventory,
        markdown: &str,
        source_texts: &BTreeMap<String, Vec<PageText>>,
    ) -> Self {
        Self::build_with_sources(inventory, markdown, source_texts, &BTreeMap::new())
    }

    /// Build with cached text and stored layout blocks. Table cells and
    /// wrapped blocks often contain headings the layout classifier did not
    /// classify as `heading`; they are additive evidence, never a replacement.
    pub fn build_with_sources(
        inventory: &Inventory,
        markdown: &str,
        source_texts: &BTreeMap<String, Vec<PageText>>,
        source_layouts: &BTreeMap<String, SourceLayout>,
    ) -> Self {
        let headings = heading_index(markdown);
        let covers = covers_index(markdown);
        let mut rows = Vec::new();
        for source in &inventory.sources {
            let source_id = source.filename.trim_end_matches(".pdf");
            for (number, title, page, origin) in candidates(
                source,
                source_texts.get(source_id).map(Vec::as_slice),
                source_layouts.get(source_id),
            ) {
                let heading = if number.is_empty() {
                    title.clone()
                } else {
                    format!("{number} {title}")
                };
                let normalized = normalize(&heading);
                let key = if number.is_empty() {
                    format!("{source_id}:{}", normalize(&title))
                } else {
                    format!("{source_id}:{number}")
                };
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
                    reason: None,
                    origin: origin.as_str().to_string(),
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
            let number = row_number(&row.heading).unwrap_or_default();
            let key = if number.is_empty() {
                format!("{}:{}", row.source, row.normalized_heading)
            } else {
                format!("{}:{number}", row.source)
            };
            let parent = number.split('.').next().unwrap_or(&number).to_string();
            let present = covers.contains(&key)
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

    /// Merge a review into a generated manifest without dropping generated
    /// rows. An unreviewed generated row stays `missing` until the author
    /// covers or explicitly excludes it.
    pub fn merge_with_review(&self, review: &Self) -> Self {
        let mut merged = BTreeMap::<String, ManifestRow>::new();
        for row in &self.rows {
            let key = row_key(row);
            merged.insert(key, row.clone());
        }
        for row in &review.rows {
            merged.insert(row_key(row), row.clone());
        }
        let mut rows = merged.into_values().collect::<Vec<_>>();
        rows.sort_by(|a, b| {
            a.source
                .cmp(&b.source)
                .then_with(|| row_number(&a.heading).cmp(&row_number(&b.heading)))
                .then_with(|| a.heading.cmp(&b.heading))
        });
        Self {
            schema_version: MANIFEST_SCHEMA_VERSION,
            reviewed_at: review.reviewed_at.clone(),
            reviewed_by: review.reviewed_by.clone(),
            canonical_total: self.canonical_total.or(review.canonical_total),
            rows,
        }
    }

    /// A review file is eligible when no canonical row is still missing.
    /// Review stamps are optional for legacy files and advisory for new ones.
    pub fn review_eligible(&self) -> bool {
        self.review_blockers().is_empty()
    }

    /// Named reasons that block adoption. Empty means eligible.
    pub fn review_blockers(&self) -> Vec<String> {
        let mut blockers = Vec::new();
        for row in &self.rows {
            match row.status {
                CoverageStatus::Missing => blockers.push(format!("missing: {}", row.heading)),
                CoverageStatus::Excluded
                    if !row
                        .reason
                        .as_deref()
                        .is_some_and(|reason| !reason.trim().is_empty()) =>
                {
                    blockers.push(format!("excluded without reason: {}", row.heading));
                }
                _ => {}
            }
        }
        blockers
    }

    /// Seed a review template from a generated manifest. Every row starts
    /// missing and must be curated; generated statuses are never adopted
    /// silently.
    pub fn template_from(generated: &Self) -> Self {
        let mut rows = generated.rows.clone();
        for row in &mut rows {
            row.status = CoverageStatus::Missing;
            row.reason = None;
            row.origin = "review".to_string();
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

    /// Build the combined, review-backed manifest. Review rows are the truth;
    /// generated rows only enrich page and guide mappings where the review is
    /// silent. Generated-only rows stay advisory and are not adopted.
    pub fn adopt_with(&self, generated: &Self) -> Self {
        let generated_by_key = generated
            .rows
            .iter()
            .map(|row| (row_key(row), row))
            .collect::<BTreeMap<_, _>>();
        let mut rows = self.rows.clone();
        for row in &mut rows {
            let Some(source) = generated_by_key.get(&row_key(row)) else {
                continue;
            };
            if row.page == 0 && source.page > 0 {
                row.page = source.page;
            }
            if row.guide_heading.is_none() {
                row.guide_heading = source.guide_heading.clone();
            }
            if row.guide_sections.is_empty() {
                row.guide_sections = source.guide_sections.clone();
            }
            if row.evidence.is_none() {
                row.evidence = source.evidence.clone();
            }
        }
        let canonical_total = rows.len();
        Self {
            schema_version: MANIFEST_SCHEMA_VERSION,
            reviewed_at: self.reviewed_at.clone(),
            reviewed_by: self.reviewed_by.clone(),
            canonical_total: Some(canonical_total),
            rows,
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

fn candidates(
    source: &SourceInventory,
    source_text: Option<&[PageText]>,
    layout: Option<&SourceLayout>,
) -> Vec<(String, String, usize, HeadingEvidence)> {
    let mut rows: BTreeMap<String, (u8, String, String, usize, String)> = BTreeMap::new();

    // Layout evidence is the canonical universe: explicit heading blocks,
    // recovered inline headings, table-cell headings, then generic layout text.
    if let Some(layout) = layout {
        for page in &layout.pages {
            for block in &page.blocks {
                if block.kind != LayoutBlockKind::Heading {
                    continue;
                }
                let Some(parsed) = parse_numbered_heading(block.text_or_empty()) else {
                    continue;
                };
                insert_candidate(
                    &mut rows,
                    &parsed.number,
                    parsed.title,
                    page.page,
                    HeadingEvidence::Heading,
                );
            }
        }
        for heading in recover_inline_headings(&layout.pages) {
            let Some(parsed) = parse_numbered_heading(&heading.heading) else {
                continue;
            };
            insert_candidate(
                &mut rows,
                &parsed.number,
                parsed.title,
                heading.page,
                HeadingEvidence::Heading,
            );
        }
        for heading in scan_table_headings(&layout.pages) {
            let Some(parsed) = parse_numbered_heading(&heading.heading) else {
                continue;
            };
            insert_candidate(
                &mut rows,
                &parsed.number,
                parsed.title,
                heading.page,
                HeadingEvidence::Heading,
            );
        }
        for page in &layout.pages {
            for block in &page.blocks {
                for text in crate::heading::block_text_surfaces(block) {
                    scan_layout_text(&mut rows, text, page.page);
                }
            }
        }
    }

    // Inventory headings are explicit parser evidence when no layout exists and
    // a fallback for anything the layout scan could not see.
    for heading in &source.headings {
        let heading_text = heading.heading.trim();
        let parsed = parse_numbered_heading(heading_text).or_else(|| {
            let number = number_token(heading_text)?;
            let title = clean_heading_title(strip_number(heading_text, &number));
            title_like(&title).then_some(crate::heading::NumberedHeading {
                heading: format!("{number} {title}"),
                number,
                title,
            })
        });
        let Some(parsed) = parsed else {
            continue;
        };
        insert_candidate(
            &mut rows,
            &parsed.number,
            parsed.title,
            heading.page,
            if layout.is_some() {
                HeadingEvidence::Layout
            } else {
                HeadingEvidence::Heading
            },
        );
    }

    // TOC rows fill gaps only; they never override heading or layout evidence.
    for section in &source.sections {
        let Some(number) = number_token(&section.number) else {
            continue;
        };
        let title = clean_heading_title(&section.title);
        if !title_like(&title) {
            continue;
        }
        let page = source
            .headings
            .iter()
            .find(|heading| number_token(&heading.heading).as_deref() == Some(number.as_str()))
            .map_or(section.page, |heading| heading.page);
        insert_fallback(&mut rows, &number, title, page, HeadingEvidence::Toc);
    }

    // Raw text is a last resort: only when the source has no layout artifact.
    if layout.is_none()
        && let Some(pages) = source_text
    {
        scan_source_text(&mut rows, pages);
    }

    rows.into_iter()
        .map(|(number, (_, title, _, page, origin))| {
            let evidence = match origin.as_str() {
                "heading" => HeadingEvidence::Heading,
                "layout" => HeadingEvidence::Layout,
                "toc" => HeadingEvidence::Toc,
                _ => HeadingEvidence::Text,
            };
            (number, title, page, evidence)
        })
        .collect()
}

fn scan_layout_text(
    rows: &mut BTreeMap<String, (u8, String, String, usize, String)>,
    text: &str,
    page: usize,
) {
    static NUMBERED_ANYWHERE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
        Regex::new(r"(?:^|\s)(\d+(?:\.\d+){1,3})\.?\s+(.{3,160})")
            .expect("valid embedded heading regex")
    });
    for line in text.lines() {
        for capture in NUMBERED_ANYWHERE.captures_iter(line) {
            let number = capture[1].trim_end_matches('.').to_string();
            if !is_section_number(&number) {
                continue;
            }
            let title = clean_heading_title(capture[2].trim());
            if !title_like(&title) {
                continue;
            }
            insert_candidate(rows, &number, title, page, HeadingEvidence::Layout);
        }
    }
}

fn scan_source_text(
    rows: &mut BTreeMap<String, (u8, String, String, usize, String)>,
    pages: &[PageText],
) {
    static NUMBERED_LINE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
        Regex::new(r"^\s*(\d+(?:\.\d+){1,3})\.?\s+(.{3,160})\s*$")
            .expect("valid numbered heading regex")
    });
    static NUMBERED_ANYWHERE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
        Regex::new(r"(?:^|\s)(\d+(?:\.\d+){1,3})\.?\s+(.{3,160})")
            .expect("valid embedded heading regex")
    });
    for page in pages {
        for line in page.text.lines() {
            let mut matched = false;
            for capture in NUMBERED_LINE.captures_iter(line) {
                matched = true;
                let number = capture[1].trim_end_matches('.').to_string();
                if !is_section_number(&number) {
                    continue;
                }
                let title = clean_heading_title(capture[2].trim());
                if !title_like(&title) {
                    continue;
                }
                insert_fallback(rows, &number, title, page.page, HeadingEvidence::Text);
            }
            if matched {
                continue;
            }
            for capture in NUMBERED_ANYWHERE.captures_iter(line) {
                let number = capture[1].trim_end_matches('.').to_string();
                if !is_section_number(&number) {
                    continue;
                }
                let title = clean_heading_title(capture[2].trim());
                if !title_like(&title) {
                    continue;
                }
                insert_fallback(rows, &number, title, page.page, HeadingEvidence::Text);
            }
        }
    }
}

fn insert_candidate(
    rows: &mut BTreeMap<String, (u8, String, String, usize, String)>,
    number: &str,
    title: String,
    page: usize,
    evidence: HeadingEvidence,
) {
    let rank = evidence.rank();
    match rows.get_mut(number) {
        Some(existing) if rank < existing.0 => {
            existing.0 = rank;
            existing.1 = title;
            existing.2 = number.to_string();
            existing.3 = page;
            existing.4 = evidence.as_str().to_string();
        }
        Some(existing) => {
            if existing.3 == 0 && page > 0 {
                existing.3 = page;
            }
        }
        None => {
            rows.insert(
                number.to_string(),
                (
                    rank,
                    title,
                    number.to_string(),
                    page,
                    evidence.as_str().to_string(),
                ),
            );
        }
    }
}

fn insert_fallback(
    rows: &mut BTreeMap<String, (u8, String, String, usize, String)>,
    number: &str,
    title: String,
    page: usize,
    evidence: HeadingEvidence,
) {
    if rows.contains_key(number) {
        return;
    }
    rows.insert(
        number.to_string(),
        (
            evidence.rank(),
            title,
            number.to_string(),
            page,
            evidence.as_str().to_string(),
        ),
    );
}

fn strip_number<'a>(heading: &'a str, number: &str) -> &'a str {
    heading
        .strip_prefix(number)
        .unwrap_or(heading)
        .trim_start_matches(['.', ':', '-', '–', '—', ' '])
        .trim()
}

fn number_token(value: &str) -> Option<String> {
    let token = value
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .trim_end_matches(['.', ':']);
    is_section_number(token).then(|| token.to_string())
}

fn default_origin() -> String {
    "review".to_string()
}

fn display_title(heading: &str, number: &str) -> String {
    heading
        .strip_prefix(number)
        .map(str::trim)
        .filter(|title| !title.is_empty())
        .unwrap_or(heading)
        .to_string()
}

pub(crate) fn row_number(heading: &str) -> Option<String> {
    number_token(heading)
}

pub fn row_key(row: &ManifestRow) -> String {
    match row_number(&row.heading) {
        Some(number) => format!("{}:{number}", row.source),
        None => format!("{}:{}", row.source, row.normalized_heading),
    }
}

/// Public row-number helper for MCP planning and hint surfaces.
pub fn manifest_row_number(heading: &str) -> Option<String> {
    row_number(heading)
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

pub(crate) fn covers_index(markdown: &str) -> BTreeSet<String> {
    static COVERS_RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
        Regex::new(r"<!--\s*covers:\s*(.+?)\s*-->").expect("valid covers regex")
    });
    static ENTRY_RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
        Regex::new(r"^\s*([A-Za-z0-9_.-]+)\s*:\s*(.+?)\s*$").expect("valid covers entry")
    });
    let mut out = BTreeSet::new();
    for capture in COVERS_RE.captures_iter(markdown) {
        let mut current_source: Option<String> = None;
        for entry in capture[1].split(',') {
            let entry = entry.trim();
            let (source, spec) = if let Some(entry) = ENTRY_RE.captures(entry) {
                let source = entry[1].trim().to_string();
                current_source = Some(source.clone());
                (source, entry[2].trim().to_string())
            } else if let Some(source) = current_source.as_ref() {
                // `PD2:3.1,3.2` keeps the previous source for the second
                // entry without requiring a repeated `PD2:` prefix.
                (source.clone(), entry.to_string())
            } else {
                continue;
            };
            for number in expand_number_spec(&spec) {
                out.insert(format!("{source}:{number}"));
            }
        }
    }
    out
}

/// Expand `2.1–2.2`, `2.1,2.2` or a single number into canonical numbers.
pub fn number_specs(value: &str) -> Vec<String> {
    let mut out = Vec::new();
    for part in value.split(',') {
        out.extend(expand_number_spec(part));
    }
    out.sort();
    out.dedup();
    out
}

fn expand_number_spec(value: &str) -> Vec<String> {
    let mut out = Vec::new();
    let value = value.trim();
    if let Some((start, end)) = value.split_once(['–', '—', '-']) {
        let Some(start_tuple) = number_tuple(start.trim()) else {
            return out;
        };
        let Some(end_tuple) = number_tuple(end.trim()) else {
            return out;
        };
        if start_tuple.len() == end_tuple.len()
            && start_tuple[..start_tuple.len() - 1] == end_tuple[..end_tuple.len() - 1]
        {
            let prefix = start_tuple[..start_tuple.len() - 1].to_vec();
            let first = *start_tuple.last().unwrap_or(&0);
            let last = *end_tuple.last().unwrap_or(&first);
            for value in first..=last {
                let mut parts = prefix.clone();
                parts.push(value);
                out.push(
                    parts
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join("."),
                );
            }
        } else {
            out.push(start.trim().to_string());
            out.push(end.trim().to_string());
        }
    } else if let Some(number) = number_token(value) {
        out.push(number);
    }
    out
}

fn number_tuple(value: &str) -> Option<Vec<u32>> {
    let mut parts = Vec::new();
    for part in value.trim().trim_end_matches('.').split('.') {
        parts.push(part.parse::<u32>().ok()?);
    }
    (!parts.is_empty()).then_some(parts)
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
        inventory.sources[0].headings = (1..=99)
            .map(|index| crate::inventory::Heading {
                heading: format!("{}.{index} Konu {index}", index % 7 + 1),
                page: index,
                level: Some(1),
                block_id: None,
            })
            .collect();
        let manifest = CoverageManifest::build(&inventory, "");
        assert_eq!(manifest.rows.len(), 99);
    }

    #[test]
    fn rejects_quantities_and_leading_punctuation() {
        let mut inventory = inventory();
        inventory.sources[0].sections.clear();
        inventory.sources[0].headings = [
            "1.0 Liste",
            "10.500 m3",
            "300.000 Adet",
            "1 ) Birim Satış Fiyatı",
            "40 x azaldı",
        ]
        .into_iter()
        .map(|heading| crate::inventory::Heading {
            heading: heading.to_string(),
            page: 1,
            level: None,
            block_id: None,
        })
        .collect();
        let manifest = CoverageManifest::build(&inventory, "");
        assert!(manifest.rows.is_empty(), "{:?}", manifest.rows);
    }

    #[test]
    fn layout_evidence_suppresses_toc_and_text_fallbacks() {
        use crate::layout::{LayoutBlock, LayoutBlockKind, LayoutCell, PageLayout, SourceLayout};

        let mut inventory = inventory();
        inventory.sources[0].headings = vec![crate::inventory::Heading {
            heading: "1 Giriş".to_string(),
            page: 1,
            level: Some(1),
            block_id: Some("PD2:p1:b0".to_string()),
        }];
        inventory.sources[0].sections = vec![
            crate::inventory::CanonicalSection {
                number: "1".to_string(),
                title: "Giriş".to_string(),
                page: 1,
                origin: "toc".to_string(),
            },
            crate::inventory::CanonicalSection {
                number: "1.0".to_string(),
                title: "Liste".to_string(),
                page: 1,
                origin: "toc".to_string(),
            },
            crate::inventory::CanonicalSection {
                number: "9.9".to_string(),
                title: "TOC only".to_string(),
                page: 2,
                origin: "toc".to_string(),
            },
        ];
        let layout = SourceLayout::new(
            "PD2",
            2,
            vec![
                PageLayout {
                    page: 1,
                    page_label: None,
                    width: 100.0,
                    height: 100.0,
                    content_bounds: None,
                    blocks: vec![
                        LayoutBlock {
                            id: "PD2:p1:b0".to_string(),
                            kind: LayoutBlockKind::Heading,
                            bbox: None,
                            text: Some("1 Giriş".to_string()),
                            level: Some(1),
                            bold: true,
                            italic: false,
                            ordered: None,
                            marker: None,
                            lines: Vec::new(),
                            lang: None,
                            header: Vec::new(),
                            rows: Vec::new(),
                            header_rows: None,
                            figure_id: None,
                            figure_format: None,
                        },
                        LayoutBlock {
                            id: "PD2:p1:b1".to_string(),
                            kind: LayoutBlockKind::Paragraph,
                            bbox: None,
                            text: Some(
                                "Bu cümle gövdedir. 4.4.4 Asıl İşveren - Alt İşveren ilişkisi"
                                    .to_string(),
                            ),
                            level: None,
                            bold: false,
                            italic: false,
                            ordered: None,
                            marker: None,
                            lines: Vec::new(),
                            lang: None,
                            header: Vec::new(),
                            rows: Vec::new(),
                            header_rows: None,
                            figure_id: None,
                            figure_format: None,
                        },
                    ],
                },
                PageLayout {
                    page: 2,
                    page_label: None,
                    width: 100.0,
                    height: 100.0,
                    content_bounds: None,
                    blocks: vec![LayoutBlock {
                        id: "PD2:p2:b0".to_string(),
                        kind: LayoutBlockKind::Table,
                        bbox: None,
                        text: None,
                        level: None,
                        bold: false,
                        italic: false,
                        ordered: None,
                        marker: None,
                        lines: Vec::new(),
                        lang: None,
                        header: Vec::new(),
                        rows: vec![vec![LayoutCell {
                            text: "10.2 Ulusal Bayram ve Genel Tatiller".to_string(),
                            bbox: None,
                            colspan: None,
                            rowspan: None,
                        }]],
                        header_rows: None,
                        figure_id: None,
                        figure_format: None,
                    }],
                },
            ],
        );
        let mut texts = BTreeMap::new();
        texts.insert(
            "PD2".to_string(),
            vec![PageText {
                page: 1,
                chars: 200,
                text: "9.8 Only in raw text\nBu bir gövde cümlesidir.".to_string(),
            }],
        );
        let mut layouts = BTreeMap::new();
        layouts.insert("PD2".to_string(), layout);
        let manifest = CoverageManifest::build_with_sources(&inventory, "", &texts, &layouts);
        let numbers = manifest
            .rows
            .iter()
            .map(|row| crate::manifest::row_number(&row.heading).unwrap_or_default())
            .collect::<Vec<_>>();
        assert!(numbers.contains(&"1".to_string()), "{numbers:?}");
        assert!(numbers.contains(&"4.4.4".to_string()), "{numbers:?}");
        assert!(numbers.contains(&"10.2".to_string()), "{numbers:?}");
        assert!(
            numbers.contains(&"9.9".to_string()),
            "TOC fallback: {numbers:?}"
        );
        assert!(!numbers.contains(&"1.0".to_string()), "{numbers:?}");
        assert!(!numbers.contains(&"9.8".to_string()), "{numbers:?}");
        assert!(
            manifest
                .rows
                .iter()
                .all(|row| row.origin == "heading" || row.origin == "toc"),
            "{:?}",
            manifest
                .rows
                .iter()
                .map(|row| (&row.heading, &row.origin))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn adoption_uses_review_rows_and_enriches_from_generated() {
        let inventory = inventory();
        let generated = CoverageManifest::build(&inventory, "## 2.1 Giriş\n\nbody\n");
        let mut review = generated.clone();
        review.rows[0].status = CoverageStatus::Covered;
        review.rows[0].guide_heading = None;
        review.rows[0].guide_sections.clear();
        review.rows[0].page = 0;
        assert!(review.review_eligible());
        let adopted = review.adopt_with(&generated);
        assert_eq!(adopted.rows.len(), review.rows.len());
        assert_eq!(adopted.rows[0].page, 1);
        assert!(adopted.rows[0].guide_heading.is_some());
        assert_eq!(adopted.canonical_total, Some(1));
    }

    #[test]
    fn review_blockers_name_missing_and_unexplained_rows() {
        let inventory = inventory();
        let generated = CoverageManifest::build(&inventory, "");
        assert_eq!(generated.review_blockers().len(), 1);
        let mut review = generated.clone();
        review.rows[0].status = CoverageStatus::Excluded;
        review.rows[0].reason = None;
        assert!(review.review_blockers()[0].contains("excluded without reason"));
    }

    #[test]
    fn excluded_rows_are_counted_and_do_not_fail_the_report() {
        let inventory = inventory();
        let body = format!("## 2.1 Giriş\n\n{}\n", "anlamlı gövde metni ".repeat(40));
        let mut manifest = CoverageManifest::build(&inventory, &body);
        manifest.rows[0].status = CoverageStatus::Excluded;
        manifest.rows[0].reason = Some("intentionally out of scope".to_string());
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
    fn covers_expand_ranges_and_lists() {
        let covers = covers_index(
            "<!-- covers: PD2:2.1–2.2, PD2:3.1,3.2 -->\n<!-- covers: PD3:10.1-10.2 -->",
        );
        for key in [
            "PD2:2.1", "PD2:2.2", "PD2:3.1", "PD2:3.2", "PD3:10.1", "PD3:10.2",
        ] {
            assert!(covers.contains(key), "missing {key}: {covers:?}");
        }
        assert!(!covers.contains("PD2:2.3"));
        assert_eq!(number_specs("2.1–2.2"), vec!["2.1", "2.2"]);
        assert_eq!(number_specs("3.1,3.2"), vec!["3.1", "3.2"]);
    }

    #[test]
    fn body_prose_is_not_a_canonical_heading() {
        let mut inventory = inventory();
        inventory.sources[0].headings = vec![crate::inventory::Heading {
            heading: "2.1 Bu paragraf gövde metnidir ve başlık değildir".to_string(),
            page: 4,
            level: Some(1),
            block_id: None,
        }];
        inventory.sources[0].sections.clear();
        let manifest = CoverageManifest::build(&inventory, "");
        assert!(manifest.rows.is_empty(), "{:?}", manifest.rows);
    }

    #[test]
    fn cached_source_text_supplies_missing_numbered_rows() {
        let mut inventory = inventory();
        inventory.sources[0].headings.clear();
        inventory.sources[0].sections.clear();
        let mut texts = BTreeMap::new();
        texts.insert(
            "PD2".to_string(),
            vec![PageText {
                page: 7,
                chars: 200,
                text: "4.4.4 Alt Düzey Konu\n10.2 Son Konu\nBu bir gövde cümlesidir.".to_string(),
            }],
        );
        let manifest = CoverageManifest::build_with_texts(&inventory, "", &texts);
        let headings = manifest
            .rows
            .iter()
            .map(|row| row.heading.as_str())
            .collect::<Vec<_>>();
        assert!(
            headings.iter().any(|heading| heading.starts_with("4.4.4 ")),
            "{headings:?}"
        );
        assert!(
            headings.iter().any(|heading| heading.starts_with("10.2 ")),
            "{headings:?}"
        );
    }

    #[test]
    fn merge_preserves_unreviewed_generated_rows_as_missing() {
        let inventory = inventory();
        let generated = CoverageManifest::build(&inventory, "");
        let mut review = generated.clone();
        review.rows.clear();
        let merged = generated.merge_with_review(&review);
        assert_eq!(merged.rows.len(), generated.rows.len());
        assert!(
            merged
                .rows
                .iter()
                .all(|row| matches!(row.status, CoverageStatus::Missing))
        );
        assert!(!merged.review_eligible());
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
