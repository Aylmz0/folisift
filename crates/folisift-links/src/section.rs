use folisift_parse::{LayoutBlockKind, SourceLayout};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Section {
    pub id: String,
    pub source: String,
    pub page_start: usize,
    pub page_end: usize,
    pub block_ids: Vec<String>,
    pub heading: Option<String>,
    pub text: String,
    pub chars: usize,
}

pub fn sections_from_layout(layout: &SourceLayout, max_chars: usize) -> Vec<Section> {
    let max_chars = max_chars.max(200);
    let mut sections = Vec::new();
    let mut current: Option<Section> = None;
    let mut index = 0usize;

    for page in &layout.pages {
        for block in &page.blocks {
            if block.kind == LayoutBlockKind::Heading {
                let heading = block.text_or_empty().trim().to_string();
                if heading.is_empty() {
                    continue;
                }
                if let Some(section) = current.as_mut()
                    && section.text.trim() == section.heading.as_deref().unwrap_or_default()
                {
                    // A wrapped heading (for example "2.18 ..." followed by
                    // "Languages):") is still one heading, not a new section.
                    section.text.push(' ');
                    section.text.push_str(&heading);
                    section.chars = section.text.chars().count();
                    section.block_ids.push(block.id.clone());
                    continue;
                }
                if let Some(section) = current.take()
                    && !section.text.trim().is_empty()
                {
                    sections.push(section);
                }
                index += 1;
                current = Some(Section {
                    id: format!("{}:p{}:s{}", layout.source, page.page, index),
                    source: layout.source.clone(),
                    page_start: page.page,
                    page_end: page.page,
                    block_ids: vec![block.id.clone()],
                    heading: Some(heading.clone()),
                    text: heading,
                    chars: 0,
                });
                continue;
            }

            let text = block_text(block);
            if text.is_empty() {
                continue;
            }
            if current.is_none() {
                index += 1;
                current = Some(Section {
                    id: format!("{}:p{}:s{}", layout.source, page.page, index),
                    source: layout.source.clone(),
                    page_start: page.page,
                    page_end: page.page,
                    block_ids: Vec::new(),
                    heading: None,
                    text: String::new(),
                    chars: 0,
                });
            }
            let section = current.as_mut().expect("section");
            if !section.text.is_empty() && section.chars + text.chars().count() > max_chars {
                let mut next = section.clone();
                next.id = format!("{}:p{}:s{}", layout.source, page.page, index + 1);
                next.block_ids = vec![block.id.clone()];
                next.text = text.clone();
                next.chars = text.chars().count();
                next.page_start = page.page;
                next.page_end = page.page;
                sections.push(current.take().expect("section"));
                index += 1;
                current = Some(next);
            } else {
                if !section.text.is_empty() {
                    section.text.push('\n');
                }
                section.text.push_str(&text);
                section.chars = section.text.chars().count();
                section.page_end = page.page;
                section.block_ids.push(block.id.clone());
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

fn block_text(block: &folisift_parse::LayoutBlock) -> String {
    if let Some(text) = block.text.as_deref() {
        return text.trim().to_string();
    }
    if !block.header.is_empty() || !block.rows.is_empty() {
        let mut lines = Vec::new();
        if !block.header.is_empty() {
            lines.push(
                block
                    .header
                    .iter()
                    .map(|cell| cell.text.as_str())
                    .collect::<Vec<_>>()
                    .join(" | "),
            );
        }
        for row in &block.rows {
            lines.push(
                row.iter()
                    .map(|cell| cell.text.as_str())
                    .collect::<Vec<_>>()
                    .join(" | "),
            );
        }
        return lines.join("\n");
    }
    if !block.lines.is_empty() {
        return block.lines.join("\n");
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use folisift_parse::{LayoutBlock, LayoutBlockKind, PageLayout};

    fn block(id: &str, kind: LayoutBlockKind, text: &str) -> LayoutBlock {
        LayoutBlock {
            id: id.to_string(),
            kind,
            bbox: None,
            text: Some(text.to_string()),
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
        }
    }

    #[test]
    fn sections_start_at_headings_and_keep_page_ranges() {
        let layout = SourceLayout::new(
            "PD0",
            1,
            vec![PageLayout {
                page: 1,
                page_label: None,
                width: 600.0,
                height: 800.0,
                content_bounds: None,
                blocks: vec![
                    block("PD0:p1:b0", LayoutBlockKind::Heading, "1. Intro"),
                    block("PD0:p1:b1", LayoutBlockKind::Paragraph, "Body text."),
                    block("PD0:p1:b2", LayoutBlockKind::Heading, "2. Next"),
                    block("PD0:p1:b3", LayoutBlockKind::Paragraph, "More text."),
                ],
            }],
        );
        let sections = sections_from_layout(&layout, 1200);
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].heading.as_deref(), Some("1. Intro"));
        assert_eq!(sections[0].page_start, 1);
        assert_eq!(sections[0].block_ids, vec!["PD0:p1:b0", "PD0:p1:b1"]);
        assert!(sections[1].text.contains("More text."));
    }
}
