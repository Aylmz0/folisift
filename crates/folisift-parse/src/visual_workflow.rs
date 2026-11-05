//! Host-facing decision and asset-maintenance helpers.
//!
//! These functions are shared by the CLI and MCP tools so a decision is
//! materialized and recorded the same way everywhere.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::error::{Error, Result};
use crate::layout::Rect;
use crate::visual_assets::{crop_from_bbox, extract_native, render_crop};
use crate::visual_audit::{VisualAudit, VisualEntry};
use crate::visual_candidates::collect_candidates;

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

pub async fn apply_decision(
    guide_slug: &str,
    guide_dir: &Path,
    source_path: &Path,
    source_id: &str,
    mut input: DecisionInput,
) -> Result<DecisionOutcome> {
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

    let mut asset = input.asset.clone();
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
            let bbox = input.bbox.ok_or_else(|| Error::PageSelection {
                value: "embed_crop requires bbox or candidate_id".to_string(),
            })?;
            let crop_box = if let Some(value) = crop.as_deref() {
                crate::render::CropBox::parse(value)?
            } else {
                let (page_width, page_height) = if let Some(candidate) = candidate.as_ref() {
                    (candidate.page_width, candidate.page_height)
                } else {
                    return Err(Error::PageSelection {
                        value: "embed_crop requires --crop when no candidate is supplied"
                            .to_string(),
                    });
                };
                crop_from_bbox(bbox, page_width, page_height, input.padding.unwrap_or(4.0))?
            };
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
            asset = input.asset.clone();
            asset_kind = Some("diagram".to_string());
        }
        _ => {}
    }

    if matches!(input.decision.as_str(), "embed_crop" | "embed_image") {
        if input.reason.trim().is_empty() {
            return Err(Error::PageSelection {
                value: format!("{} requires a reason", input.decision),
            });
        }
        if input
            .caption
            .as_deref()
            .is_none_or(|value| value.trim().is_empty())
        {
            return Err(Error::PageSelection {
                value: format!("{} requires a caption", input.decision),
            });
        }
        if input
            .alt
            .as_deref()
            .is_none_or(|value| value.trim().is_empty())
        {
            return Err(Error::PageSelection {
                value: format!("{} requires alt text", input.decision),
            });
        }
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
