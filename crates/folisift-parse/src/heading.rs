//! Shared heading evidence and section-number filters.
//!
//! The inventory, the coverage manifest, the outline builder and the brief
//! resolver must agree on what counts as a section heading. This module owns
//! that rule once: a number must look like a section number, a title must look
//! like a title, and an inline heading must be recoverable from paragraph,
//! list or table-cell text.

use std::sync::LazyLock;

use regex::Regex;

use crate::inventory::Heading;
use crate::layout::{LayoutBlock, LayoutBlockKind, PageLayout};

/// Maximum accepted section depth (`1.2.3.4`).
pub const MAX_SECTION_DEPTH: usize = 4;

/// A parsed numbered heading such as `4.4.4 Asıl İşveren - Alt İşveren`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumberedHeading {
    pub number: String,
    pub title: String,
    pub heading: String,
}

static NUMBER_PREFIX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(\d+(?:\.\d+){0,4})\.?\s*(.*)$").expect("valid heading prefix regex")
});

static INLINE_HEADING_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:^|[.!?]\s+)(\d+(?:\.\d+){1,})\.?\s+([A-ZÇĞİÖŞÜ][^.;!?]{2,120})")
        .expect("valid inline heading regex")
});

/// Parse a heading that starts with a section number and a title.
pub fn parse_numbered_heading(value: &str) -> Option<NumberedHeading> {
    let value = value.trim();
    let captures = NUMBER_PREFIX.captures(value)?;
    let number = captures.get(1)?.as_str().trim_end_matches('.').to_string();
    if !is_section_number(&number) {
        return None;
    }
    let title = clean_heading_title(captures.get(2).map_or("", |value| value.as_str()));
    if !title_like(&title) {
        return None;
    }
    Some(NumberedHeading {
        heading: format!("{number} {title}"),
        number,
        title,
    })
}

/// True when the value is a plausible section number.
///
/// Components must be positive integers. `1.0`, `2.00`, `10.500`, `300.000`
/// and dates are distances, quantities or list markers, not sections.
pub fn is_section_number(value: &str) -> bool {
    let value = value.trim().trim_end_matches('.');
    if value.is_empty() {
        return false;
    }
    let parts = value.split('.').collect::<Vec<_>>();
    if parts.len() > MAX_SECTION_DEPTH {
        return false;
    }
    if parts
        .iter()
        .any(|part| part.is_empty() || !part.chars().all(|character| character.is_ascii_digit()))
    {
        return false;
    }
    let Some(numbers) = parts
        .iter()
        .map(|part| part.parse::<u32>().ok())
        .collect::<Option<Vec<_>>>()
    else {
        return false;
    };
    if numbers.contains(&0) {
        return false;
    }
    // A bare section above 99 is a page/quantity unless the source explicitly
    // says "Bölüm"/"Chapter". The caller may still pass such a heading through
    // `parse_numbered_heading` only when the title carries that prefix.
    if numbers[0] > 99 {
        return false;
    }
    if numbers.iter().skip(1).any(|number| *number > 99) {
        return false;
    }
    !is_date_like(value)
}

/// True when a title looks like a heading rather than a chart label, table
/// row or wrapped body sentence.
pub fn title_like(value: &str) -> bool {
    let value = value.trim();
    let words = value.split_whitespace().count();
    let first = value.split_whitespace().next().unwrap_or_default();
    let first_lower = first.trim_end_matches([':', ',', ';', '.']).to_lowercase();
    let starts_with_punctuation = matches!(
        value.chars().next(),
        Some(')' | ']' | '-' | '–' | '—' | '*' | '%' | ',')
    );
    // Allow `e-Devlet` and `iPhone`; reject `x azaldı` and lowercase prose.
    let lowercase_word = first
        .chars()
        .next()
        .is_some_and(|character| character.is_lowercase())
        && !first.chars().any(|character| character.is_uppercase());
    !value.is_empty()
        && value.chars().count() <= 100
        && (1..=18).contains(&words)
        && !starts_with_punctuation
        && !lowercase_word
        && !value.ends_with("...")
        && !value.ends_with(',')
        && !value.ends_with(';')
        && !(value.contains(',') && words > 8)
        && !matches!(
            first_lower.as_str(),
            "bu" | "bir"
                | "amaç"
                | "örnek"
                | "şekil"
                | "tablo"
                | "kaynak"
                | "not"
                | "ve"
                | "veya"
                | "ile"
                | "için"
                | "daha"
                | "gibi"
                | "adım"
                | "aşama"
                | "step"
        )
}

/// A heading ends at the first sentence boundary. Colon and semicolon are
/// valid inside titles; only a period followed by whitespace ends one.
pub fn clean_heading_title(value: &str) -> String {
    let mut value = value.trim();
    if let Some(index) = value.find(". ") {
        value = &value[..index];
    }
    let value = value
        .trim()
        .trim_end_matches(['.', ',', ';', ':', '·', '•'])
        .trim();
    let mut words = value.split_whitespace().collect::<Vec<_>>();
    if words.len() > 18 {
        words.truncate(18);
    }
    let mut title = words.join(" ");
    if title.chars().count() > 100 {
        title = title.chars().take(100).collect::<String>();
        if let Some(index) = title.rfind(' ') {
            title.truncate(index);
        }
    }
    title
}

/// A date encoded as `d.m.yyyy`.
pub fn is_date_like(number: &str) -> bool {
    let parts = number
        .split('.')
        .map(|part| part.parse::<u32>())
        .collect::<std::result::Result<Vec<_>, _>>();
    matches!(parts, Ok(parts) if parts.len() == 3
        && (1..=31).contains(&parts[0])
        && (1..=12).contains(&parts[1])
        && (1900..=2100).contains(&parts[2]))
}

/// Normalize a heading block from the layout classifier.
pub fn normalize_heading(heading: &str) -> String {
    let heading = heading.trim();
    parse_numbered_heading(heading).map_or_else(|| heading.to_string(), |parsed| parsed.heading)
}

/// A numbered heading recovered from running text, with its byte offset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineHeadingMatch {
    pub start: usize,
    pub heading: NumberedHeading,
}

/// Find every numbered heading embedded in a paragraph, list item or table
/// cell text. `start` is the byte offset of the section number.
pub fn inline_heading_matches(text: &str) -> Vec<InlineHeadingMatch> {
    let mut out = Vec::new();
    for capture in INLINE_HEADING_RE.captures_iter(text) {
        let Some(number) = capture.get(1).map(|value| value.as_str()) else {
            continue;
        };
        if !is_section_number(number) {
            continue;
        }
        let title = capture
            .get(2)
            .map(|value| value.as_str().trim().trim_end_matches(['-', '–', ':']))
            .unwrap_or_default();
        let title = clean_heading_title(title);
        if !title_like(&title) {
            continue;
        }
        let Some(start) = capture.get(1).map(|value| value.start()) else {
            continue;
        };
        let number = number.trim_end_matches('.').to_string();
        out.push(InlineHeadingMatch {
            start,
            heading: NumberedHeading {
                heading: format!("{number} {title}"),
                number,
                title,
            },
        });
    }
    out
}

/// Recover inline headings from paragraph, list and unknown blocks.
///
/// This is the inventory's existing evidence path. Table cells are recovered
/// separately by [`scan_table_headings`] so the brief resolver can attach the
/// whole table to the heading.
pub fn recover_inline_headings(pages: &[PageLayout]) -> Vec<Heading> {
    let mut out = Vec::new();
    for page in pages {
        for block in &page.blocks {
            if !matches!(
                block.kind,
                LayoutBlockKind::Paragraph | LayoutBlockKind::ListItem | LayoutBlockKind::Unknown
            ) {
                continue;
            }
            let Some(text) = block.text.as_deref() else {
                continue;
            };
            for matched in inline_heading_matches(text) {
                out.push(Heading {
                    heading: matched.heading.heading,
                    page: page.page,
                    level: Some(2),
                    block_id: Some(block.id.clone()),
                });
            }
        }
    }
    out
}

/// Recover headings that appear at the start of a table cell.
pub fn scan_table_headings(pages: &[PageLayout]) -> Vec<Heading> {
    let mut out = Vec::new();
    for page in pages {
        for block in &page.blocks {
            if !block.kind.is_table() {
                continue;
            }
            for cell in block.header.iter().chain(block.rows.iter().flatten()) {
                let Some(parsed) = parse_numbered_heading(&cell.text) else {
                    continue;
                };
                out.push(Heading {
                    heading: parsed.heading,
                    page: page.page,
                    level: Some(2),
                    block_id: Some(block.id.clone()),
                });
            }
        }
    }
    out
}

/// Iterate every text surface that can carry a numbered heading.
pub fn block_text_surfaces(block: &LayoutBlock) -> Vec<&str> {
    let mut out = Vec::new();
    if let Some(text) = block.text.as_deref() {
        out.push(text);
    }
    for cell in &block.header {
        out.push(&cell.text);
    }
    for row in &block.rows {
        for cell in row {
            out.push(&cell.text);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_real_section_numbers() {
        for number in ["1", "1.1", "1.1.1", "4.4.4", "10.2", "2.10"] {
            assert!(is_section_number(number), "{number} should be a section");
        }
    }

    #[test]
    fn rejects_list_markers_quantities_and_dates() {
        for number in [
            "1.0",
            "2.00",
            "3.000",
            "10.500",
            "12.500",
            "40.000",
            "300.000",
            "500.000",
            "1.2.3.4.5",
            "2024.10.07",
        ] {
            assert!(
                !is_section_number(number),
                "{number} should not be a section"
            );
        }
    }

    #[test]
    fn rejects_table_and_chart_noise() {
        assert!(parse_numbered_heading("1 ) Birim Satış Fiyatı; Satılan Miktar").is_none());
        assert!(parse_numbered_heading("40 x azaldı* Dönüşümü Robot Maliyeti").is_none());
        assert!(parse_numbered_heading("10.500 m3 olabilmektedir").is_none());
        assert!(parse_numbered_heading("4. Adım: Mevcut süreçlerin incelenmesi").is_none());
    }

    #[test]
    fn keeps_table_cell_headings() {
        let parsed = parse_numbered_heading(
            "10.2. UlusalBayram ve GenelTatiller Ulusal Bayram ve Genel Tatiller Hakkında Kanuna",
        )
        .expect("table heading");
        assert_eq!(parsed.number, "10.2");
        assert!(parsed.title.starts_with("UlusalBayram"));
    }
}
