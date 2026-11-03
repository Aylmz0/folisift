//! Deterministic PDF layout checks that do not need an LLM.

use std::collections::BTreeMap;
use std::path::Path;

use serde::Serialize;

use crate::error::{Error, Result};
use crate::pdfium;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PageRole {
    FrontMatter,
    Toc,
    ChapterOpener,
    FigurePage,
    TablePage,
    Body,
    IntentionalBlank,
}

impl PageRole {
    fn is_intentional(self) -> bool {
        !matches!(self, Self::Body)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PageRoleEntry {
    pub page: usize,
    pub role: PageRole,
    pub chars: usize,
    pub visual: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct PageAudit {
    pub page: usize,
    pub chars: usize,
    pub empty: bool,
    pub overflow: bool,
    pub visual: bool,
    pub role: PageRole,
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

#[derive(Debug, Clone, Copy)]
pub struct AuditOptions {
    pub margin_x_pt: f64,
    pub margin_top_pt: f64,
    pub margin_bottom_pt: f64,
    pub min_font_pt: f64,
    pub max_line_chars: usize,
}

impl Default for AuditOptions {
    fn default() -> Self {
        Self {
            margin_x_pt: 46.0,
            margin_top_pt: 52.0,
            margin_bottom_pt: 45.0,
            min_font_pt: 6.0,
            max_line_chars: 0,
        }
    }
}

impl AuditOptions {
    /// Derive the content box from a style pack's resolved tokens. The
    /// absolute 6 pt floor remains until the table/label migration lands.
    pub fn from_tokens(tokens: &serde_json::Value) -> Self {
        let mm = |key: &str, default: f64| {
            tokens
                .get("layout")
                .and_then(|value| value.get(key))
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(default)
                * 72.0
                / 25.4
        };
        let audit = tokens.get("audit");
        let min_font_pt = audit
            .and_then(|value| value.get("min_font_pt"))
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(6.0);
        let max_line_chars = audit
            .and_then(|value| value.get("max_line_chars"))
            .and_then(serde_json::Value::as_u64)
            .map_or(0, |value| value as usize);
        Self {
            margin_x_pt: mm("margin_side_mm", 16.2),
            margin_top_pt: mm("margin_top_mm", 18.3),
            margin_bottom_pt: mm("margin_bottom_mm", 15.9),
            min_font_pt,
            max_line_chars,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct FontSizeCount {
    pub size: f64,
    pub chars: usize,
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
    pub page_roles: Vec<PageRoleEntry>,
    pub figure_only_pages: Vec<usize>,
    pub table_only_pages: Vec<usize>,
    pub chapter_split_pages: Vec<usize>,
    pub orphan_heading_pages: Vec<usize>,
    pub bookmark_mismatches: Vec<String>,
    pub image_pages: usize,
    pub figure_captions: usize,
    pub figure_caption_pages: Vec<usize>,
    pub image_overflow_pages: Vec<usize>,
    pub missing_glyphs: usize,
    pub tagged: bool,
    pub bookmarks: usize,
    pub min_font_size: f64,
    pub body_font_size: f64,
    pub code_font_size: Option<f64>,
    pub code_body_ratio: Option<f64>,
    pub font_sizes: Vec<FontSizeCount>,
    pub line_chars_p50: usize,
    pub line_chars_p90: usize,
    pub long_line_pages: Vec<usize>,
    pub passed: bool,
    pub issues: Vec<String>,
}

/// Audit a rendered PDF for empty pages, outside-page text and missing glyphs.
pub fn audit_pdf(path: &Path, expected_pages: Option<usize>) -> Result<LayoutAudit> {
    audit_pdf_with(path, expected_pages, AuditOptions::default())
}

pub fn audit_pdf_with(
    path: &Path,
    expected_pages: Option<usize>,
    options: AuditOptions,
) -> Result<LayoutAudit> {
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
    let mut figure_captions = 0usize;
    let mut figure_caption_pages = Vec::new();
    let mut image_overflow_pages = Vec::new();
    let mut missing_glyphs = 0usize;
    let mut min_font_size = f64::MAX;
    let mut page_texts: Vec<String> = Vec::with_capacity(pages);

    // Body text is the font size with the most characters in the document.
    // This keeps heading/orphan detection style-aware: revision bodies are
    // 11 pt, accessible bodies 12 pt, paper bodies 9.5 pt.
    let mut font_counts: BTreeMap<u32, usize> = BTreeMap::new();
    let mut code_font_counts: BTreeMap<u32, usize> = BTreeMap::new();
    let mut line_lengths: Vec<usize> = Vec::new();
    let mut long_line_pages = Vec::new();
    for index in 0..pages {
        let page = document.page(index as i32).map_err(|error| Error::Parse {
            path: path.to_path_buf(),
            message: error.to_string(),
        })?;
        let text = page.text().map_err(|error| Error::Parse {
            path: path.to_path_buf(),
            message: error.to_string(),
        })?;
        for char_index in 0..text.char_count() {
            if let Some(character) = text.char_at(char_index) {
                let size = character.font_size();
                if size > 0.0 {
                    let key = (size * 10.0).round() as u32;
                    *font_counts.entry(key).or_default() += 1;
                    if character
                        .font_name()
                        .is_some_and(|name| name.to_lowercase().contains("mono"))
                    {
                        *code_font_counts.entry(key).or_default() += 1;
                    }
                }
            }
        }
    }
    let body_font_size = if font_counts.len() < 2 {
        // A single-size fixture is usually a heading-only seed; use the
        // paper body default so a large bottom line is still checked.
        9.5
    } else {
        font_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(size, _)| f64::from(*size) / 10.0)
            .unwrap_or(9.5)
    };

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
        let mut page_line_lengths: Vec<usize> = Vec::new();
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
                let in_body = y1 > options.margin_top_pt && y2 < height - options.margin_bottom_pt;
                let margin_x = options.margin_x_pt;
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
                                if (line_size - body_font_size).abs() <= 0.6 {
                                    page_line_lengths.push(line_text.trim().chars().count());
                                }
                                let in_body_band = line_y > options.margin_top_pt
                                    && line_y < height - options.margin_bottom_pt;
                                if in_body_band
                                    && line_size > body_font_size + 1.0
                                    && line_text.trim().chars().count() > 2
                                {
                                    heading_lines.push((
                                        line_y,
                                        line_size,
                                        line_text.trim().to_string(),
                                    ));
                                } else if in_body_band {
                                    body_line_ys.push(line_y);
                                }
                            }
                            current_line = Some((y1, size, value.to_string(), y1));
                        }
                    }
                }
            }
        }
        let captions = count_figure_captions(&page_text);
        if captions > 0 {
            figure_captions += captions;
            figure_caption_pages.push(page_number);
        }
        page_texts.push(compact(&page_text));

        let touched = left != f64::MAX;
        let overflow =
            touched && (left < -1.0 || top < -1.0 || right > width + 1.0 || bottom > height + 1.0);
        if let Some((line_y, line_size, text, _)) = current_line.take() {
            if (line_size - body_font_size).abs() <= 0.6 {
                page_line_lengths.push(text.trim().chars().count());
            }
            let in_body_band =
                line_y > options.margin_top_pt && line_y < height - options.margin_bottom_pt;
            if in_body_band && line_size > body_font_size + 1.0 && text.trim().chars().count() > 2 {
                heading_lines.push((line_y, line_size, text.trim().to_string()));
            } else if in_body_band {
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
        let role = classify_page(
            page_number,
            chars,
            !images.is_empty() || captions > 0,
            &page_text,
            heading_lines
                .iter()
                .map(|(_, size, _)| *size)
                .fold(0.0, f64::max),
            body_font_size,
        );
        page_reports.push(PageAudit {
            page: page_number,
            chars,
            empty: chars < 5,
            overflow,
            visual: !images.is_empty() || captions > 0,
            role,
            text_left: if touched { left } else { 0.0 },
            text_top: if touched { top } else { 0.0 },
            text_right: if touched { right } else { 0.0 },
            text_bottom: if touched { bottom } else { 0.0 },
        });
        if page_line_lengths
            .iter()
            .any(|count| options.max_line_chars > 0 && *count > options.max_line_chars)
        {
            long_line_pages.push(page_number);
        }
        line_lengths.extend(page_line_lengths);
    }

    let underfilled_pages = detect_underfilled(&page_reports);
    let page_roles = page_reports
        .iter()
        .map(|page| PageRoleEntry {
            page: page.page,
            role: page.role,
            chars: page.chars,
            visual: page.visual,
        })
        .collect::<Vec<_>>();
    let figure_only_pages = page_roles
        .iter()
        .filter(|entry| entry.role == PageRole::FigurePage)
        .map(|entry| entry.page)
        .collect::<Vec<_>>();
    let table_only_pages = page_roles
        .iter()
        .filter(|entry| entry.role == PageRole::TablePage)
        .map(|entry| entry.page)
        .collect::<Vec<_>>();
    let chapter_split_pages = page_reports
        .windows(2)
        .filter(|pair| {
            pair[0].role == PageRole::ChapterOpener && pair[1].role == PageRole::ChapterOpener
        })
        .map(|pair| pair[1].page)
        .collect::<Vec<_>>();
    let page_count_ok = expected_pages.is_none_or(|expected| expected == pages);
    let mut sorted_lines = line_lengths;
    sorted_lines.sort_unstable();
    let line_chars_p50 = percentile(&sorted_lines, 0.50);
    let line_chars_p90 = percentile(&sorted_lines, 0.90);
    let font_sizes = font_counts
        .iter()
        .map(|(size, chars)| FontSizeCount {
            size: f64::from(*size) / 10.0,
            chars: *chars,
        })
        .collect::<Vec<_>>();
    let code_font_size = code_font_counts
        .iter()
        .max_by_key(|(_, chars)| *chars)
        .map(|(size, _)| f64::from(*size) / 10.0);
    let code_body_ratio = code_font_size.map(|size| size / body_font_size);
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
    if min_font_size != f64::MAX && min_font_size < options.min_font_pt {
        issues.push(format!(
            "text below {:.1} pt: {min_font_size:.1} pt",
            options.min_font_pt
        ));
    }
    if !long_line_pages.is_empty() {
        issues.push(format!(
            "lines above {} characters: {long_line_pages:?}",
            options.max_line_chars
        ));
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
        page_roles,
        figure_only_pages,
        table_only_pages,
        chapter_split_pages,
        orphan_heading_pages,
        bookmark_mismatches,
        image_pages,
        figure_captions,
        figure_caption_pages,
        image_overflow_pages,
        missing_glyphs,
        tagged,
        bookmarks,
        min_font_size: if min_font_size == f64::MAX {
            0.0
        } else {
            min_font_size
        },
        body_font_size,
        code_font_size,
        code_body_ratio,
        font_sizes,
        line_chars_p50,
        line_chars_p90,
        long_line_pages,
        passed,
        issues,
    })
}

fn percentile(sorted: &[usize], fraction: f64) -> usize {
    if sorted.is_empty() {
        return 0;
    }
    let index = ((sorted.len() - 1) as f64 * fraction).round() as usize;
    sorted[index.min(sorted.len() - 1)]
}

static FIGURE_CAPTION_RE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r#"(?i)(şekil|figure)[”"\x27]?\s*\d+"#).expect("valid figure caption regex")
});

fn count_figure_captions(page_text: &str) -> usize {
    FIGURE_CAPTION_RE.find_iter(page_text).count()
}

/// Flag pages whose body text is far below the normal body density.
///
/// The first three pages (cover/front matter/TOC) are excluded because their
/// low character count is intentional. The threshold is intentionally
/// conservative: a page must be below 35% of the median body page to be
/// reported. This is a review gate, not a claim that every short page is bad.
fn classify_page(
    page: usize,
    chars: usize,
    visual: bool,
    text: &str,
    max_heading_size: f64,
    body_font_size: f64,
) -> PageRole {
    if page == 1 {
        return PageRole::FrontMatter;
    }
    let lower = text.to_lowercase().replace('\u{0307}', "");
    let head = lower.chars().take(240).collect::<String>();
    let compact_head = head.split_whitespace().collect::<String>();
    if page <= 6
        && chars >= 80
        && (compact_head.contains("içindekiler") || compact_head.contains("contents"))
        && (lower.contains("....") || lower.matches(char::is_numeric).count() > 7)
    {
        return PageRole::Toc;
    }
    if chars < 5 {
        return PageRole::Body;
    }
    if visual && chars < 250 {
        return PageRole::FigurePage;
    }
    if chars < 1500 && max_heading_size >= body_font_size * 1.9 {
        return PageRole::ChapterOpener;
    }
    if lower.contains("tablo") && chars < 500 {
        return PageRole::TablePage;
    }
    PageRole::Body
}

fn detect_underfilled(pages: &[PageAudit]) -> Vec<UnderfilledPage> {
    let mut counts = pages
        .iter()
        .filter(|page| !page.visual && page.chars >= 100)
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
        .filter(|page| {
            !page.role.is_intentional() && page.chars >= 5 && (page.chars as f64) < threshold
        })
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
                .any(|body_y| *body_y < *y - 15.0 && *body_y > 45.0)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_style_aware_audit_margins() {
        let tokens = serde_json::json!({
            "layout": {
                "margin_top_mm": 18,
                "margin_bottom_mm": 16,
                "margin_side_mm": 32
            },
            "audit": {"min_font_pt": 7.0}
        });
        let options = AuditOptions::from_tokens(&tokens);
        assert!((options.margin_x_pt - 90.7).abs() < 0.2);
        assert!((options.margin_top_pt - 51.0).abs() < 0.2);
        assert!((options.margin_bottom_pt - 45.4).abs() < 0.2);
        assert!((options.min_font_pt - 7.0).abs() < 0.01);
    }

    #[test]
    fn detects_a_heading_with_no_body_below_it() {
        let headings = vec![(80.0, 14.0, "D.1 Derleme".to_string())];
        assert!(has_orphan_heading(&headings, &[]));
        assert!(has_orphan_heading(&headings, &[90.0]));
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
    fn counts_figure_captions() {
        assert_eq!(count_figure_captions("“Şekil” 1 Açıklama"), 1);
        assert_eq!(count_figure_captions("Figure 2.1 Example and Figure 3"), 2);
        assert_eq!(count_figure_captions("no caption"), 0);
    }

    #[test]
    fn detects_underfilled_body_pages_but_not_front_matter() {
        let page = |page: usize, chars: usize| PageAudit {
            page,
            chars,
            empty: chars < 5,
            overflow: false,
            visual: false,
            role: PageRole::Body,
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

    #[test]
    fn page_roles_explain_toc_and_figure_pages() {
        assert_eq!(
            classify_page(1, 300, false, "Kapak", 0.0, 9.5),
            PageRole::FrontMatter
        );
        assert_eq!(
            classify_page(2, 900, false, "İÇİNDEKİLER 1 .... 2 .... 3 ....", 0.0, 9.5),
            PageRole::Toc
        );
        assert_eq!(
            classify_page(3, 200, true, "Şekil 1", 0.0, 9.5),
            PageRole::FigurePage
        );
        assert_eq!(
            classify_page(4, 500, false, "BÖLÜM 1 Giriş", 24.0, 10.0),
            PageRole::ChapterOpener
        );
        assert_eq!(
            classify_page(5, 3000, false, "gövde", 10.0, 10.0),
            PageRole::Body
        );
    }

    #[test]
    fn visual_pages_are_not_underfilled() {
        let page = |page: usize, chars: usize, visual: bool| PageAudit {
            page,
            chars,
            empty: chars < 5,
            overflow: false,
            visual,
            role: if visual {
                PageRole::FigurePage
            } else {
                PageRole::Body
            },
            text_left: 0.0,
            text_top: 0.0,
            text_right: 0.0,
            text_bottom: 0.0,
        };
        let pages = vec![
            page(1, 300, false),
            page(2, 1800, false),
            page(3, 1000, false),
            page(4, 3000, false),
            page(5, 2900, false),
            page(6, 200, true),
            page(7, 2800, false),
        ];
        assert!(detect_underfilled(&pages).is_empty());
    }
}
