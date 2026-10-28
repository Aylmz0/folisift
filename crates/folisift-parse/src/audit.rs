//! Deterministic PDF layout checks that do not need an LLM.

use std::path::Path;

use serde::Serialize;

use crate::error::{Error, Result};
use crate::pdfium;

#[derive(Debug, Clone, Serialize)]
pub struct PageAudit {
    pub page: usize,
    pub chars: usize,
    pub empty: bool,
    pub overflow: bool,
    pub text_left: f64,
    pub text_top: f64,
    pub text_right: f64,
    pub text_bottom: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct LayoutAudit {
    pub pdf: String,
    pub pages: usize,
    pub expected_pages: Option<usize>,
    pub page_count_ok: bool,
    pub empty_pages: Vec<usize>,
    pub overflow_pages: Vec<usize>,
    pub missing_glyphs: usize,
    pub passed: bool,
    pub issues: Vec<String>,
}

/// Audit a rendered PDF for empty pages, outside-page text and missing glyphs.
pub fn audit_pdf(path: &Path, expected_pages: Option<usize>) -> Result<LayoutAudit> {
    pdfium::init(None)?;
    let path_text = path.to_str().ok_or_else(|| Error::Parse {
        path: path.to_path_buf(),
        message: "PDF path is not valid UTF-8".to_string(),
    })?;
    let library = liteparse_pdfium::Library::try_init().map_err(|error| Error::Pdfium {
        message: error.to_string(),
    })?;
    let document = library
        .load_document(path_text, None)
        .map_err(|error| Error::Parse {
            path: path.to_path_buf(),
            message: error.to_string(),
        })?;

    let pages = document.page_count().max(0) as usize;
    let mut page_reports = Vec::with_capacity(pages);
    let mut empty_pages = Vec::new();
    let mut overflow_pages = Vec::new();
    let mut missing_glyphs = 0usize;

    for index in 0..pages {
        let page = document.page(index as i32).map_err(|error| Error::Parse {
            path: path.to_path_buf(),
            message: error.to_string(),
        })?;
        let width = f64::from(page.width());
        let height = f64::from(page.height());
        let text = page.text().map_err(|error| Error::Parse {
            path: path.to_path_buf(),
            message: error.to_string(),
        })?;

        let mut chars = 0usize;
        let (mut left, mut top, mut right, mut bottom) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
        for index in 0..text.char_count() {
            let Some(character) = text.char_at(index) else {
                continue;
            };
            let code = character.unicode();
            if code == 0 || code == u32::from('\u{FFFD}') {
                missing_glyphs += 1;
                continue;
            }
            if char::from_u32(code).is_some_and(char::is_whitespace) {
                continue;
            }
            chars += 1;
            if let Some(rect) = text.rect(index) {
                left = left.min(rect.left);
                top = top.min(rect.top);
                right = right.max(rect.right);
                bottom = bottom.max(rect.bottom);
            }
        }

        let touched = left != f64::MAX;
        let overflow =
            touched && (left < -1.0 || top < -1.0 || right > width + 1.0 || bottom > height + 1.0);
        if chars < 20 {
            empty_pages.push(index + 1);
        }
        if overflow {
            overflow_pages.push(index + 1);
        }
        page_reports.push(PageAudit {
            page: index + 1,
            chars,
            empty: chars < 20,
            overflow,
            text_left: if touched { left } else { 0.0 },
            text_top: if touched { top } else { 0.0 },
            text_right: if touched { right } else { 0.0 },
            text_bottom: if touched { bottom } else { 0.0 },
        });
    }

    let page_count_ok = expected_pages.is_none_or(|expected| expected == pages);
    let mut issues = Vec::new();
    if !page_count_ok {
        issues.push(format!(
            "page count is {pages}, expected {}",
            expected_pages.unwrap_or_default()
        ));
    }
    if !empty_pages.is_empty() {
        issues.push(format!("empty pages: {empty_pages:?}"));
    }
    if !overflow_pages.is_empty() {
        issues.push(format!("text outside page: {overflow_pages:?}"));
    }
    if missing_glyphs > 0 {
        issues.push(format!("missing glyphs: {missing_glyphs}"));
    }
    let passed = issues.is_empty();

    Ok(LayoutAudit {
        pdf: path.display().to_string(),
        pages,
        expected_pages,
        page_count_ok,
        empty_pages,
        overflow_pages,
        missing_glyphs,
        passed,
        issues,
    })
}
