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
pub const DECISIONS: &[&str] = &["redraw", "table", "text", "decorative", "unreadable"];

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
    pub priority: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct VisualReport {
    pub low_text_pages: usize,
    pub entries: usize,
    pub redrawn: usize,
    pub tables: usize,
    pub text: usize,
    pub decorative: usize,
    pub unreadable: usize,
    pub missing: Vec<String>,
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
        if !path.is_file() {
            return Err(Error::Parse {
                path: path.to_path_buf(),
                message: "visual audit missing; create it with one decision per low-text page"
                    .to_string(),
            });
        }
        let audit = Self::load(path)?;
        let report = audit.validate(inventory, guide_dir);
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
        let mut expected = Vec::new();
        for source in &inventory.sources {
            let id = source.filename.trim_end_matches(".pdf");
            for page in &source.low_text_pages {
                expected.push(format!("{id}:{page}"));
            }
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
                "text" | "decorative" | "unreadable"
            ) && entry
                .reason
                .as_deref()
                .is_none_or(|reason| reason.trim().is_empty())
            {
                unresolved.push(format!("{key}: {} needs a reason", entry.decision));
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
        for key in &expected {
            if !seen.contains(key) {
                missing.push(key.clone());
            }
        }
        let passed = missing.is_empty() && unresolved.is_empty();
        VisualReport {
            low_text_pages: expected.len(),
            entries: seen.len(),
            redrawn: counts.get("redraw").copied().unwrap_or(0),
            tables: counts.get("table").copied().unwrap_or(0),
            text: counts.get("text").copied().unwrap_or(0),
            decorative: counts.get("decorative").copied().unwrap_or(0),
            unreadable: counts.get("unreadable").copied().unwrap_or(0),
            missing,
            unresolved,
            passed,
        }
    }
}

/// Asset paths must resolve inside the guide and point to an existing file.
/// `canonicalize` also rejects symlink escapes; a lexical `..` check is not
/// enough for a workspace boundary.
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
                low_text_pages,
                headings: vec![],
                sections: vec![],
                page_map: vec![PageRow {
                    page: 1,
                    first_line: String::new(),
                    chars: 0,
                    low_text: true,
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
            priority: Some("high".to_string()),
        }
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
