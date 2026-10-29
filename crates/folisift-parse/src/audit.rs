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
pub struct UnderfilledPage {
    pub page: usize,
    pub chars: usize,
    pub median_chars: usize,
    pub ratio: f64,
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
    pub underfilled_pages: Vec<UnderfilledPage>,
    pub orphan_heading_pages: Vec<usize>,
    pub bookmark_mismatches: Vec<String>,
    pub image_pages: usize,
    pub image_overflow_pages: Vec<usize>,
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
    let mut orphan_heading_pages = Vec::new();
    let mut bookmark_mismatches = Vec::new();
    let mut image_pages = 0usize;
    let mut image_overflow_pages = Vec::new();
    let mut missing_glyphs = 0usize;
    let mut min_font_size = f64::MAX;
    let mut page_texts: Vec<String> = Vec::with_capacity(pages);

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

        let page_number = index + 1;
        let images = page.image_bounds(5.0, 1.0);
        if !images.is_empty() {
            image_pages += 1;
        }
        for image in &images {
            if image.x < -1.0
                || image.y < -1.0
                || image.x + image.width > width as f32 + 1.0
                || image.y + image.height > height as f32 + 1.0
            {
                image_overflow_pages.push(page_number);
                break;
            }
        }
        let mut chars = 0usize;
        let mut content_overflow = false;
        let mut page_text = String::new();
        let mut heading_lines: Vec<(f64, f64, String)> = Vec::new();
        let mut body_line_ys: Vec<f64> = Vec::new();
        let mut current_line: Option<(f64, f64, String, f64)> = None;
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
                page_text.push(' ');
                continue;
            }
            if let Some(value) = char::from_u32(code) {
                page_text.push(value);
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
                if in_body && (x1 < margin_x - 4.0 || x2 > width - margin_x + 4.0) {
                    content_overflow = true;
                    if std::env::var("FOLISIFT_AUDIT_DEBUG").is_ok() {
                        eprintln!(
                            "content box page {page_number}: x {x1:.1}..{x2:.1} y {y1:.1}..{y2:.1} width {width:.1} char {:?}",
                            char::from_u32(code)
                        );
                    }
                }
                if let Some(value) = char::from_u32(code) {
                    match &mut current_line {
                        Some((line_y, line_size, line_text, _)) if (y1 - *line_y).abs() <= 3.0 => {
                            line_text.push(value);
                            *line_size = line_size.max(size);
                        }
                        _ => {
                            if let Some((line_y, line_size, line_text, _)) = current_line.take() {
                                if line_size >= 10.5 && line_text.trim().chars().count() > 2 {
                                    heading_lines.push((
                                        line_y,
                                        line_size,
                                        line_text.trim().to_string(),
                                    ));
                                } else if line_size < 10.5 && line_y > 45.0 {
                                    body_line_ys.push(line_y);
                                }
                            }
                            current_line = Some((y1, size, value.to_string(), y1));
                        }
                    }
                }
            }
        }
        page_texts.push(compact(&page_text));

        let touched = left != f64::MAX;
        let overflow =
            touched && (left < -1.0 || top < -1.0 || right > width + 1.0 || bottom > height + 1.0);
        if let Some((line_y, line_size, text, _)) = current_line.take() {
            if line_size >= 10.5 && text.trim().chars().count() > 2 {
                heading_lines.push((line_y, line_size, text.trim().to_string()));
            } else if line_size < 10.5 && line_y > 45.0 {
                body_line_ys.push(line_y);
            }
        }
        if has_orphan_heading(&heading_lines, &body_line_ys) {
            orphan_heading_pages.push(index + 1);
        }
        if chars < 5 {
            empty_pages.push(index + 1);
        }
        if overflow {
            overflow_pages.push(index + 1);
        }
        if content_overflow {
            content_box_pages.push(index + 1);
        }
        page_reports.push(PageAudit {
            page: page_number,
            chars,
            empty: chars < 5,
            overflow,
            text_left: if touched { left } else { 0.0 },
            text_top: if touched { top } else { 0.0 },
            text_right: if touched { right } else { 0.0 },
            text_bottom: if touched { bottom } else { 0.0 },
        });
    }

    let underfilled_pages = detect_underfilled(&page_reports);
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
    if !orphan_heading_pages.is_empty() {
        issues.push(format!(
            "possible orphan headings at page bottom: {orphan_heading_pages:?}"
        ));
    }

    for entry in document.outline() {
        let Some(page_index) = entry.page_index else {
            bookmark_mismatches.push(format!(
                "bookmark {:?} has no page destination",
                entry.title
            ));
            continue;
        };
        let Some(text) = page_texts.get(page_index as usize) else {
            bookmark_mismatches.push(format!(
                "bookmark {:?} points to missing page {}",
                entry.title,
                page_index + 1
            ));
            continue;
        };
        let title = compact(&entry.title);
        if !title.is_empty() && !text.contains(&title) {
            bookmark_mismatches.push(format!(
                "bookmark {:?} does not appear on page {}",
                entry.title,
                page_index + 1
            ));
        }
    }
    if !bookmark_mismatches.is_empty() {
        let shown = bookmark_mismatches
            .iter()
            .take(5)
            .cloned()
            .collect::<Vec<_>>()
            .join("; ");
        issues.push(format!(
            "{} bookmark destination mismatch(es): {shown}",
            bookmark_mismatches.len()
        ));
    }
    if !image_overflow_pages.is_empty() {
        issues.push(format!(
            "image/figure outside the page: {image_overflow_pages:?}"
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
        underfilled_pages,
        orphan_heading_pages,
        bookmark_mismatches,
        image_pages,
        image_overflow_pages,
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

/// Flag pages whose body text is far below the normal body density.
///
/// The first three pages (cover/front matter/TOC) are excluded because their
/// low character count is intentional. The threshold is intentionally
/// conservative: a page must be below 35% of the median body page to be
/// reported. This is a review gate, not a claim that every short page is bad.
fn detect_underfilled(pages: &[PageAudit]) -> Vec<UnderfilledPage> {
    let mut counts = pages
        .iter()
        .filter(|page| page.chars >= 100)
        .map(|page| page.chars)
        .collect::<Vec<_>>();
    counts.sort_unstable();
    let Some(median) = counts.get(counts.len() / 2).copied() else {
        return Vec::new();
    };
    let threshold = (median as f64 * 0.35).max(100.0);
    pages
        .iter()
        .skip(3)
        .filter(|page| page.chars >= 5 && (page.chars as f64) < threshold)
        .map(|page| UnderfilledPage {
            page: page.page,
            chars: page.chars,
            median_chars: median,
            ratio: if median == 0 {
                0.0
            } else {
                page.chars as f64 / median as f64
            },
        })
        .collect()
}

/// Case- and spacing-insensitive form used for bookmark destination checks.
///
/// Headings in the paper layout are letter-spaced, so “D.1 Derleme” may extract
/// as `D.1 D E R L E M E`. Removing all whitespace makes both forms comparable.
/// Turkish dotted/dotless i is folded to a plain `i`.
fn compact(text: &str) -> String {
    text.chars()
        .filter(|character| !character.is_whitespace())
        .flat_map(|character| match character {
            'İ' | 'I' | 'ı' => "i".chars().collect::<Vec<_>>(),
            '\u{0307}' => Vec::new(),
            other => other.to_lowercase().collect::<Vec<_>>(),
        })
        .collect()
}

fn has_orphan_heading(heading_lines: &[(f64, f64, String)], body_line_ys: &[f64]) -> bool {
    heading_lines.iter().any(|(y, _, text)| {
        *y < 95.0
            && text.trim().chars().count() > 2
            && !body_line_ys
                .iter()
                .any(|body_y| *body_y < *y - 5.0 && *body_y > 45.0)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_a_heading_with_no_body_below_it() {
        let headings = vec![(80.0, 14.0, "D.1 Derleme".to_string())];
        assert!(has_orphan_heading(&headings, &[]));
        assert!(has_orphan_heading(&headings, &[90.0, 92.0]));
        assert!(!has_orphan_heading(&headings, &[50.0]));
    }

    #[test]
    fn ignores_a_large_line_near_the_top() {
        let headings = vec![(700.0, 14.0, "D.1 Derleme".to_string())];
        assert!(!has_orphan_heading(&headings, &[]));
    }

    #[test]
    fn compacts_letter_spaced_headings_and_turkish_i() {
        assert_eq!(compact("D.1 D E R L E M E"), compact("d.1derleme"));
        assert_eq!(compact("KISALTMALAR"), compact("Kısaltmalar"));
        assert_eq!(compact("İÇİNDEKİLER"), compact("içindekiler"));
    }

    #[test]
    fn detects_underfilled_body_pages_but_not_front_matter() {
        let page = |page: usize, chars: usize| PageAudit {
            page,
            chars,
            empty: chars < 5,
            overflow: false,
            text_left: 0.0,
            text_top: 0.0,
            text_right: 0.0,
            text_bottom: 0.0,
        };
        let pages = vec![
            page(1, 300),
            page(2, 1800),
            page(3, 1000),
            page(4, 3000),
            page(5, 2900),
            page(6, 300),
            page(7, 2800),
        ];
        let underfilled = detect_underfilled(&pages);
        assert_eq!(underfilled.len(), 1, "{underfilled:?}");
        assert_eq!(underfilled[0].page, 6);
        assert!(underfilled[0].ratio < 0.35);
    }
}
