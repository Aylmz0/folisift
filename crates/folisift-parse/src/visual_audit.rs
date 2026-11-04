//! Visual-audit manifest: every low-text page needs a recorded disposition.
//!
//! The manifest is host-authored. Coverage of headings is not evidence that a
//! visual page was inspected, so the build gate requires this file for guides
//! that contain low-text pages.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result, io};
use crate::inventory::Inventory;

pub const VISUAL_AUDIT_SCHEMA_VERSION: u32 = 1;
pub const DECISIONS: &[&str] = &[
    "redraw",
    "redraw_figure",
    "redraw_table",
    "native_table",
    "table",
    "embed_crop",
    "describe",
    "text",
    "decorative",
    "unreadable",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VisualAudit {
    #[serde(default = "default_schema")]
    pub schema_version: u32,
    #[serde(default)]
    pub source_pages: Vec<VisualEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VisualEntry {
    pub source: String,
    pub page: usize,
    pub decision: String,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub asset: Option<String>,
    #[serde(default)]
    pub crop: Option<String>,
    #[serde(default)]
    pub caption: Option<String>,
    #[serde(default)]
    pub alt: Option<String>,
    #[serde(default)]
    pub hash: Option<String>,
    #[serde(default)]
    pub local_only: bool,
    #[serde(default)]
    pub priority: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct VisualReport {
    pub low_text_pages: usize,
    pub entries: usize,
    pub redrawn: usize,
    pub redraw_figures: usize,
    pub redraw_tables: usize,
    pub native_tables: usize,
    pub tables: usize,
    pub text: usize,
    pub decorative: usize,
    pub unreadable: usize,
    pub figure_pages: usize,
    pub visual_gate: String,
    pub missing: Vec<String>,
    pub missing_low_text: Vec<String>,
    pub missing_figures: Vec<String>,
    pub unresolved: Vec<String>,
    pub passed: bool,
}

impl VisualAudit {
    pub fn load(path: &Path) -> Result<Self> {
        let bytes = std::fs::read(path).map_err(|error| io(path, error))?;
        let audit: Self = serde_json::from_slice(&bytes).map_err(|source| Error::Parse {
            path: path.to_path_buf(),
            message: format!("visual audit JSON: {source}"),
        })?;
        if audit.schema_version != VISUAL_AUDIT_SCHEMA_VERSION {
            return Err(Error::Parse {
                path: path.to_path_buf(),
                message: format!(
                    "visual audit schema {} is not supported; expected {VISUAL_AUDIT_SCHEMA_VERSION}",
                    audit.schema_version
                ),
            });
        }
        Ok(audit)
    }

    /// Load and validate a manifest, with a single error surface for build
    /// gates. The caller supplies the guide directory so asset containment is
    /// resolved against the actual bundle.
    pub fn load_and_validate(
        path: &Path,
        inventory: &Inventory,
        guide_dir: &Path,
    ) -> Result<VisualReport> {
        Self::load_and_validate_with(path, inventory, guide_dir, false)
    }

    pub fn load_and_validate_with(
        path: &Path,
        inventory: &Inventory,
        guide_dir: &Path,
        include_figures: bool,
    ) -> Result<VisualReport> {
        if !path.is_file() {
            return Err(Error::Parse {
                path: path.to_path_buf(),
                message: "visual audit missing; create it with one decision per low-text page"
                    .to_string(),
            });
        }
        let audit = Self::load(path)?;
        let report = audit.validate_with(path, inventory, guide_dir, include_figures);
        if report.passed {
            Ok(report)
        } else {
            Err(Error::Parse {
                path: path.to_path_buf(),
                message: format!(
                    "visual audit failed: missing {:?}, unresolved {:?}",
                    report.missing, report.unresolved
                ),
            })
        }
    }

    /// Validate dispositions against the source inventory and guide assets.
    pub fn validate(&self, inventory: &Inventory, guide_dir: &Path) -> VisualReport {
        self.validate_with(Path::new(""), inventory, guide_dir, false)
    }

    pub fn validate_with(
        &self,
        _path: &Path,
        inventory: &Inventory,
        guide_dir: &Path,
        include_figures: bool,
    ) -> VisualReport {
        self.validate_with_policy(_path, inventory, guide_dir, include_figures, &[])
    }

    /// Validate with explicit intentional-page exemptions. Intentional pages
    /// are configured page numbers, not source-specific exceptions.
    pub fn validate_with_policy(
        &self,
        _path: &Path,
        inventory: &Inventory,
        guide_dir: &Path,
        include_figures: bool,
        intentional_pages: &[usize],
    ) -> VisualReport {
        let mut low_text = Vec::new();
        for source in &inventory.sources {
            let id = source.filename.trim_end_matches(".pdf");
            for page in &source.low_text_pages {
                low_text.push(format!("{id}:{page}"));
            }
        }
        low_text.sort();
        low_text.dedup();
        let figure_pages = if include_figures {
            figure_pages(guide_dir, inventory, intentional_pages)
        } else {
            Vec::new()
        };
        let mut expected = low_text.clone();
        for (source, page) in &figure_pages {
            expected.push(format!("{source}:{page}"));
        }
        expected.sort();
        expected.dedup();

        let mut seen = HashSet::new();
        let mut missing = Vec::new();
        let mut unresolved = Vec::new();
        let mut counts = HashMap::<&str, usize>::new();
        for entry in &self.source_pages {
            let key = format!("{}:{}", entry.source, entry.page);
            if !seen.insert(key.clone()) {
                unresolved.push(format!("{key}: duplicate entry"));
                continue;
            }
            if !expected.contains(&key) {
                unresolved.push(format!("{key}: not a low-text page in the inventory"));
                continue;
            }
            if !DECISIONS.contains(&entry.decision.as_str()) {
                unresolved.push(format!("{key}: unknown decision {:?}", entry.decision));
                continue;
            }
            if entry
                .priority
                .as_deref()
                .is_some_and(|priority| !matches!(priority, "low" | "medium" | "high"))
            {
                unresolved.push(format!("{key}: unknown priority {:?}", entry.priority));
            }
            *counts.entry(entry.decision.as_str()).or_default() += 1;
            if matches!(
                entry.decision.as_str(),
                "text" | "decorative" | "unreadable" | "describe"
            ) && entry
                .reason
                .as_deref()
                .is_none_or(|reason| reason.trim().is_empty())
            {
                unresolved.push(format!("{key}: {} needs a reason", entry.decision));
            }
            if entry.decision == "embed_crop" {
                for (name, value) in [
                    ("reason", entry.reason.as_deref()),
                    ("caption", entry.caption.as_deref()),
                    ("alt", entry.alt.as_deref()),
                    ("crop", entry.crop.as_deref()),
                    ("asset", entry.asset.as_deref()),
                    ("hash", entry.hash.as_deref()),
                ] {
                    if value.is_none_or(|value| value.trim().is_empty()) {
                        unresolved.push(format!("{key}: embed_crop needs {name}"));
                    }
                }
                if !entry.local_only {
                    unresolved.push(format!(
                        "{key}: embed_crop must be local_only; source crops are not committed"
                    ));
                }
                if let Some(crop) = entry.crop.as_deref()
                    && !valid_crop(crop)
                {
                    unresolved.push(format!(
                        "{key}: crop must be four normalized 0..1 values left,top,width,height"
                    ));
                }
                if let Some(asset) = entry.asset.as_deref() {
                    if !asset.starts_with("data/crops/") {
                        unresolved.push(format!(
                            "{key}: source crop asset {asset:?} must live under data/crops/"
                        ));
                    }
                    if !safe_asset_path(guide_dir, asset) {
                        unresolved
                            .push(format!("{key}: crop asset {asset:?} is missing or unsafe"));
                    } else if let Some(expected) = entry.hash.as_deref()
                        && let Ok(bytes) = std::fs::read(guide_dir.join(asset))
                        && blake3::hash(&bytes).to_hex().to_string() != expected
                    {
                        unresolved.push(format!("{key}: crop hash mismatch"));
                    }
                }
            }
            if entry.priority.as_deref() == Some("high") && entry.decision == "decorative" {
                unresolved.push(format!("{key}: a high-priority page cannot be decorative"));
            }
            if entry.decision == "redraw" {
                let Some(asset) = entry.asset.as_deref() else {
                    unresolved.push(format!("{key}: redraw needs an asset"));
                    continue;
                };
                if !safe_asset_path(guide_dir, asset) {
                    unresolved.push(format!("{key}: asset {asset:?} is missing or unsafe"));
                }
            }
        }
        let mut missing_low_text = Vec::new();
        let mut missing_figures = Vec::new();
        for key in &expected {
            if seen.contains(key) {
                continue;
            }
            missing.push(key.clone());
            if low_text.contains(key) {
                missing_low_text.push(key.clone());
            } else {
                missing_figures.push(key.clone());
            }
        }
        let passed = missing.is_empty() && unresolved.is_empty();
        VisualReport {
            low_text_pages: low_text.len(),
            entries: seen.len(),
            redrawn: counts.get("redraw").copied().unwrap_or(0),
            redraw_figures: counts.get("redraw_figure").copied().unwrap_or(0),
            redraw_tables: counts.get("redraw_table").copied().unwrap_or(0)
                + counts.get("table").copied().unwrap_or(0),
            native_tables: counts.get("native_table").copied().unwrap_or(0),
            tables: counts.get("table").copied().unwrap_or(0),
            text: counts.get("text").copied().unwrap_or(0),
            decorative: counts.get("decorative").copied().unwrap_or(0),
            unreadable: counts.get("unreadable").copied().unwrap_or(0),
            figure_pages: figure_pages.len(),
            visual_gate: if include_figures {
                "required".to_string()
            } else {
                "off".to_string()
            },
            missing,
            missing_low_text,
            missing_figures,
            unresolved,
            passed,
        }
    }
}

/// Asset paths must resolve inside the guide and point to an existing file.
/// `canonicalize` also rejects symlink escapes; a lexical `..` check is not
/// enough for a workspace boundary.
fn figure_pages(
    guide_dir: &Path,
    inventory: &Inventory,
    intentional_pages: &[usize],
) -> Vec<(String, usize)> {
    let mut pages = Vec::new();
    let blocks = guide_dir.join("data/blocks");
    let Ok(entries) = std::fs::read_dir(blocks) else {
        return pages;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(layout) = crate::layout::SourceLayout::read_json(&path) else {
            continue;
        };
        let source = layout.source.trim_end_matches(".pdf").to_string();
        if !inventory
            .sources
            .iter()
            .any(|item| item.filename.trim_end_matches(".pdf") == source)
        {
            continue;
        }
        for page in &layout.pages {
            if intentional_pages.contains(&page.page) {
                continue;
            }
            if page.blocks.iter().any(|block| {
                block.kind == crate::layout::LayoutBlockKind::Figure || block.kind.is_table()
            }) {
                pages.push((source.clone(), page.page));
            }
        }
    }
    pages.sort();
    pages.dedup();
    pages
}

fn safe_asset_path(guide_dir: &Path, asset: &str) -> bool {
    let path = PathBuf::from(asset);
    if path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return false;
    }
    let Ok(root) = guide_dir.canonicalize() else {
        return false;
    };
    let Ok(candidate) = guide_dir.join(path).canonicalize() else {
        return false;
    };
    candidate.starts_with(&root) && candidate.is_file()
}

fn valid_crop(value: &str) -> bool {
    let parts = value
        .split(',')
        .map(str::trim)
        .map(str::parse::<f32>)
        .collect::<std::result::Result<Vec<_>, _>>();
    let Ok(parts) = parts else {
        return false;
    };
    if parts.len() != 4 {
        return false;
    }
    parts.iter().all(|part| (0.0..=1.0).contains(part))
        && parts[2] > 0.0
        && parts[3] > 0.0
        && parts[0] + parts[2] <= 1.0
        && parts[1] + parts[3] <= 1.0
}

fn default_schema() -> u32 {
    VISUAL_AUDIT_SCHEMA_VERSION
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{Inventory, PageRow, SourceInventory, Totals};

    fn inventory(low_text_pages: Vec<usize>) -> Inventory {
        Inventory {
            generated_at: String::new(),
            low_text_threshold: 120,
            sources: vec![SourceInventory {
                path: String::new(),
                filename: "PD0.pdf".to_string(),
                title: String::new(),
                pages: 10,
                chars: 0,
                words: 0,
                low_text_threshold: 120,
                source_digest: None,
                low_text_pages,
                headings: vec![],
                layout: None,
                sections: vec![],
                page_map: vec![PageRow {
                    page: 1,
                    first_line: String::new(),
                    chars: 0,
                    low_text: true,
                    blocks: 0,
                    headings: 0,
                    tables: 0,
                }],
            }],
            totals: Totals {
                pages: 10,
                words: 0,
                low_text_pages: 1,
            },
        }
    }

    fn entry(page: usize, decision: &str) -> VisualEntry {
        VisualEntry {
            source: "PD0".to_string(),
            page,
            decision: decision.to_string(),
            reason: Some("reviewed".to_string()),
            asset: None,
            crop: None,
            caption: None,
            alt: None,
            hash: None,
            local_only: true,
            priority: Some("high".to_string()),
        }
    }

    #[test]
    fn figure_pages_enter_the_visual_gate() {
        let dir = tempfile::tempdir().expect("tempdir");
        let blocks = dir.path().join("data/blocks");
        let layout = crate::layout::SourceLayout::new(
            "PD0",
            1,
            vec![crate::layout::PageLayout {
                page: 1,
                page_label: None,
                width: 595.0,
                height: 842.0,
                content_bounds: None,
                blocks: vec![crate::layout::LayoutBlock {
                    id: "PD0:p1:b0".to_string(),
                    kind: crate::layout::LayoutBlockKind::Figure,
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
                    rows: Vec::new(),
                    header_rows: None,
                    figure_id: None,
                    figure_format: None,
                }],
            }],
        );
        layout.write_json(&blocks.join("PD0.json")).expect("layout");
        let inventory = Inventory {
            generated_at: String::new(),
            low_text_threshold: 100,
            sources: vec![crate::inventory::SourceInventory {
                path: "sources/PD0.pdf".to_string(),
                filename: "PD0.pdf".to_string(),
                title: "PD0".to_string(),
                pages: 1,
                chars: 0,
                words: 0,
                low_text_threshold: 100,
                source_digest: None,
                low_text_pages: Vec::new(),
                headings: Vec::new(),
                layout: None,
                sections: Vec::new(),
                page_map: Vec::new(),
            }],
            totals: crate::inventory::Totals {
                pages: 1,
                words: 0,
                low_text_pages: 0,
            },
        };
        let audit = VisualAudit {
            schema_version: 1,
            source_pages: Vec::new(),
        };
        let report = audit.validate_with(Path::new(""), &inventory, dir.path(), true);
        assert_eq!(report.figure_pages, 1);
        assert!(report.missing.contains(&"PD0:1".to_string()));
        assert!(!report.passed);
    }

    #[test]
    fn embed_crop_requires_provenance_and_describe_needs_a_reason() {
        let inventory = inventory(vec![1]);
        let dir = tempfile::tempdir().expect("tempdir");
        let mut crop = entry(1, "embed_crop");
        crop.caption = None;
        let audit = VisualAudit {
            schema_version: 1,
            source_pages: vec![crop],
        };
        let report = audit.validate(&inventory, dir.path());
        assert!(!report.passed);
        assert!(
            report
                .unresolved
                .iter()
                .any(|item| item.contains("embed_crop needs caption"))
        );
        let mut describe = entry(1, "describe");
        describe.reason = None;
        let audit = VisualAudit {
            schema_version: 1,
            source_pages: vec![describe],
        };
        assert!(
            audit
                .validate(&inventory, dir.path())
                .unresolved
                .iter()
                .any(|item| item.contains("describe needs a reason"))
        );
    }

    #[test]
    fn intentional_pages_are_exempt_from_figure_evidence() {
        let dir = tempfile::tempdir().expect("tempdir");
        let blocks = dir.path().join("data/blocks");
        let layout = crate::layout::SourceLayout::new(
            "PD0",
            1,
            vec![crate::layout::PageLayout {
                page: 1,
                page_label: None,
                width: 595.0,
                height: 842.0,
                content_bounds: None,
                blocks: vec![crate::layout::LayoutBlock {
                    id: "PD0:p1:b0".to_string(),
                    kind: crate::layout::LayoutBlockKind::Figure,
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
                    rows: Vec::new(),
                    header_rows: None,
                    figure_id: None,
                    figure_format: None,
                }],
            }],
        );
        layout.write_json(&blocks.join("PD0.json")).expect("layout");
        let inventory = inventory(Vec::new());
        let audit = VisualAudit {
            schema_version: 1,
            source_pages: Vec::new(),
        };
        let report = audit.validate_with_policy(Path::new(""), &inventory, dir.path(), true, &[1]);
        assert_eq!(report.figure_pages, 0);
        assert!(report.missing.is_empty(), "{:?}", report.missing);
        assert!(report.passed);
    }

    #[test]
    fn missing_low_text_and_figure_pages_are_reported_separately() {
        let inventory = inventory(vec![1]);
        let dir = tempfile::tempdir().expect("tempdir");
        let audit = VisualAudit {
            schema_version: 1,
            source_pages: Vec::new(),
        };
        let report = audit.validate_with(Path::new(""), &inventory, dir.path(), false);
        assert_eq!(report.missing_low_text, vec!["PD0:1"]);
        assert!(report.missing_figures.is_empty());
        assert!(!report.passed);
    }

    #[test]
    fn embed_crop_requires_hash_and_local_only_policy() {
        let inventory = inventory(vec![1]);
        let dir = tempfile::tempdir().expect("tempdir");
        let mut crop = entry(1, "embed_crop");
        crop.reason = Some("chart".to_string());
        crop.caption = Some("ROC".to_string());
        crop.alt = Some("ROC eğrisi".to_string());
        crop.crop = Some("0.1,0.1,0.5,0.5".to_string());
        crop.asset = Some("data/crops/x.png".to_string());
        crop.hash = None;
        crop.local_only = false;
        let audit = VisualAudit {
            schema_version: 1,
            source_pages: vec![crop],
        };
        let report = audit.validate(&inventory, dir.path());
        assert!(
            report
                .unresolved
                .iter()
                .any(|item| item.contains("needs hash"))
        );
        assert!(
            report
                .unresolved
                .iter()
                .any(|item| item.contains("local_only"))
        );
    }

    #[test]
    fn every_low_text_page_needs_a_decision() {
        let audit = VisualAudit {
            schema_version: 1,
            source_pages: vec![entry(1, "text")],
        };
        let report = audit.validate(&inventory(vec![1, 2]), Path::new("."));
        assert!(!report.passed);
        assert_eq!(report.missing, vec!["PD0:2"]);
    }

    #[test]
    fn decisions_without_reasons_are_unresolved() {
        let mut audit = VisualAudit {
            schema_version: 1,
            source_pages: vec![entry(1, "decorative")],
        };
        audit.source_pages[0].reason = None;
        audit.source_pages[0].priority = Some("low".to_string());
        let report = audit.validate(&inventory(vec![1]), Path::new("."));
        assert!(!report.passed);
        assert_eq!(report.unresolved.len(), 1);
    }

    #[test]
    fn redraw_requires_an_existing_asset_inside_the_guide() {
        let mut audit = VisualAudit {
            schema_version: 1,
            source_pages: vec![entry(1, "redraw")],
        };
        audit.source_pages[0].asset = Some("../outside.json".to_string());
        let report = audit.validate(&inventory(vec![1]), Path::new("."));
        assert!(!report.passed);
        assert!(report.unresolved[0].contains("unsafe"));
    }

    #[test]
    fn passing_manifest_counts_decisions_and_resolves_assets() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(root.path().join("diagrams")).expect("diagrams");
        std::fs::write(root.path().join("diagrams/flow.json"), "{}").expect("asset");
        let mut redraw = entry(1, "redraw");
        redraw.asset = Some("diagrams/flow.json".to_string());
        let audit = VisualAudit {
            schema_version: 1,
            source_pages: vec![redraw, entry(2, "text")],
        };
        let report = audit.validate(&inventory(vec![1, 2]), root.path());
        assert!(report.passed, "{report:?}");
        assert_eq!(report.redrawn, 1);
        assert_eq!(report.text, 1);
    }

    #[test]
    fn rejects_duplicates_unknown_decisions_and_high_priority_decorative() {
        let mut duplicate = VisualAudit {
            schema_version: 1,
            source_pages: vec![entry(1, "text"), entry(1, "text")],
        };
        let report = duplicate.validate(&inventory(vec![1]), Path::new("."));
        assert!(!report.passed);
        assert!(
            report
                .unresolved
                .iter()
                .any(|item| item.contains("duplicate"))
        );

        duplicate.source_pages = vec![entry(1, "mystery")];
        let report = duplicate.validate(&inventory(vec![1]), Path::new("."));
        assert!(!report.passed);
        assert!(
            report
                .unresolved
                .iter()
                .any(|item| item.contains("unknown decision"))
        );

        duplicate.source_pages = vec![entry(1, "decorative")];
        duplicate.source_pages[0].priority = Some("high".to_string());
        let report = duplicate.validate(&inventory(vec![1]), Path::new("."));
        assert!(!report.passed);
        assert!(
            report
                .unresolved
                .iter()
                .any(|item| item.contains("high-priority"))
        );
    }
}
