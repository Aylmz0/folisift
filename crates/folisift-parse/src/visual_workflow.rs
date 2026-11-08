//! Host-facing decision and asset-maintenance helpers.
//!
//! These functions are shared by the CLI and MCP tools so a decision is
//! materialized and recorded the same way everywhere.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::error::{Error, Result};
use crate::layout::{PageLayout, Rect, SourceLayout};
use crate::visual_assets::{
    MAX_CROP_DPI, MIN_CROP_DPI, crop_from_bbox, extract_native, render_crop,
};
use crate::visual_audit::{VisualAudit, VisualEntry};
use crate::visual_candidates::{VisualCandidate, collect_candidates};

#[derive(Debug, Clone, Default)]
pub struct DecisionInput {
    pub source: String,
    pub page: usize,
    pub candidate_id: Option<String>,
    pub decision: String,
    pub reason: String,
    pub caption: Option<String>,
    pub alt: Option<String>,
    pub priority: Option<String>,
    pub asset: Option<String>,
    pub crop: Option<String>,
    pub bbox: Option<Rect>,
    pub dpi: Option<f32>,
    pub padding: Option<f32>,
    pub image_id: Option<String>,
    pub width: Option<u8>,
    pub redraw_kind: Option<String>,
    pub schematic_of: Option<String>,
    pub quality_exception: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DecisionOutcome {
    pub guide: String,
    pub candidate_id: Option<String>,
    pub decision: String,
    pub asset: Option<String>,
    pub hash: Option<String>,
    pub markdown: String,
    pub audit: String,
    pub content_unchanged: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct AssetReport {
    pub guide: String,
    pub action: String,
    pub checked: usize,
    pub ok: usize,
    pub missing: usize,
    pub mismatched: usize,
    pub refreshed: usize,
    pub issues: Vec<String>,
}

/// Collect every missing or invalid decision field before any file is written.
///
/// The caller resolves the candidate and stored layout page first, so
/// `embed_crop` can be accepted from a crop alone or a candidate alone.
pub fn validate_decision(
    input: &DecisionInput,
    candidate: Option<&VisualCandidate>,
    layout_page: Option<&PageLayout>,
) -> Vec<String> {
    let mut errors = Vec::new();
    let embedding = matches!(input.decision.as_str(), "embed_crop" | "embed_image");
    if embedding {
        if input.reason.trim().is_empty() {
            errors.push(format!("{} requires a reason", input.decision));
        }
        if input
            .caption
            .as_deref()
            .is_none_or(|value| value.trim().is_empty())
        {
            errors.push(format!("{} requires a caption", input.decision));
        }
        if input
            .alt
            .as_deref()
            .is_none_or(|value| value.trim().is_empty())
        {
            errors.push(format!("{} requires alt text", input.decision));
        }
        if let Some(width) = input.width
            && !(35..=100).contains(&width)
        {
            errors.push(format!(
                "{} width {width} is outside 35..=100",
                input.decision
            ));
        }
        if let Some(dpi) = input.dpi
            && !(MIN_CROP_DPI..=MAX_CROP_DPI).contains(&dpi)
        {
            errors.push(format!(
                "{} dpi {dpi} is outside {MIN_CROP_DPI}..={MAX_CROP_DPI}",
                input.decision
            ));
        }
    }
    match input.decision.as_str() {
        "embed_crop" => {
            let has_bbox = input.bbox.is_some();
            let has_crop = input
                .crop
                .as_deref()
                .is_some_and(|crop| !crop.trim().is_empty());
            let has_candidate = candidate.is_some();
            if !has_bbox && !has_crop && !has_candidate {
                errors.push("embed_crop requires bbox or crop or candidate_id".to_string());
            }
            if let Some(crop) = input.crop.as_deref()
                && !crop.trim().is_empty()
                && let Err(error) = crate::render::CropBox::parse(crop)
            {
                errors.push(format!("embed_crop crop is invalid: {error}"));
            }
            if !has_bbox && has_crop && !has_candidate && layout_page.is_none() {
                errors.push(
                    "embed_crop with crop alone requires a stored layout page for the bbox"
                        .to_string(),
                );
            }
            if has_bbox
                && !has_crop
                && !has_candidate
                && page_dimensions(candidate, layout_page).is_none()
            {
                errors.push(
                    "embed_crop with bbox alone requires candidate_id or a stored layout page to derive the crop"
                        .to_string(),
                );
            }
        }
        "embed_image" => {
            let has_image_id = input
                .image_id
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty());
            let candidate_image = candidate
                .and_then(|candidate| candidate.image_id.as_deref())
                .is_some_and(|value| !value.trim().is_empty());
            if !has_image_id && !candidate_image {
                errors.push(
                    "embed_image requires image_id or a candidate_id with an embedded image"
                        .to_string(),
                );
            }
        }
        "redraw" => {
            if input
                .asset
                .as_deref()
                .is_none_or(|value| value.trim().is_empty())
            {
                errors.push("redraw requires an existing diagram asset".to_string());
            }
            if input.redraw_kind.as_deref() == Some("schematic") {
                let caption = input.caption.as_deref().unwrap_or("").to_lowercase();
                if !caption.contains("şematik") && !caption.contains("schematic") {
                    errors.push(
                        "schematic redraw caption must contain “şematik” or “schematic”"
                            .to_string(),
                    );
                }
            }
        }
        _ => {}
    }
    errors.sort();
    errors.dedup();
    errors
}

fn layout_page(guide_dir: &Path, source_id: &str, page: usize) -> Option<PageLayout> {
    let path = guide_dir
        .join("data/blocks")
        .join(format!("{source_id}.json"));
    SourceLayout::read_json(&path)
        .ok()?
        .pages
        .into_iter()
        .find(|candidate| candidate.page == page)
}

fn page_dimensions(
    candidate: Option<&VisualCandidate>,
    layout_page: Option<&PageLayout>,
) -> Option<(f32, f32)> {
    candidate
        .map(|candidate| (candidate.page_width, candidate.page_height))
        .or_else(|| layout_page.map(|page| (page.width, page.height)))
}

fn bbox_from_crop(crop: &crate::render::CropBox, page_width: f32, page_height: f32) -> Rect {
    Rect {
        x: crop.left * page_width,
        y: crop.top * page_height,
        width: crop.width * page_width,
        height: crop.height * page_height,
    }
}

pub async fn apply_decision(
    guide_slug: &str,
    guide_dir: &Path,
    source_path: &Path,
    source_id: &str,
    mut input: DecisionInput,
) -> Result<DecisionOutcome> {
    if !crate::visual_audit::DECISIONS.contains(&input.decision.as_str()) {
        return Err(Error::PageSelection {
            value: format!(
                "unknown visual decision {:?}; use one of {:?}",
                input.decision,
                crate::visual_audit::DECISIONS
            ),
        });
    }
    if input
        .priority
        .as_deref()
        .is_some_and(|priority| !matches!(priority, "low" | "medium" | "high"))
    {
        return Err(Error::PageSelection {
            value: format!(
                "unknown priority {:?}; use low, medium or high",
                input.priority
            ),
        });
    }
    if input
        .redraw_kind
        .as_deref()
        .is_some_and(|kind| !matches!(kind, "concept" | "schematic"))
    {
        return Err(Error::PageSelection {
            value: format!(
                "unknown redraw_kind {:?}; use concept or schematic",
                input.redraw_kind
            ),
        });
    }
    let candidate = if let Some(candidate_id) = input.candidate_id.as_deref() {
        let report = collect_candidates(
            source_path,
            source_id,
            Some(guide_slug.to_string()),
            Some(&input.page.to_string()),
            0,
            100,
        )
        .await?;
        let candidate = report
            .candidates
            .into_iter()
            .find(|candidate| candidate.id == candidate_id)
            .ok_or_else(|| Error::Parse {
                path: source_path.to_path_buf(),
                message: format!(
                    "candidate {candidate_id:?} not found on page {}",
                    input.page
                ),
            })?;
        Some(candidate)
    } else {
        None
    };

    if let Some(candidate) = candidate.as_ref() {
        input.page = candidate.page;
        input.candidate_id = Some(candidate.id.clone());
        if input.bbox.is_none() {
            input.bbox = Some(candidate.bbox);
        }
        if input.image_id.is_none() {
            input.image_id = candidate.image_id.clone();
        }
        if input.width.is_none() {
            input.width = Some(candidate.recommended_width_percent);
        }
    }

    // Validate the complete decision before touching the audit or asset tree.
    // A crop alone is valid when the stored layout page supplies the bbox.
    let stored_page = layout_page(guide_dir, source_id, input.page);
    let errors = validate_decision(&input, candidate.as_ref(), stored_page.as_ref());
    if !errors.is_empty() {
        let detail = errors
            .iter()
            .map(|error| format!("- {error}"))
            .collect::<Vec<_>>()
            .join("\n");
        return Err(Error::PageSelection {
            value: format!(
                "invalid visual decision:\n{detail}\nnext: supply every listed field and retry; no files were written"
            ),
        });
    }

    let mut asset = input.asset.clone();
    let preexisting_asset = asset
        .as_deref()
        .is_some_and(|value| guide_dir.join(value).is_file());
    let mut hash = None;
    let mut crop = input.crop.clone();
    let mut pixel_width = None;
    let mut pixel_height = None;
    let mut effective_dpi = None;
    let mut quality = None;
    let mut asset_kind = None;
    let mut image_id = input.image_id.clone();

    match input.decision.as_str() {
        "embed_image" => {
            let requested_image_id = image_id.as_deref().ok_or_else(|| Error::PageSelection {
                value: "embed_image requires image_id".to_string(),
            })?;
            let native = extract_native(
                source_path,
                guide_dir,
                input.page as u32,
                requested_image_id,
                asset.as_deref(),
            )
            .await?;
            let failures = native.quality.hard_failures(input.quality_exception);
            if !failures.is_empty() {
                if !preexisting_asset {
                    let _ = std::fs::remove_file(guide_dir.join(&native.asset));
                }
                return Err(Error::PageSelection {
                    value: format!("asset quality failed: {}", failures.join("; ")),
                });
            }
            asset = Some(native.asset.clone());
            hash = Some(native.hash.clone());
            pixel_width = Some(native.width);
            pixel_height = Some(native.height);
            effective_dpi = native.effective_dpi;
            quality = Some(native.quality);
            asset_kind = Some("native".to_string());
            image_id = Some(native.image_id.clone());
        }
        "embed_crop" => {
            let crop_box = if let Some(value) = crop.as_deref() {
                let parsed = crate::render::CropBox::parse(value)?;
                if input.bbox.is_none() {
                    let (page_width, page_height) = page_dimensions(
                        candidate.as_ref(),
                        stored_page.as_ref(),
                    )
                    .ok_or_else(|| Error::PageSelection {
                        value:
                            "embed_crop with crop alone requires a stored layout page for the bbox"
                                .to_string(),
                    })?;
                    input.bbox = Some(bbox_from_crop(&parsed, page_width, page_height));
                }
                parsed
            } else {
                let bbox = input.bbox.ok_or_else(|| Error::PageSelection {
                    value: "embed_crop requires bbox or crop or candidate_id".to_string(),
                })?;
                let (page_width, page_height) =
                    page_dimensions(candidate.as_ref(), stored_page.as_ref()).ok_or_else(|| {
                        Error::PageSelection {
                            value:
                                "embed_crop with bbox alone requires candidate_id or a stored layout page to derive the crop"
                                    .to_string(),
                        }
                    })?;
                crop_from_bbox(bbox, page_width, page_height, input.padding.unwrap_or(4.0))?
            };
            let bbox = input.bbox.ok_or_else(|| Error::PageSelection {
                value: "embed_crop requires bbox or crop or candidate_id".to_string(),
            })?;
            crop = Some(format!(
                "{},{},{},{}",
                crop_box.left, crop_box.top, crop_box.width, crop_box.height
            ));
            let rendered = render_crop(
                source_path,
                guide_dir,
                input.page as u32,
                crop_box,
                bbox,
                input.dpi.unwrap_or(300.0),
                asset.as_deref(),
            )
            .await?;
            let failures = rendered.quality.hard_failures(input.quality_exception);
            if !failures.is_empty() {
                if !preexisting_asset {
                    let _ = std::fs::remove_file(guide_dir.join(&rendered.asset));
                }
                return Err(Error::PageSelection {
                    value: format!("asset quality failed: {}", failures.join("; ")),
                });
            }
            asset = Some(rendered.asset.clone());
            hash = Some(rendered.hash.clone());
            pixel_width = Some(rendered.width);
            pixel_height = Some(rendered.height);
            effective_dpi = rendered.effective_dpi;
            quality = Some(rendered.quality);
            asset_kind = Some("crop".to_string());
        }
        "redraw" => {
            if input
                .asset
                .as_deref()
                .is_none_or(|value| value.trim().is_empty())
            {
                return Err(Error::PageSelection {
                    value: "redraw requires an existing diagram asset".to_string(),
                });
            }
            asset = input.asset.clone();
            asset_kind = Some("diagram".to_string());
        }
        _ => {}
    }

    let source_digest = std::fs::read(source_path)
        .ok()
        .map(|bytes| blake3::hash(&bytes).to_hex().to_string());
    let entry = VisualEntry {
        source: source_id.to_string(),
        page: input.page,
        decision: input.decision.clone(),
        reason: Some(input.reason.clone()),
        asset: asset.clone(),
        crop: crop.clone(),
        caption: input.caption.clone(),
        alt: input.alt.clone(),
        hash: hash.clone(),
        local_only: true,
        priority: input.priority.clone(),
        candidate_id: input.candidate_id.clone(),
        asset_kind,
        image_id,
        bbox: input.bbox.map(format_bbox),
        dpi: input.dpi,
        pixel_width,
        pixel_height,
        effective_dpi,
        relevance: candidate
            .as_ref()
            .map(|candidate| candidate.relevance.label.clone()),
        reconstruction: candidate
            .as_ref()
            .map(|candidate| candidate.reconstruction.kind.clone()),
        redraw_kind: input.redraw_kind.clone(),
        schematic_of: input.schematic_of.clone(),
        source_digest,
        renderer: Some(format!("pdfium {}", crate::pdfium::PDFIUM_VERSION)),
        quality,
        quality_exception: input.quality_exception,
        notes: input.notes.clone(),
        extra: BTreeMap::new(),
    };
    let audit_path = guide_dir.join("visual-audit.json");
    VisualAudit::upsert(&audit_path, entry)?;
    let width = input.width.unwrap_or(90).clamp(35, 100);
    let markdown = if matches!(input.decision.as_str(), "embed_crop" | "embed_image") {
        let asset = asset.clone().unwrap_or_default();
        let caption = input.caption.clone().unwrap_or_default();
        let alt = input.alt.clone().unwrap_or_default();
        format!(
            "::: figure src=\"{}\" caption=\"{}\" source=\"{}, s. {}\" alt=\"{}\" width=\"{}%\"\n:::\n",
            escape(&asset),
            escape(&caption),
            source_id,
            input.page,
            escape(&alt),
            width
        )
    } else if input.decision == "redraw" {
        let id = asset
            .as_deref()
            .and_then(|value| Path::new(value).file_stem())
            .and_then(|value| value.to_str())
            .unwrap_or("diagram");
        format!(
            "::: diagram id=\"{}\" width=\"{}%\"\n:::\n",
            escape(id),
            width
        )
    } else {
        String::new()
    };
    Ok(DecisionOutcome {
        guide: guide_slug.to_string(),
        candidate_id: input.candidate_id,
        decision: input.decision,
        asset,
        hash,
        markdown,
        audit: audit_path.display().to_string(),
        content_unchanged: true,
    })
}

pub async fn assets(
    guide_slug: &str,
    guide_dir: &Path,
    sources: &[(String, PathBuf)],
    action: &str,
) -> Result<AssetReport> {
    if !matches!(action, "check" | "materialize" | "refresh") {
        return Err(Error::PageSelection {
            value: format!("unknown assets action {action:?}; use check, materialize or refresh"),
        });
    }
    let audit_path = guide_dir.join("visual-audit.json");
    let mut audit = VisualAudit::load_or_default(&audit_path)?;
    let mut report = AssetReport {
        guide: guide_slug.to_string(),
        action: action.to_string(),
        checked: 0,
        ok: 0,
        missing: 0,
        mismatched: 0,
        refreshed: 0,
        issues: Vec::new(),
    };
    let mut changed = false;
    for entry in &mut audit.source_pages {
        if !matches!(
            entry.decision.as_str(),
            "embed_crop" | "embed_image" | "redraw"
        ) {
            continue;
        }
        report.checked += 1;
        let Some(asset) = entry.asset.as_deref() else {
            report.issues.push(format!(
                "{}:{}: missing asset path",
                entry.source, entry.page
            ));
            continue;
        };
        let asset_path = guide_dir.join(asset);
        let exists = asset_path.is_file();
        if entry.decision == "redraw" {
            if exists {
                report.ok += 1;
            } else {
                report.missing += 1;
                report.issues.push(format!(
                    "{}:{}: {asset} is missing",
                    entry.source, entry.page
                ));
            }
            continue;
        }
        let hash_ok = exists
            && entry.hash.as_deref().is_some_and(|expected| {
                std::fs::read(&asset_path)
                    .ok()
                    .is_some_and(|bytes| blake3::hash(&bytes).to_hex().to_string() == expected)
            });
        if exists && hash_ok && action != "refresh" {
            report.ok += 1;
            continue;
        }
        if action == "check" {
            if !exists {
                report.missing += 1;
                report.issues.push(format!(
                    "{}:{}: {asset} is missing",
                    entry.source, entry.page
                ));
            } else {
                report.mismatched += 1;
                report.issues.push(format!(
                    "{}:{}: {asset} hash mismatch",
                    entry.source, entry.page
                ));
            }
            continue;
        }
        let Some((_, source_path)) = sources.iter().find(|(source, _)| source == &entry.source)
        else {
            report.issues.push(format!(
                "{}:{}: source {} is not configured",
                entry.source, entry.page, entry.source
            ));
            continue;
        };
        let result = if entry.decision == "embed_image" {
            let image_id = entry.image_id.as_deref().unwrap_or_default();
            extract_native(
                source_path,
                guide_dir,
                entry.page as u32,
                image_id,
                Some(asset),
            )
            .await
            .map(|native| {
                (
                    native.hash,
                    native.width,
                    native.height,
                    native.effective_dpi,
                    native.quality,
                )
            })
        } else if entry.decision == "embed_crop" {
            let Some(crop) = entry.crop.as_deref() else {
                report
                    .issues
                    .push(format!("{}:{}: crop is missing", entry.source, entry.page));
                continue;
            };
            let Some(bbox) = entry.bbox.as_deref().and_then(parse_bbox) else {
                report.issues.push(format!(
                    "{}:{}: bbox is missing; cannot materialize",
                    entry.source, entry.page
                ));
                continue;
            };
            render_crop(
                source_path,
                guide_dir,
                entry.page as u32,
                crate::render::CropBox::parse(crop)?,
                bbox,
                entry.dpi.unwrap_or(300.0),
                Some(asset),
            )
            .await
            .map(|crop| {
                (
                    crop.hash,
                    crop.width,
                    crop.height,
                    crop.effective_dpi,
                    crop.quality,
                )
            })
        } else {
            continue;
        };
        match result {
            Ok((hash, width, height, dpi, quality)) => {
                entry.hash = Some(hash);
                entry.pixel_width = Some(width);
                entry.pixel_height = Some(height);
                entry.effective_dpi = dpi;
                entry.quality = Some(quality);
                if let Ok(bytes) = std::fs::read(source_path) {
                    entry.source_digest = Some(blake3::hash(&bytes).to_hex().to_string());
                }
                report.refreshed += 1;
                changed = true;
            }
            Err(error) => report
                .issues
                .push(format!("{}:{}: {error}", entry.source, entry.page)),
        }
    }
    if changed {
        audit.save(&audit_path)?;
    }
    Ok(report)
}

fn format_bbox(bbox: Rect) -> String {
    format!("{},{},{},{}", bbox.x, bbox.y, bbox.width, bbox.height)
}

fn parse_bbox(value: &str) -> Option<Rect> {
    let parts = value
        .split(',')
        .map(str::trim)
        .map(str::parse::<f32>)
        .collect::<std::result::Result<Vec<_>, _>>()
        .ok()?;
    (parts.len() == 4).then(|| Rect {
        x: parts[0],
        y: parts[1],
        width: parts[2],
        height: parts[3],
    })
}

fn escape(value: &str) -> String {
    value.replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::PageLayout;

    fn input(decision: &str) -> DecisionInput {
        DecisionInput {
            decision: decision.to_string(),
            reason: "reviewed".to_string(),
            caption: Some("Figure caption".to_string()),
            alt: Some("Figure takeaway".to_string()),
            ..DecisionInput::default()
        }
    }

    fn page() -> PageLayout {
        PageLayout {
            page: 1,
            page_label: None,
            width: 600.0,
            height: 800.0,
            content_bounds: None,
            blocks: Vec::new(),
        }
    }

    #[test]
    fn empty_embed_crop_names_every_route() {
        let errors = validate_decision(&input("embed_crop"), None, None);
        let text = errors.join("\n");
        assert!(text.contains("bbox"), "{text}");
        assert!(text.contains("crop"), "{text}");
        assert!(text.contains("candidate"), "{text}");
    }

    #[test]
    fn crop_alone_is_valid_with_a_stored_layout_page() {
        let mut decision = input("embed_crop");
        decision.crop = Some("0.1,0.1,0.5,0.4".to_string());
        assert!(
            validate_decision(&decision, None, None)
                .iter()
                .any(|error| error.contains("layout page"))
        );
        let page = page();
        let errors = validate_decision(&decision, None, Some(&page));
        assert!(errors.is_empty(), "{errors:?}");
    }

    #[test]
    fn embed_image_names_image_id_and_candidate() {
        let errors = validate_decision(&input("embed_image"), None, None);
        let text = errors.join("\n");
        assert!(text.contains("image_id"), "{text}");
        assert!(text.contains("candidate_id"), "{text}");
    }

    #[tokio::test]
    async fn failing_validation_leaves_the_audit_and_assets_untouched() {
        let root = tempfile::tempdir().expect("tempdir");
        let source = root.path().join("source.pdf");
        let mut decision = input("embed_crop");
        decision.caption = None;
        decision.alt = None;
        decision.reason.clear();
        let error = apply_decision("demo", root.path(), &source, "PD0", decision)
            .await
            .expect_err("validation");
        assert!(error.to_string().contains("no files were written"));
        assert!(!root.path().join("visual-audit.json").exists());
        assert!(!root.path().join("data").exists());
    }
}
