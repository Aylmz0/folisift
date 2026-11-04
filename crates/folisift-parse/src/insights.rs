//! Opt-in LiteParse depth report.
//!
//! Annotations, AcroForm fields, tagged structure, vector graphics, image
//! references and per-page complexity are expensive and deliberately excluded
//! from the normal inventory. This module exposes them as a separate,
//! read-only diagnostics surface.

use std::path::Path;

use liteparse::{LiteParse, LiteParseConfig, OutputFormat};
use serde_json::{Value, json};

use crate::error::{Error, Result};
use crate::pdfium;

pub async fn extract_insights(
    path: &Path,
    ocr: bool,
    target_pages: Option<&str>,
    continue_on_page_error: bool,
) -> Result<Value> {
    pdfium::init(None)?;
    let path_text = path.to_str().ok_or_else(|| Error::Parse {
        path: path.to_path_buf(),
        message: "source path is not valid UTF-8".to_string(),
    })?;
    let result = LiteParse::new(LiteParseConfig {
        ocr_enabled: ocr,
        target_pages: target_pages.map(ToString::to_string),
        continue_on_page_error,
        output_format: OutputFormat::Json,
        extract_annotations: true,
        extract_form_fields: true,
        extract_structure_tree: true,
        extract_vector_graphics: true,
        extract_text_metadata: true,
        include_complexity: true,
        quiet: true,
        max_pages: 10_000,
        ..Default::default()
    })
    .parse(path_text)
    .await
    .map_err(|error| Error::Parse {
        path: path.to_path_buf(),
        message: error.to_string(),
    })?;

    let mut annotations_total = 0usize;
    let mut form_fields_total = 0usize;
    let pages = result
        .pages
        .iter()
        .map(|page| {
            let annotations = page.annotations.as_deref().unwrap_or_default();
            let form_fields = page.form_fields.as_deref().unwrap_or_default();
            annotations_total += annotations.len();
            form_fields_total += form_fields.len();
            json!({
                "page": page.page_number,
                "label": page.page_label,
                "chars": page.text.chars().count(),
                "annotations": annotations,
                "form_fields": form_fields,
                "complexity": page.complexity,
                "structure_nodes": page.struct_nodes.len(),
                "image_refs": page.image_refs.len(),
                "figures": page.figures.len(),
                "vector_graphics": page.vector_graphics.as_ref().map_or(0, |graphics| graphics.shapes.len() + graphics.lines.len()),
            })
        })
        .collect::<Vec<_>>();

    let mut ocr_reasons = std::collections::BTreeMap::<String, usize>::new();
    let mut ocr_candidates = Vec::new();
    for page in &pages {
        let complexity = &page["complexity"];
        if complexity["needs_ocr"].as_bool() == Some(true) {
            let page_number = page["page"].as_u64().unwrap_or(0);
            let reasons = complexity["reasons"]
                .as_array()
                .map(|values| {
                    values
                        .iter()
                        .filter_map(|value| value.as_str().map(ToString::to_string))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            for reason in &reasons {
                *ocr_reasons.entry(reason.clone()).or_default() += 1;
            }
            ocr_candidates.push(json!({
                "page": page_number,
                "reasons": reasons,
                "needs_ocr": true,
            }));
        }
    }
    Ok(json!({
        "source": path.display().to_string(),
        "total_pages": result.total_pages,
        "parsed_pages": pages.len(),
        "annotations_total": annotations_total,
        "form_fields_total": form_fields_total,
        "form_type": result.form_type,
        "page_errors": result.page_errors,
        "pages": pages,
        "ocr_evidence": {
            "policy": if ocr { "explicit-on" } else { "off; auto is explicit opt-in" },
            "pages_ocr": if ocr { pages.len() } else { 0 },
            "pages_skipped": if ocr { 0 } else { pages.len() },
            "candidates": ocr_candidates,
            "reasons": ocr_reasons,
            "manual_review": true,
            "note": "OCR is advisory; text-layer pages are skipped unless explicitly enabled.",
        },
        "policy": {
            "read_only": true,
            "ocr": ocr,
            "target_pages": target_pages,
            "continue_on_page_error": continue_on_page_error,
            "writes_files": false,
            "note": "Annotations/forms/structure/vector/complexity are diagnostics, not coverage inputs."
        },
    }))
}
