//! Section-oriented retrieval for agent context.
//!
//! Briefs are extractive and deterministic: no model call, no network. They
//! turn stored layout blocks into a small, page-anchored answer before the host
//! asks for the full section.

use std::path::Path;

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::layout::{LayoutBlock, LayoutBlockKind, SourceLayout};

pub const BRIEF_CACHE_VERSION: u32 = 3;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BriefSection {
    pub heading: String,
    pub page_start: usize,
    pub page_end: usize,
    pub summary: String,
    pub estimated_tokens: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SectionBrief {
    pub source: String,
    pub heading: String,
    pub level: String,
    #[serde(default = "default_mode")]
    pub mode: String,
    #[serde(default)]
    pub sections: Vec<BriefSection>,
    pub page_start: usize,
    pub page_end: usize,
    pub summary: String,
    pub numbers: Vec<String>,
    pub table_rows: Vec<String>,
    pub citations: Vec<String>,
    pub chars: usize,
    pub estimated_tokens: usize,
    #[serde(default)]
    pub full_chars: usize,
    #[serde(default)]
    pub budget_tokens: usize,
    #[serde(default)]
    pub budget_chars: usize,
    #[serde(default)]
    pub requested_tokens: usize,
    #[serde(default)]
    pub response_tokens: usize,
    #[serde(default)]
    pub truncated: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_offset: Option<usize>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub candidates: Vec<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub ambiguous: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub cache_hit: bool,
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
                    && (!section.heading.trim().is_empty() || !section.text.trim().is_empty())
                {
                    sections.push(section);
                }
                current = Some(SourceSection {
                    heading: heading.to_string(),
                    page_start: page.page,
                    page_end: page.page,
                    text: String::new(),
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
        && (!section.heading.trim().is_empty() || !section.text.trim().is_empty())
    {
        sections.push(section);
    }
    sections
}

struct SectionResolution<'a> {
    section: &'a SourceSection,
    ambiguous: bool,
    candidates: Vec<String>,
}

pub fn find_section<'a>(sections: &'a [SourceSection], query: &str) -> Option<&'a SourceSection> {
    resolve_section(sections, query).map(|resolution| resolution.section)
}

fn resolve_section<'a>(
    sections: &'a [SourceSection],
    query: &str,
) -> Option<SectionResolution<'a>> {
    let query = normalize(query).trim_end_matches('.').to_string();
    if query.is_empty() {
        return None;
    }
    let query_number = query.split_whitespace().next().unwrap_or_default();
    let mut best: Option<(&'a SourceSection, u8)> = None;
    let mut candidates = Vec::new();
    for section in sections {
        let heading = normalize(&section.heading);
        let number = heading_number(&section.heading);
        let rank = if heading == query || number == query_number {
            0
        } else if heading.starts_with(&format!("{query} "))
            || number.starts_with(&format!("{query_number}."))
        {
            1
        } else if heading.contains(&query) {
            2
        } else {
            continue;
        };
        match best {
            None => {
                best = Some((section, rank));
                candidates.push(section.heading.clone());
            }
            Some((_, best_rank)) if rank < best_rank => {
                best = Some((section, rank));
                candidates.clear();
                candidates.push(section.heading.clone());
            }
            Some((best_section, best_rank))
                if rank == best_rank && best_section.heading != section.heading =>
            {
                candidates.push(section.heading.clone());
            }
            _ => {}
        }
    }
    let (section, _) = best?;
    candidates.truncate(5);
    Some(SectionResolution {
        section,
        ambiguous: candidates.len() > 1,
        candidates,
    })
}

/// Read a deterministic brief artifact or compute and store it.
///
/// The key is the layout digest + heading + character budget + brief version,
/// so an unchanged source layout is not re-extracted on every call.
pub fn section_brief_cached(
    cache_dir: &Path,
    layout: &SourceLayout,
    query: &str,
    max_chars: usize,
) -> Option<(SectionBrief, bool)> {
    let bytes = serde_json::to_vec(layout).ok()?;
    let layout_digest = blake3::hash(&bytes).to_hex().to_string();
    let key = blake3::hash(
        format!(
            "{BRIEF_CACHE_VERSION}\n{layout_digest}\n{}\n{max_chars}",
            query.trim().to_lowercase()
        )
        .as_bytes(),
    )
    .to_hex()
    .to_string();
    let path = cache_dir.join(format!("{key}.json"));
    if let Ok(text) = std::fs::read_to_string(&path)
        && let Ok(mut brief) = serde_json::from_str::<SectionBrief>(&text)
    {
        brief.cache_hit = true;
        return Some((brief, true));
    }
    let mut brief = section_brief(layout, query, max_chars)?;
    if std::fs::create_dir_all(cache_dir).is_ok()
        && let Ok(text) = serde_json::to_string_pretty(&brief)
    {
        let _ = std::fs::write(path, text + "\n");
    }
    brief.cache_hit = false;
    Some((brief, false))
}

fn is_false(value: &bool) -> bool {
    !*value
}

pub fn section_brief(layout: &SourceLayout, query: &str, max_chars: usize) -> Option<SectionBrief> {
    let sections = source_sections(layout);
    let resolution = resolve_section(&sections, query)?;
    let section = resolution.section;
    let max_chars = max_chars.clamp(40, 4000);
    let text = body_text(section);
    let full_chars = text.chars().count();
    let brief = SectionBrief {
        source: layout.source.clone(),
        heading: section.heading.clone(),
        level: section_level(&section.heading).to_string(),
        mode: "section".to_string(),
        sections: Vec::new(),
        page_start: section.page_start,
        page_end: section.page_end,
        summary: summary(text, max_chars),
        numbers: numbers(text),
        table_rows: section.table_rows.iter().take(8).cloned().collect(),
        citations: citations(text),
        chars: full_chars,
        estimated_tokens: 0,
        full_chars,
        budget_tokens: max_chars.div_ceil(4),
        budget_chars: max_chars,
        requested_tokens: max_chars.div_ceil(4),
        response_tokens: 0,
        truncated: false,
        next_offset: None,
        candidates: resolution.candidates,
        ambiguous: resolution.ambiguous,
        cache_hit: false,
    };
    Some(apply_budget(brief, max_chars))
}

/// Hierarchical chapter brief: the parent body plus one bounded summary per
/// immediate child section. It is still extractive; no generated prose.
pub fn chapter_brief(layout: &SourceLayout, query: &str, max_chars: usize) -> Option<SectionBrief> {
    let sections = source_sections(layout);
    let resolution = resolve_section(&sections, query)?;
    let chapter = resolution.section;
    let chapter_number = heading_number(&chapter.heading);
    let child_depth = chapter_number.matches('.').count() + 1;
    let children = sections
        .iter()
        .filter(|section| {
            let number = heading_number(&section.heading);
            number.starts_with(&format!("{chapter_number}."))
                && number.matches('.').count() == child_depth
        })
        .collect::<Vec<_>>();
    let mut parts = Vec::new();
    let chapter_body = body_text(chapter);
    if !chapter_body.is_empty() {
        parts.push(chapter_body.to_string());
    }
    for child in &children {
        let body = body_text(child);
        if !body.is_empty() {
            parts.push(body.to_string());
        }
    }
    let text = parts.join("\n").trim().to_string();
    let full_chars = text.chars().count();
    let max_chars = max_chars.clamp(80, 6000);
    let child_budget = (max_chars / children.len().max(1)).max(80);
    let brief_sections = children
        .iter()
        .map(|child| {
            let body = body_text(child);
            let child_summary = summary(body, child_budget);
            BriefSection {
                heading: child.heading.clone(),
                page_start: child.page_start,
                page_end: child.page_end,
                estimated_tokens: estimate_tokens(&child_summary),
                summary: child_summary,
            }
        })
        .collect::<Vec<_>>();
    let page_end = children
        .iter()
        .map(|child| child.page_end)
        .max()
        .unwrap_or(chapter.page_end);
    let brief = SectionBrief {
        source: layout.source.clone(),
        heading: chapter.heading.clone(),
        level: "chapter".to_string(),
        mode: "chapter".to_string(),
        sections: brief_sections,
        page_start: chapter.page_start,
        page_end,
        summary: summary(&text, max_chars),
        numbers: numbers(&text),
        table_rows: chapter
            .table_rows
            .iter()
            .chain(children.iter().flat_map(|child| child.table_rows.iter()))
            .take(8)
            .cloned()
            .collect(),
        citations: citations(&text),
        chars: full_chars,
        estimated_tokens: 0,
        full_chars,
        budget_tokens: max_chars.div_ceil(4),
        budget_chars: max_chars,
        requested_tokens: max_chars.div_ceil(4),
        response_tokens: 0,
        truncated: false,
        next_offset: None,
        candidates: resolution.candidates,
        ambiguous: resolution.ambiguous,
        cache_hit: false,
    };
    Some(apply_budget(brief, max_chars))
}

fn body_text(section: &SourceSection) -> &str {
    let text = section.text.trim();
    text.strip_prefix(section.heading.trim())
        .map(str::trim)
        .unwrap_or(text)
}

fn heading_number(heading: &str) -> String {
    static NUMBER_RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
        Regex::new(r"^\s*(\d+(?:\.\d+)*)\.?").expect("valid heading number regex")
    });
    NUMBER_RE
        .captures(heading)
        .and_then(|captures| captures.get(1))
        .map_or_else(String::new, |capture| capture.as_str().to_string())
}

fn apply_budget(mut brief: SectionBrief, max_chars: usize) -> SectionBrief {
    let budget = max_chars.div_ceil(4).max(10);
    brief.budget_chars = max_chars.max(40);
    brief.budget_tokens = budget;
    brief.estimated_tokens = 0;
    let summary_limit = max_chars.max(40);
    if brief.summary.chars().count() > summary_limit {
        brief.summary = brief.summary.chars().take(summary_limit).collect();
        brief.truncated = true;
    }
    let mut truncated = brief.truncated || brief.summary.chars().count() < brief.chars;
    if content_tokens(&brief) > budget {
        brief.table_rows.clear();
        brief.citations.clear();
        truncated = true;
    }
    if content_tokens(&brief) > budget {
        brief.numbers.truncate(4);
        truncated = true;
    }
    while content_tokens(&brief) > budget && !brief.sections.is_empty() {
        let mut changed = false;
        for section in &mut brief.sections {
            let length = section.summary.chars().count();
            if length > 40 {
                let target = (length / 2).max(40);
                section.summary = section.summary.chars().take(target).collect();
                section.estimated_tokens = section.summary.chars().count().div_ceil(4);
                changed = true;
            }
        }
        if !changed {
            break;
        }
        truncated = true;
    }
    while content_tokens(&brief) > budget && brief.sections.len() > 1 {
        brief.sections.pop();
        truncated = true;
    }
    if content_tokens(&brief) > budget {
        let min_chars = if brief.chars == 0 {
            0
        } else {
            40.min(brief.chars)
        };
        let target = (budget * 4)
            .max(min_chars)
            .min(brief.summary.chars().count());
        if brief.summary.chars().count() > target {
            brief.summary = brief.summary.chars().take(target).collect();
            truncated = true;
        }
    }
    if content_tokens(&brief) > budget {
        brief.sections.clear();
        brief.numbers.clear();
        brief.table_rows.clear();
        brief.citations.clear();
        truncated = true;
    }
    // At the minimum content budget, keep a short useful summary rather than
    // returning an empty string.
    if brief.summary.is_empty() && brief.chars > 0 {
        let fallback = brief
            .sections
            .first()
            .map(|section| section.summary.clone())
            .unwrap_or_else(|| brief.heading.clone());
        brief.summary = fallback.chars().take(40).collect();
        truncated = true;
    }
    if content_tokens(&brief) > budget {
        let target = (budget * 4).max(1);
        brief.summary = brief.summary.chars().take(target).collect();
        truncated = true;
    }
    brief.truncated = truncated;
    brief.next_offset = if truncated && !brief.summary.is_empty() {
        Some(brief.summary.chars().count())
    } else {
        None
    };
    brief.response_tokens = serialized_tokens(&brief);
    brief.estimated_tokens = content_tokens(&brief);
    brief
}

fn content_tokens(brief: &SectionBrief) -> usize {
    let mut chars = brief.summary.chars().count();
    for section in &brief.sections {
        chars += section.summary.chars().count() + section.heading.chars().count();
    }
    for value in brief
        .numbers
        .iter()
        .chain(&brief.table_rows)
        .chain(&brief.citations)
    {
        chars += value.chars().count();
    }
    chars.div_ceil(4)
}

fn default_mode() -> String {
    "section".to_string()
}

fn serialized_tokens(brief: &SectionBrief) -> usize {
    serde_json::to_string(brief)
        .map(|text| text.chars().count().div_ceil(4))
        .unwrap_or_else(|_| brief.estimated_tokens)
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
    let number = heading_number(heading);
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
        let brief = section_brief(&layout(), "3.1", 2000).expect("brief");
        assert_eq!(brief.heading, "3.1 Giriş");
        assert!(brief.summary.contains("Türkçe"));
        assert!(brief.numbers.iter().any(|value| value == "2026"));
        assert!(brief.citations.iter().any(|value| value.contains("PD3")));
        assert!(brief.table_rows.iter().any(|value| value.contains("Ad")));
    }

    #[test]
    fn chapter_number_does_not_match_a_subsection() {
        let sections = vec![
            SourceSection {
                heading: "1.4. Sosyal Güvenlik Hukuku".to_string(),
                page_start: 1,
                page_end: 1,
                text: "x".to_string(),
                table_rows: Vec::new(),
            },
            SourceSection {
                heading: "4 4857 sayılı Kanun".to_string(),
                page_start: 2,
                page_end: 2,
                text: "y".to_string(),
                table_rows: Vec::new(),
            },
        ];
        let resolved = resolve_section(&sections, "4").expect("4");
        assert_eq!(resolved.section.heading, "4 4857 sayılı Kanun");
        assert!(!resolved.ambiguous);
        assert_eq!(
            resolve_section(&sections, "1.4")
                .expect("1.4")
                .section
                .heading,
            "1.4. Sosyal Güvenlik Hukuku"
        );
    }

    #[test]
    fn brief_budget_is_bounded_and_explicit() {
        let brief = section_brief(&layout(), "3.1", 80).expect("brief");
        assert!(brief.truncated);
        assert!(brief.estimated_tokens <= 80, "{brief:#?}");
        assert!(brief.budget_tokens <= 20, "{brief:#?}");
    }

    #[test]
    fn brief_artifacts_are_reused_for_an_unchanged_layout() {
        let dir = tempfile::tempdir().expect("tempdir");
        let layout = layout();
        let (first, first_hit) =
            section_brief_cached(dir.path(), &layout, "3.1", 300).expect("first");
        let (second, second_hit) =
            section_brief_cached(dir.path(), &layout, "3.1", 300).expect("second");
        assert!(!first_hit);
        assert!(second_hit);
        assert_eq!(first.summary, second.summary);
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
