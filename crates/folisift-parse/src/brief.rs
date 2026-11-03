//! Section-oriented retrieval for agent context.
//!
//! Briefs are extractive and deterministic: no model call, no network. They
//! turn stored layout blocks into a small, page-anchored answer before the host
//! asks for the full section.

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::layout::{LayoutBlock, LayoutBlockKind, SourceLayout};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SectionBrief {
    pub source: String,
    pub heading: String,
    pub level: String,
    pub page_start: usize,
    pub page_end: usize,
    pub summary: String,
    pub numbers: Vec<String>,
    pub table_rows: Vec<String>,
    pub citations: Vec<String>,
    pub chars: usize,
    pub estimated_tokens: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SectionRead {
    pub source: String,
    pub heading: String,
    pub level: String,
    pub mode: String,
    pub page_start: usize,
    pub page_end: usize,
    pub text: String,
    pub chars: usize,
    pub estimated_tokens: usize,
    pub truncated: bool,
    pub next_offset: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceSection {
    pub heading: String,
    pub page_start: usize,
    pub page_end: usize,
    pub text: String,
    pub table_rows: Vec<String>,
}

pub fn source_sections(layout: &SourceLayout) -> Vec<SourceSection> {
    let mut sections = Vec::new();
    let mut current: Option<SourceSection> = None;
    for page in &layout.pages {
        for block in &page.blocks {
            if block.kind == LayoutBlockKind::Heading {
                let heading = block.text_or_empty().trim();
                if heading.is_empty() {
                    continue;
                }
                if let Some(section) = current.take()
                    && !section.text.trim().is_empty()
                {
                    sections.push(section);
                }
                current = Some(SourceSection {
                    heading: heading.to_string(),
                    page_start: page.page,
                    page_end: page.page,
                    text: heading.to_string(),
                    table_rows: Vec::new(),
                });
                continue;
            }
            let text = block_text(block);
            if text.is_empty() {
                continue;
            }
            let section = current.get_or_insert_with(|| SourceSection {
                heading: String::new(),
                page_start: page.page,
                page_end: page.page,
                text: String::new(),
                table_rows: Vec::new(),
            });
            if !section.text.is_empty() {
                section.text.push('\n');
            }
            section.text.push_str(&text);
            section.page_end = page.page;
            if block.kind.is_table() {
                section.table_rows.extend(table_rows(block));
            }
        }
    }
    if let Some(section) = current
        && !section.text.trim().is_empty()
    {
        sections.push(section);
    }
    sections
}

pub fn find_section<'a>(sections: &'a [SourceSection], query: &str) -> Option<&'a SourceSection> {
    let query = normalize(query);
    sections
        .iter()
        .find(|section| normalize(&section.heading).contains(&query))
}

pub fn section_brief(layout: &SourceLayout, query: &str, max_chars: usize) -> Option<SectionBrief> {
    let sections = source_sections(layout);
    let section = find_section(&sections, query)?;
    let max_chars = max_chars.clamp(200, 4000);
    let text = section.text.trim();
    let summary = summary(text, max_chars);
    Some(SectionBrief {
        source: layout.source.clone(),
        heading: section.heading.clone(),
        level: section_level(&section.heading).to_string(),
        page_start: section.page_start,
        page_end: section.page_end,
        summary,
        numbers: numbers(text),
        table_rows: section.table_rows.iter().take(8).cloned().collect(),
        citations: citations(text),
        chars: text.chars().count(),
        estimated_tokens: estimate_tokens(text),
    })
}

pub fn read_section(
    layout: &SourceLayout,
    query: &str,
    max_chars: usize,
    offset: usize,
) -> Option<SectionRead> {
    read_section_mode(layout, query, max_chars, offset, "full")
}

pub fn read_section_mode(
    layout: &SourceLayout,
    query: &str,
    max_chars: usize,
    offset: usize,
    mode: &str,
) -> Option<SectionRead> {
    let sections = source_sections(layout);
    let section = find_section(&sections, query)?;
    let full = section.text.trim();
    let chars = full.chars().count();
    let max_chars = max_chars.clamp(20, 24000);
    let start = offset.min(chars);
    let text = match mode {
        "summary" => summary(full, max_chars),
        "claims" => claim_lines(full),
        "tables" => table_text(section),
        _ => full.chars().skip(start).take(max_chars).collect::<String>(),
    };
    let truncated = match mode {
        "full" => start + text.chars().count() < chars,
        _ => text.chars().count() > max_chars,
    };
    let text = text.chars().take(max_chars).collect::<String>();
    Some(SectionRead {
        source: layout.source.clone(),
        heading: section.heading.clone(),
        level: section_level(&section.heading).to_string(),
        mode: mode.to_string(),
        page_start: section.page_start,
        page_end: section.page_end,
        text,
        chars,
        estimated_tokens: estimate_tokens(full),
        truncated,
        next_offset: truncated.then_some(start + max_chars),
    })
}

fn section_level(heading: &str) -> &'static str {
    let number = heading.split_whitespace().next().unwrap_or_default();
    match number.matches('.').count() {
        0 => "chapter",
        1 => "section",
        _ => "heading",
    }
}

fn claim_lines(text: &str) -> String {
    let kept = text
        .split_inclusive(['.', '!', '?'])
        .map(str::trim)
        .filter(|sentence| {
            sentence.chars().count() >= 20
                && (sentence.contains('[')
                    || sentence.chars().any(|character| character.is_ascii_digit())
                    || sentence.to_lowercase().contains("artar")
                    || sentence.to_lowercase().contains("azalır"))
        })
        .collect::<Vec<_>>();
    if kept.is_empty() {
        text.to_string()
    } else {
        kept.join("\n")
    }
}

fn table_text(section: &SourceSection) -> String {
    if section.table_rows.is_empty() {
        section
            .text
            .lines()
            .filter(|line| line.contains('|'))
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        section.table_rows.join("\n")
    }
}

fn summary(text: &str, max_chars: usize) -> String {
    let sentences = text
        .split_inclusive(['.', '!', '?'])
        .map(str::trim)
        .filter(|sentence| !sentence.is_empty())
        .take(4)
        .collect::<Vec<_>>()
        .join(" ");
    if sentences.chars().count() >= 80 {
        sentences.chars().take(max_chars).collect()
    } else {
        text.chars().take(max_chars).collect()
    }
}

fn numbers(text: &str) -> Vec<String> {
    static NUMBER_RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
        Regex::new(r"\b(?:\d{1,4}[.,]\d+|\d{1,4}|[A-Za-zÇĞİÖŞÜçğıöşü]{3,}\s+\d{4})\b")
            .expect("valid number regex")
    });
    let mut seen = std::collections::BTreeSet::new();
    NUMBER_RE
        .find_iter(text)
        .filter_map(|hit| {
            let value = hit.as_str().to_string();
            seen.insert(value.clone()).then_some(value)
        })
        .take(12)
        .collect()
}

fn citations(text: &str) -> Vec<String> {
    static CITE_RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
        Regex::new(r"\[[A-Za-z0-9_.-]+,\s*(?:s|p)\.\s*\d+(?:\s*[–-]\s*\d+)?\]")
            .expect("valid citation regex")
    });
    CITE_RE
        .find_iter(text)
        .map(|hit| hit.as_str().to_string())
        .take(12)
        .collect()
}

fn block_text(block: &LayoutBlock) -> String {
    let text = block.text_or_empty().trim();
    if !text.is_empty() {
        return text.to_string();
    }
    if !block.header.is_empty() || !block.rows.is_empty() {
        return table_rows(block).join("\n");
    }
    block.lines.join("\n")
}

fn table_rows(block: &LayoutBlock) -> Vec<String> {
    let mut rows = Vec::new();
    if !block.header.is_empty() {
        rows.push(
            block
                .header
                .iter()
                .map(|cell| cell.text.as_str())
                .collect::<Vec<_>>()
                .join(" | "),
        );
    }
    for row in &block.rows {
        rows.push(
            row.iter()
                .map(|cell| cell.text.as_str())
                .collect::<Vec<_>>()
                .join(" | "),
        );
    }
    rows
}

fn normalize(value: &str) -> String {
    value
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn estimate_tokens(text: &str) -> usize {
    text.chars().count().div_ceil(4)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{LayoutCell, PageLayout};

    fn layout() -> SourceLayout {
        SourceLayout::new(
            "PD3",
            1,
            vec![PageLayout {
                page: 1,
                page_label: None,
                width: 595.0,
                height: 842.0,
                content_bounds: None,
                blocks: vec![
                    LayoutBlock {
                        id: "PD3:p1:b0".into(),
                        kind: LayoutBlockKind::Heading,
                        bbox: None,
                        text: Some("3.1 Giriş".into()),
                        level: Some(1),
                        bold: false,
                        italic: false,
                        ordered: None,
                        marker: None,
                        lines: vec![],
                        lang: None,
                        header: vec![],
                        rows: vec![],
                        header_rows: None,
                        figure_id: None,
                        figure_format: None,
                    },
                    LayoutBlock {
                        id: "PD3:p1:b1".into(),
                        kind: LayoutBlockKind::Paragraph,
                        bbox: None,
                        text: Some(
                            "Türkçe bir tanım. 2026 yılında 42 örnek vardır. [PD3, s. 5]".into(),
                        ),
                        level: None,
                        bold: false,
                        italic: false,
                        ordered: None,
                        marker: None,
                        lines: vec![],
                        lang: None,
                        header: vec![],
                        rows: vec![],
                        header_rows: None,
                        figure_id: None,
                        figure_format: None,
                    },
                    LayoutBlock {
                        id: "PD3:p1:b2".into(),
                        kind: LayoutBlockKind::Table,
                        bbox: None,
                        text: None,
                        level: None,
                        bold: false,
                        italic: false,
                        ordered: None,
                        marker: None,
                        lines: vec![],
                        lang: None,
                        header: vec![LayoutCell {
                            text: "Ad".into(),
                            bbox: None,
                            colspan: None,
                            rowspan: None,
                        }],
                        rows: vec![vec![LayoutCell {
                            text: "Değer".into(),
                            bbox: None,
                            colspan: None,
                            rowspan: None,
                        }]],
                        header_rows: None,
                        figure_id: None,
                        figure_format: None,
                    },
                ],
            }],
        )
    }

    #[test]
    fn brief_is_bounded_and_page_anchored() {
        let brief = section_brief(&layout(), "3.1", 300).expect("brief");
        assert_eq!(brief.heading, "3.1 Giriş");
        assert!(brief.summary.contains("Türkçe"));
        assert!(brief.numbers.iter().any(|value| value == "2026"));
        assert!(brief.citations.iter().any(|value| value.contains("PD3")));
        assert!(brief.table_rows.iter().any(|value| value.contains("Ad")));
    }

    #[test]
    fn read_section_modes_are_bounded_and_explicit() {
        let summary = read_section_mode(&layout(), "3.1", 200, 0, "summary").expect("summary");
        assert_eq!(summary.mode, "summary");
        assert_eq!(summary.level, "section");
        let claims = read_section_mode(&layout(), "3.1", 200, 0, "claims").expect("claims");
        assert!(claims.text.contains("2026") || claims.text.contains("[PD3"));
        let tables = read_section_mode(&layout(), "3.1", 200, 0, "tables").expect("tables");
        assert!(tables.text.contains("Ad") || tables.text.is_empty());
    }

    #[test]
    fn read_section_reports_truncation() {
        let read = read_section(&layout(), "3.1", 20, 0).expect("read");
        assert!(read.truncated);
        assert!(read.next_offset.is_some());
    }
}
