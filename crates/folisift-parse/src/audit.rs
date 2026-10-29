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
    pub content_box_pages: Vec<usize>,
    pub missing_glyphs: usize,
    pub tagged: bool,
    pub bookmarks: usize,
    pub min_font_size: f64,
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
    let tagged = document.is_tagged();
    let bookmarks = document.outline().len();
    let mut page_reports = Vec::with_capacity(pages);
    let mut empty_pages = Vec::new();
    let mut overflow_pages = Vec::new();
    let mut content_box_pages = Vec::new();
    let mut missing_glyphs = 0usize;
    let mut min_font_size = f64::MAX;

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
        let mut content_overflow = false;
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
            let size = character.font_size();
            if size > 0.0 {
                min_font_size = min_font_size.min(size);
            }
            let rect = text
                .rect(index)
                .map(|rect| (rect.left, rect.top, rect.right, rect.bottom))
                .or_else(|| {
                    character
                        .char_box()
                        .map(|bbox| (bbox.left, bbox.top, bbox.right, bbox.bottom))
                })
                .or_else(|| {
                    character.origin().map(|(x, y)| {
                        let size = character.font_size();
                        (x, y, x + size * 0.6, y + size)
                    })
                });
            if let Some((x1, y1, x2, y2)) = rect {
                left = left.min(x1);
                top = top.min(y1);
                right = right.max(x2);
                bottom = bottom.max(y2);
                // Ignore header/footer bands; check the body against the
                // paper margins (16.5 mm sides, 16 mm top, 14 mm bottom).
                let in_body = y1 > 52.0 && y2 < height - 45.0;
                let margin_x = 46.0;
                if in_body && (x1 < margin_x - 1.0 || x2 > width - margin_x + 1.0) {
                    content_overflow = true;
                }
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
        if content_overflow {
            content_box_pages.push(index + 1);
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
    if !content_box_pages.is_empty() {
        issues.push(format!(
            "text outside the content box: {content_box_pages:?}"
        ));
    }
    if missing_glyphs > 0 {
        issues.push(format!("missing glyphs: {missing_glyphs}"));
    }
    if !tagged {
        issues.push("PDF is not tagged".to_string());
    }
    if bookmarks == 0 {
        issues.push("PDF has no bookmarks".to_string());
    }
    if min_font_size != f64::MAX && min_font_size < 6.0 {
        issues.push(format!("text below 6 pt: {min_font_size:.1} pt"));
    }
    let passed = issues.is_empty();

    Ok(LayoutAudit {
        pdf: path.display().to_string(),
        pages,
        expected_pages,
        page_count_ok,
        empty_pages,
        overflow_pages,
        content_box_pages,
        missing_glyphs,
        tagged,
        bookmarks,
        min_font_size: if min_font_size == f64::MAX {
            0.0
        } else {
            min_font_size
        },
        passed,
        issues,
    })
}
