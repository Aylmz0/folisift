//! Visual-audit manifest: every low-text page needs a recorded disposition.
//!
//! The manifest is host-authored. Coverage of headings is not evidence that a
//! visual page was inspected, so the build gate requires this file for guides
//! that contain low-text pages.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{Error, Result, io};
use crate::inventory::Inventory;
use crate::visual_quality::{QualityReport, inspect};

pub const VISUAL_AUDIT_SCHEMA_VERSION: u32 = 1;
pub const DECISIONS: &[&str] = &[
    "redraw",
    "redraw_figure",
    "redraw_table",
    "native_table",
    "table",
    "embed_crop",
    "embed_image",
    "describe",
    "text",
    "decorative",
    "unreadable",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VisualAudit {
    #[serde(default = "default_schema")]
    pub schema_version: u32,
    #[serde(default)]
    pub source_pages: Vec<VisualEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub candidate_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bbox: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dpi: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pixel_width: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pixel_height: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_dpi: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relevance: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reconstruction: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redraw_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schematic_of: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub renderer: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality: Option<QualityReport>,
    #[serde(default)]
    pub quality_exception: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
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
    pub embedded_crops: usize,
    pub embedded_images: usize,
    pub concept_diagrams: usize,
    pub schematic_diagrams: usize,
    pub text: usize,
    pub decorative: usize,
    pub unreadable: usize,
    pub figure_pages: usize,
    pub visual_gate: String,
    pub missing: Vec<String>,
    pub missing_low_text: Vec<String>,
    pub missing_figures: Vec<String>,
    pub unresolved: Vec<String>,
    pub warnings: Vec<String>,
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

        let mut seen_keys = HashSet::new();
        let mut seen_pages = HashSet::new();
        let mut missing = Vec::new();
        let mut unresolved = Vec::new();
        let mut warnings = Vec::new();
        let mut asset_hashes = HashMap::<String, String>::new();
        let mut counts = HashMap::<&str, usize>::new();
        for entry in &self.source_pages {
            let page_key = format!("{}:{}", entry.source, entry.page);
            let candidate_key = entry.candidate_id.as_deref().unwrap_or("-");
            let key = format!("{page_key}:{candidate_key}");
            if !seen_keys.insert(key.clone()) {
                unresolved.push(format!("{key}: duplicate entry"));
                continue;
            }
            if !expected.contains(&page_key) {
                unresolved.push(format!("{key}: not a low-text page in the inventory"));
                continue;
            }
            seen_pages.insert(page_key.clone());
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
            if let Some(relevance) = entry.relevance.as_deref()
                && !matches!(
                    relevance,
                    "core" | "supporting" | "decorative" | "seductive"
                )
            {
                unresolved.push(format!("{key}: unknown relevance {relevance:?}"));
            }
            if let Some(reconstruction) = entry.reconstruction.as_deref()
                && !matches!(
                    reconstruction,
                    "d2_concept" | "d2_schematic" | "table" | "text" | "unique"
                )
            {
                unresolved.push(format!("{key}: unknown reconstruction {reconstruction:?}"));
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

            if matches!(entry.decision.as_str(), "embed_crop" | "embed_image") {
                validate_embed_entry(
                    entry,
                    &key,
                    guide_dir,
                    &mut unresolved,
                    &mut warnings,
                    &mut asset_hashes,
                );
                if matches!(entry.relevance.as_deref(), Some("decorative" | "seductive"))
                    && !entry.quality_exception
                    && !entry
                        .notes
                        .as_deref()
                        .is_some_and(|notes| notes.to_lowercase().contains("override"))
                {
                    unresolved.push(format!(
                        "{key}: decorative/seductive embed needs a quality_exception or an explicit override note"
                    ));
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
                if let Some(kind) = entry.redraw_kind.as_deref() {
                    if !matches!(kind, "concept" | "schematic") {
                        unresolved.push(format!("{key}: unknown redraw_kind {kind:?}"));
                    }
                    if kind == "schematic"
                        && !entry.caption.as_deref().is_some_and(|caption| {
                            let caption = caption.to_lowercase();
                            caption.contains("şematik") || caption.contains("schematic")
                        })
                    {
                        unresolved.push(format!(
                            "{key}: schematic redraw caption must contain “şematik” or “schematic”"
                        ));
                    }
                }
                if let Some(reference) = entry.schematic_of.as_deref()
                    && reference.starts_with("data/")
                    && !safe_asset_path(guide_dir, reference)
                {
                    unresolved.push(format!(
                        "{key}: schematic_of asset {reference:?} is missing or unsafe"
                    ));
                }
            }

            if let Some(expected_digest) = entry.source_digest.as_deref()
                && let Some(source) = inventory
                    .sources
                    .iter()
                    .find(|source| source.filename.trim_end_matches(".pdf") == entry.source)
                && let Some(actual) = source.source_digest.as_deref()
                && actual != expected_digest
            {
                warnings.push(format!(
                    "{key}: source digest changed; run visual assets --materialize --refresh"
                ));
            }
        }

        let mut missing_low_text = Vec::new();
        let mut missing_figures = Vec::new();
        for key in &expected {
            if seen_pages.contains(key) {
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
            entries: seen_keys.len(),
            redrawn: counts.get("redraw").copied().unwrap_or(0),
            redraw_figures: counts.get("redraw_figure").copied().unwrap_or(0),
            redraw_tables: counts.get("redraw_table").copied().unwrap_or(0)
                + counts.get("table").copied().unwrap_or(0),
            native_tables: counts.get("native_table").copied().unwrap_or(0),
            tables: counts.get("table").copied().unwrap_or(0),
            embedded_crops: counts.get("embed_crop").copied().unwrap_or(0),
            embedded_images: counts.get("embed_image").copied().unwrap_or(0),
            concept_diagrams: self
                .source_pages
                .iter()
                .filter(|entry| entry.redraw_kind.as_deref() == Some("concept"))
                .count(),
            schematic_diagrams: self
                .source_pages
                .iter()
                .filter(|entry| entry.redraw_kind.as_deref() == Some("schematic"))
                .count(),
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
            warnings,
            passed,
        }
    }

    pub fn load_or_default(path: &Path) -> Result<Self> {
        if path.is_file() {
            Self::load(path)
        } else {
            Ok(Self {
                schema_version: VISUAL_AUDIT_SCHEMA_VERSION,
                source_pages: Vec::new(),
            })
        }
    }

    /// Write the manifest atomically.
    pub fn save(&self, path: &Path) -> Result<()> {
        let text = serde_json::to_string_pretty(self).map_err(|error| Error::Parse {
            path: path.to_path_buf(),
            message: format!("serialize visual audit: {error}"),
        })?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| io(parent, error))?;
        }
        let temporary = path.with_extension("json.tmp");
        std::fs::write(&temporary, text + "\n").map_err(|error| io(&temporary, error))?;
        std::fs::rename(&temporary, path).map_err(|error| io(path, error))?;
        Ok(())
    }

    /// Insert or replace one candidate decision. The write is atomic.
    pub fn upsert(path: &Path, entry: VisualEntry) -> Result<()> {
        let mut audit = Self::load_or_default(path)?;
        let key = (entry.source.clone(), entry.page, entry.candidate_id.clone());
        if let Some(existing) = audit.source_pages.iter_mut().find(|existing| {
            (
                existing.source.clone(),
                existing.page,
                existing.candidate_id.clone(),
            ) == key
        }) {
            *existing = entry;
        } else {
            audit.source_pages.push(entry);
        }
        audit.source_pages.sort_by(|a, b| {
            a.source
                .cmp(&b.source)
                .then_with(|| a.page.cmp(&b.page))
                .then_with(|| a.candidate_id.cmp(&b.candidate_id))
        });
        audit.save(path)
    }
}

fn validate_embed_entry(
    entry: &VisualEntry,
    key: &str,
    guide_dir: &Path,
    unresolved: &mut Vec<String>,
    warnings: &mut Vec<String>,
    asset_hashes: &mut HashMap<String, String>,
) {
    let image = entry.decision == "embed_image";
    for (name, value) in [
        ("reason", entry.reason.as_deref()),
        ("caption", entry.caption.as_deref()),
        ("alt", entry.alt.as_deref()),
        ("asset", entry.asset.as_deref()),
        ("hash", entry.hash.as_deref()),
    ] {
        if value.is_none_or(|value| value.trim().is_empty()) {
            unresolved.push(format!("{key}: {} needs {name}", entry.decision));
        }
    }
    if image {
        if entry
            .image_id
            .as_deref()
            .is_none_or(|value| value.trim().is_empty())
        {
            unresolved.push(format!("{key}: embed_image needs image_id"));
        }
    } else if entry
        .crop
        .as_deref()
        .is_none_or(|value| value.trim().is_empty())
    {
        unresolved.push(format!("{key}: embed_crop needs crop"));
    }
    if !entry.local_only {
        unresolved.push(format!(
            "{key}: {} must be local_only; source assets are not committed",
            entry.decision
        ));
    }
    if !image
        && let Some(crop) = entry.crop.as_deref()
        && !valid_crop(crop)
    {
        unresolved.push(format!(
            "{key}: crop must be four normalized 0..1 values left,top,width,height"
        ));
    }
    let Some(asset) = entry.asset.as_deref() else {
        return;
    };
    if image && !asset.starts_with("data/figures/") {
        unresolved.push(format!(
            "{key}: native image asset {asset:?} must live under data/figures/"
        ));
    }
    if !image && !asset.starts_with("data/crops/") {
        unresolved.push(format!(
            "{key}: source crop asset {asset:?} must live under data/crops/"
        ));
    }
    if !safe_asset_path(guide_dir, asset) {
        unresolved.push(format!("{key}: asset {asset:?} is missing or unsafe"));
        return;
    }
    let bytes = match std::fs::read(guide_dir.join(asset)) {
        Ok(bytes) => bytes,
        Err(error) => {
            unresolved.push(format!("{key}: cannot read asset {asset:?}: {error}"));
            return;
        }
    };
    if let Some(expected) = entry.hash.as_deref()
        && blake3::hash(&bytes).to_hex().to_string() != expected
    {
        unresolved.push(format!("{key}: asset hash mismatch"));
    }
    if let Some(caption) = entry.caption.as_deref() {
        validate_caption(entry, key, caption, unresolved);
    }
    if let Some(hash) = entry.hash.as_deref()
        && let Some(previous) = asset_hashes.insert(hash.to_string(), key.to_string())
        && previous != key
    {
        warnings.push(format!(
            "{key}: same asset hash is also used by {previous}; verify intentional reuse"
        ));
    }
    match inspect(&bytes, entry.effective_dpi) {
        Ok(quality) => {
            for failure in quality.hard_failures(entry.quality_exception) {
                unresolved.push(format!("{key}: {failure}"));
            }
            for warning in quality.warnings() {
                warnings.push(format!("{key}: {warning}"));
            }
        }
        Err(error) => unresolved.push(format!("{key}: quality check failed: {error}")),
    }
}

fn validate_caption(entry: &VisualEntry, key: &str, caption: &str, unresolved: &mut Vec<String>) {
    let length = caption.trim().chars().count();
    if !(8..=200).contains(&length) {
        unresolved.push(format!("{key}: caption must be 8–200 characters"));
    }
    if caption.to_lowercase().contains("lorem")
        || caption.contains("TODO")
        || caption.contains("...")
    {
        unresolved.push(format!("{key}: caption looks like a placeholder"));
    }
    if label_only_regex().is_match(caption.trim()) {
        unresolved.push(format!(
            "{key}: caption must state a takeaway, not only a figure/table label"
        ));
    }
    if let Some(alt) = entry.alt.as_deref() {
        let alt_length = alt.trim().chars().count();
        if !(12..=300).contains(&alt_length) {
            unresolved.push(format!("{key}: alt must be 12–300 characters"));
        }
        if alt.trim() == caption.trim() {
            unresolved.push(format!("{key}: alt must differ from the caption"));
        }
    }
}

fn label_only_regex() -> &'static Regex {
    static CAPTION: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    CAPTION.get_or_init(|| {
        Regex::new(r"(?i)^(şekil|sekil|figure|tablo|table)\s*\d+[.:]?$")
            .expect("label-only caption regex")
    })
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
            candidate_id: None,
            asset_kind: None,
            image_id: None,
            bbox: None,
            dpi: None,
            pixel_width: None,
            pixel_height: None,
            effective_dpi: None,
            relevance: None,
            reconstruction: None,
            redraw_kind: None,
            schematic_of: None,
            source_digest: None,
            renderer: None,
            quality: None,
            quality_exception: false,
            notes: None,
            extra: BTreeMap::new(),
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

    fn write_png(path: &Path) -> String {
        std::fs::create_dir_all(path.parent().expect("parent")).expect("dirs");
        let mut image = image::RgbaImage::new(400, 400);
        for (x, y, pixel) in image.enumerate_pixels_mut() {
            *pixel = if (x / 20 + y / 20) % 2 == 0 {
                image::Rgba([30, 90, 150, 255])
            } else {
                image::Rgba([240, 220, 180, 255])
            };
        }
        let mut bytes = Vec::new();
        image::DynamicImage::ImageRgba8(image)
            .write_to(
                &mut std::io::Cursor::new(&mut bytes),
                image::ImageFormat::Png,
            )
            .expect("png");
        std::fs::write(path, &bytes).expect("write png");
        blake3::hash(&bytes).to_hex().to_string()
    }

    #[test]
    fn embed_crop_with_a_real_asset_passes_quality_and_provenance() {
        let root = tempfile::tempdir().expect("tempdir");
        let asset = "data/crops/x.png";
        let hash = write_png(&root.path().join(asset));
        let mut crop = entry(1, "embed_crop");
        crop.crop = Some("0.1,0.1,0.5,0.5".to_string());
        crop.asset = Some(asset.to_string());
        crop.hash = Some(hash);
        crop.caption = Some("ROC eğrisi ve eşik noktaları".to_string());
        crop.alt = Some("ROC eğrisi; yatay eksende FPR, dikey eksende TPR".to_string());
        crop.effective_dpi = Some(200.0);
        let audit = VisualAudit {
            schema_version: 1,
            source_pages: vec![crop],
        };
        let report = audit.validate(&inventory(vec![1]), root.path());
        assert!(report.passed, "{report:?}");
    }

    #[test]
    fn embed_image_requires_an_image_id() {
        let root = tempfile::tempdir().expect("tempdir");
        let asset = "data/figures/x.png";
        let hash = write_png(&root.path().join(asset));
        let mut image = entry(1, "embed_image");
        image.asset = Some(asset.to_string());
        image.hash = Some(hash);
        image.caption = Some("Sözdizim grafiği (kaynak görsel)".to_string());
        image.alt = Some("Sözdizim grafiği; ifade ve terim düğümleri".to_string());
        image.effective_dpi = Some(200.0);
        let audit = VisualAudit {
            schema_version: 1,
            source_pages: vec![image],
        };
        let report = audit.validate(&inventory(vec![1]), root.path());
        assert!(
            report
                .unresolved
                .iter()
                .any(|item| item.contains("needs image_id"))
        );
    }

    #[test]
    fn several_candidates_can_have_decisions_on_one_page() {
        let mut first = entry(1, "text");
        first.candidate_id = Some("PD0:p1:fig1".to_string());
        let mut second = entry(1, "describe");
        second.candidate_id = Some("PD0:p1:fig2".to_string());
        second.reason = Some("İkinci görsel metinle açıklandı.".to_string());
        let audit = VisualAudit {
            schema_version: 1,
            source_pages: vec![first, second],
        };
        let report = audit.validate(&inventory(vec![1]), Path::new("."));
        assert!(report.passed, "{report:?}");
        assert_eq!(report.entries, 2);
    }

    #[test]
    fn a_schematic_redraw_requires_a_schematic_caption() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(root.path().join("diagrams")).expect("diagrams");
        std::fs::write(root.path().join("diagrams/flow.json"), "{}").expect("asset");
        let mut redraw = entry(1, "redraw");
        redraw.asset = Some("diagrams/flow.json".to_string());
        redraw.redraw_kind = Some("schematic".to_string());
        redraw.caption = Some("Küme ayrımı".to_string());
        let audit = VisualAudit {
            schema_version: 1,
            source_pages: vec![redraw.clone()],
        };
        let report = audit.validate(&inventory(vec![1]), root.path());
        assert!(
            report
                .unresolved
                .iter()
                .any(|item| item.contains("şematik"))
        );
        redraw.caption = Some("Küme ayrımı (şematik)".to_string());
        let audit = VisualAudit {
            schema_version: 1,
            source_pages: vec![redraw],
        };
        assert!(audit.validate(&inventory(vec![1]), root.path()).passed);
    }
}
