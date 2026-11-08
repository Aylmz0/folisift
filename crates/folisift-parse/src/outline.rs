//! Canonical outline and greenfield skeleton.
//!
//! One shared builder owns the heading universe for the inventory, the coverage
//! manifest and the generated Markdown skeleton. The scaffold is deliberately
//! boring: front matter, a table of contents, one chapter per source, one
//! `##`/`###` entry per canonical heading, the structure checklist and the
//! coverage audit. It never emits a `::: front` page, so a generated guide
//! cannot trip the strict audit's consecutive-chapter-opener check.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::heading::{
    clean_heading_title, parse_numbered_heading, recover_inline_headings, scan_table_headings,
};
use crate::inventory::Inventory;
use crate::layout::{LayoutBlockKind, SourceLayout};

/// Where a canonical heading came from. Lower rank wins.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeadingOrigin {
    /// Explicit `heading` block from the layout classifier.
    Heading,
    /// Numbered heading recovered from paragraph/list text.
    Inline,
    /// Numbered heading at the start of a table cell.
    TableCell,
    /// Fallback inventory heading without layout evidence.
    Layout,
    /// Source topic list / TOC fallback.
    Toc,
}

impl HeadingOrigin {
    fn rank(self) -> u8 {
        match self {
            Self::Heading => 0,
            Self::Inline => 1,
            Self::TableCell => 2,
            Self::Layout => 3,
            Self::Toc => 4,
        }
    }
}

/// One heading in the canonical source outline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalHeading {
    pub source: String,
    pub number: String,
    pub title: String,
    /// `"{number} {title}"`, normalized for rendering and coverage keys.
    pub heading: String,
    pub page: usize,
    /// Markdown level: 2 for `##`, 3 for `###`.
    pub level: u8,
    pub origin: HeadingOrigin,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block_id: Option<String>,
}

/// One source group in the outline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutlineChapter {
    pub source: String,
    pub title: String,
    pub headings: Vec<CanonicalHeading>,
}

/// A deterministic outline over one inventory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Outline {
    pub chapters: Vec<OutlineChapter>,
    pub items: Vec<CanonicalHeading>,
}

impl Outline {
    /// Build from an inventory and its stored layouts.
    pub fn build(inventory: &Inventory, layouts: &[SourceLayout]) -> Self {
        let items = canonical_headings(inventory, layouts);
        let chapters = inventory
            .sources
            .iter()
            .map(|source| {
                let id = source.filename.trim_end_matches(".pdf").to_string();
                let headings = items
                    .iter()
                    .filter(|heading| heading.source == id)
                    .cloned()
                    .collect::<Vec<_>>();
                OutlineChapter {
                    source: id,
                    title: sanitize_source_title(source.title_or_filename()),
                    headings,
                }
            })
            .collect();
        Self { chapters, items }
    }
}

/// Build the canonical heading list for every source.
///
/// Evidence rank: explicit heading > inline paragraph > table cell > fallback
/// inventory heading > TOC. One number appears once; lower-rank evidence never
/// overrides higher-rank evidence.
pub fn canonical_headings(
    inventory: &Inventory,
    layouts: &[SourceLayout],
) -> Vec<CanonicalHeading> {
    let mut out = Vec::new();
    for source in &inventory.sources {
        let source_id = source.filename.trim_end_matches(".pdf");
        let layout = layouts.iter().find(|layout| layout.source == source_id);
        let mut rows = BTreeMap::<String, (u8, CanonicalHeading)>::new();

        if let Some(layout) = layout {
            for page in &layout.pages {
                for block in &page.blocks {
                    if block.kind != LayoutBlockKind::Heading {
                        continue;
                    }
                    let Some(parsed) = parse_numbered_heading(block.text_or_empty()) else {
                        continue;
                    };
                    insert_heading(
                        &mut rows,
                        source_id,
                        parsed.number,
                        parsed.title,
                        page.page,
                        HeadingOrigin::Heading,
                        Some(block.id.clone()),
                    );
                }
            }
            for heading in recover_inline_headings(&layout.pages) {
                let Some(parsed) = parse_numbered_heading(&heading.heading) else {
                    continue;
                };
                insert_heading(
                    &mut rows,
                    source_id,
                    parsed.number,
                    parsed.title,
                    heading.page,
                    HeadingOrigin::Inline,
                    heading.block_id,
                );
            }
            for heading in scan_table_headings(&layout.pages) {
                let Some(parsed) = parse_numbered_heading(&heading.heading) else {
                    continue;
                };
                insert_heading(
                    &mut rows,
                    source_id,
                    parsed.number,
                    parsed.title,
                    heading.page,
                    HeadingOrigin::TableCell,
                    heading.block_id,
                );
            }
        }

        // Inventory headings are a fallback for sources without a stored
        // layout, and a safety net for headings the layout scan could not see.
        for heading in &source.headings {
            let Some(parsed) = parse_numbered_heading(&heading.heading) else {
                continue;
            };
            if rows.contains_key(&parsed.number) {
                continue;
            }
            insert_heading(
                &mut rows,
                source_id,
                parsed.number,
                parsed.title,
                heading.page,
                HeadingOrigin::Layout,
                heading.block_id.clone(),
            );
        }

        // TOC rows are a last resort. They never create a heading when the
        // layout already has one for the number.
        for section in &source.sections {
            let candidate = format!("{} {}", section.number, section.title);
            let Some(parsed) = parse_numbered_heading(&candidate) else {
                continue;
            };
            if rows.contains_key(&parsed.number) {
                continue;
            }
            insert_heading(
                &mut rows,
                source_id,
                parsed.number,
                parsed.title,
                section.page,
                HeadingOrigin::Toc,
                None,
            );
        }

        let mut source_rows = rows
            .into_values()
            .map(|(_, heading)| heading)
            .collect::<Vec<_>>();
        source_rows.sort_by(|left, right| {
            left.page
                .cmp(&right.page)
                .then_with(|| compare_numbers(&left.number, &right.number))
        });
        out.extend(source_rows);
    }
    out
}

fn insert_heading(
    rows: &mut BTreeMap<String, (u8, CanonicalHeading)>,
    source: &str,
    number: String,
    title: String,
    page: usize,
    origin: HeadingOrigin,
    block_id: Option<String>,
) {
    let rank = origin.rank();
    let title = clean_heading_title(&title);
    let heading = CanonicalHeading {
        source: source.to_string(),
        number: number.clone(),
        title: title.clone(),
        heading: format!("{number} {title}"),
        page,
        level: level_for_number(&number),
        origin,
        block_id,
    };
    match rows.get(&number) {
        Some((existing_rank, _)) if *existing_rank <= rank => {}
        _ => {
            rows.insert(number, (rank, heading));
        }
    }
}

fn level_for_number(number: &str) -> u8 {
    match number.matches('.').count() {
        0 | 1 => 2,
        _ => 3,
    }
}

fn compare_numbers(left: &str, right: &str) -> std::cmp::Ordering {
    let key = |value: &str| {
        value
            .split('.')
            .filter_map(|part| part.parse::<u64>().ok())
            .collect::<Vec<_>>()
    };
    key(left).cmp(&key(right)).then_with(|| left.cmp(right))
}

fn sanitize_source_title(value: &str) -> String {
    value
        .replace('"', "'")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Render the greenfield guide skeleton from a built outline.
pub fn skeleton_markdown(
    title: &str,
    language: &str,
    structure: &str,
    inventory: &Inventory,
    outline: &Outline,
) -> String {
    let source_ids = inventory
        .sources
        .iter()
        .map(|source| source.filename.trim_end_matches(".pdf").to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let mut body = format!(
        "---\ntitle: {title:?}\nrunning: {title:?}\nlanguage: {language:?}\nsources: [{source_ids}]\n---\n\n"
    );
    if language == "en" {
        body.push_str(
            "::: toc title=\"Guide map\" note=\"Page numbers refer to this PDF.\"\n:::\n\n",
        );
    } else {
        body.push_str(
            "::: toc title=\"Kılavuzun haritası\" note=\"Sayfa numaraları bu PDF’e aittir.\"\n:::\n\n",
        );
    }

    if structure == "irac" {
        for case in 1..=10 {
            if language == "en" {
                body.push_str(&format!(
                    "## Case {case}\n\n### Issue\n\nDescribe the legal issue. [PD0, p. 1]\n\n### Rule\n\nState the applicable rule. [PD0, p. 1]\n\n### Application\n\nApply the rule to the facts. [PD0, p. 1]\n\n### Conclusion\n\nGive the reasoned conclusion. [PD0, p. 1]\n\n"
                ));
            } else {
                body.push_str(&format!(
                    "## Dava {case}\n\n### Sorun\n\nÇözülecek hukuki soruyu yaz. [PD0, s. 1]\n\n### Kural\n\nUygulanacak norm ve içtihadı yaz. [PD0, s. 1]\n\n### Uygulama\n\nKuralı olaya uygula. [PD0, s. 1]\n\n### Sonuç\n\nGerekçeli sonucu yaz. [PD0, s. 1]\n\n"
                ));
            }
        }
    } else {
        for (index, chapter) in outline.chapters.iter().enumerate() {
            body.push_str(&format!(
                "::: chapter part=\"{part}\" side=\"{side}: {id}\" kicker=\"{kicker} {number}\" title=\"{name}\" toc=\"{name}\" running=\"{name}\" subs=true\n:::\n\n",
                part = if language == "en" { "B · DETAILED REVIEW" } else { "B · AYRINTILI İNCELEME" },
                side = if language == "en" { "SOURCE" } else { "KAYNAK" },
                kicker = if language == "en" { "CHAPTER" } else { "BÖLÜM" },
                number = index + 1,
                id = chapter.source,
                name = chapter.title,
            ));
            if chapter.headings.is_empty() {
                let (heading, note) = if language == "en" {
                    ("Giriş", "This section will be written from the source.")
                } else {
                    ("Giriş", "Bu bölüm kaynaktan yazılacaktır.")
                };
                body.push_str(&format!("## {heading}\n\n{note}\n\n"));
                continue;
            }
            for heading in &chapter.headings {
                let hashes = "#".repeat(usize::from(heading.level));
                let note = if language == "en" {
                    "This section will be written from the source.".to_string()
                } else {
                    "Bu bölüm kaynaktan yazılacaktır.".to_string()
                };
                body.push_str(&format!(
                    "<!-- covers: {id}:{number} -->\n{hashes} {heading}\n\n{note}\n\n",
                    id = chapter.source,
                    number = heading.number,
                    heading = heading.heading,
                ));
            }
        }
    }

    let required = folisift_core::structures::scaffold_sections(structure, language);
    if !required.is_empty() && structure != "irac" {
        if language == "en" {
            body.push_str("::: chapter part=\"C · STRUCTURE\" side=\"FRAMEWORK\" kicker=\"C\" title=\"Structure checklist\" toc=\"Structure checklist\" running=\"Structure checklist\" subs=false\n:::\n\n");
        } else {
            body.push_str("::: chapter part=\"C · YAPI\" side=\"ÇERÇEVE\" kicker=\"C\" title=\"Yapı denetim listesi\" toc=\"Yapı denetim listesi\" running=\"Yapı denetim listesi\" subs=false\n:::\n\n");
        }
        for (section, hint, level) in required {
            let hint = if language == "en" {
                "Source content goes here.".to_string()
            } else {
                hint
            };
            body.push_str(&format!(
                "{} {section}\n\n{hint}\n\n",
                "#".repeat(usize::from(level))
            ));
        }
    }
    body.push_str("<!-- audit -->\n");
    if language == "en" {
        body.push_str("::: chapter part=\"AUDIT\" side=\"QUALITY\" kicker=\"AUDIT\" title=\"Coverage audit\" toc=\"Coverage audit\" running=\"Coverage audit\" subs=false\n:::\n\n## Coverage\n\nSource coverage and limitations will be reported here.\n");
    } else {
        body.push_str("::: chapter part=\"DENETİM\" side=\"KALİTE\" kicker=\"DENETİM\" title=\"Kapsam denetimi\" toc=\"Kapsam denetimi\" running=\"Kapsam denetimi\" subs=false\n:::\n\n## Kapsam\n\nKaynak kapsamı ve sınırlamalar burada raporlanacaktır.\n");
    }
    let citation_source = inventory
        .sources
        .first()
        .map(|source| source.filename.trim_end_matches(".pdf"))
        .unwrap_or("SOURCE");
    body = body
        .replace("[PD0, s. 1]", &format!("[{citation_source}, s. 1]"))
        .replace("[PD0, p. 1]", &format!("[{citation_source}, p. 1]"));
    body
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{CanonicalSection, Heading, Inventory, SourceInventory, Totals};
    use crate::layout::{LayoutBlock, LayoutBlockKind, PageLayout};

    fn source(headings: Vec<Heading>) -> SourceInventory {
        SourceInventory {
            path: "PD2.pdf".to_string(),
            filename: "PD2.pdf".to_string(),
            title: "PD2".to_string(),
            pages: 1,
            chars: 0,
            words: 0,
            low_text_threshold: 120,
            source_digest: None,
            low_text_pages: Vec::new(),
            headings,
            layout: None,
            sections: Vec::new(),
            page_map: Vec::new(),
        }
    }

    fn inventory(source: SourceInventory) -> Inventory {
        Inventory {
            generated_at: "now".to_string(),
            low_text_threshold: 120,
            sources: vec![source],
            totals: Totals {
                pages: 1,
                words: 0,
                low_text_pages: 0,
            },
        }
    }

    #[test]
    fn top_level_and_nested_numbers_survive() {
        let headings = [
            "1 Giriş",
            "1.1 Alt başlık",
            "1.1.1 Ayrıntı",
            "2 Yöntem",
            "10.2 Sonuç",
        ]
        .into_iter()
        .enumerate()
        .map(|(index, heading)| Heading {
            heading: heading.to_string(),
            page: index + 1,
            level: None,
            block_id: None,
        })
        .collect();
        let inventory = inventory(source(headings));
        let outline = Outline::build(&inventory, &[]);
        assert_eq!(outline.items.len(), 5);
        assert!(outline.items.iter().any(|heading| heading.number == "10.2"));
    }

    #[test]
    fn skeleton_has_no_front_and_one_heading_per_section() {
        let headings = ["1 Giriş", "1.1 Alt başlık", "2 Yöntem"]
            .into_iter()
            .map(|heading| Heading {
                heading: heading.to_string(),
                page: 1,
                level: None,
                block_id: None,
            })
            .collect();
        let inventory = inventory(source(headings));
        let outline = Outline::build(&inventory, &[]);
        let markdown = skeleton_markdown("Test", "tr", "a-g", &inventory, &outline);
        assert!(!markdown.contains("::: front"));
        assert!(markdown.contains("## 1 Giriş"));
        assert!(markdown.contains("## 1.1 Alt başlık"));
        assert!(markdown.contains("## 2 Yöntem"));
        assert!(markdown.contains("<!-- covers: PD2:1.1 -->"));
    }

    #[test]
    fn empty_source_gets_a_chapter_and_intro() {
        let inventory = inventory(source(Vec::new()));
        let outline = Outline::build(&inventory, &[]);
        let markdown = skeleton_markdown("Test", "tr", "a-g", &inventory, &outline);
        assert!(markdown.contains("::: chapter"));
        assert!(markdown.contains("## Giriş"));
    }

    #[test]
    fn table_cell_heading_joins_the_outline() {
        let layout = SourceLayout::new(
            "PD2",
            1,
            vec![PageLayout {
                page: 1,
                page_label: None,
                width: 100.0,
                height: 100.0,
                content_bounds: None,
                blocks: vec![LayoutBlock {
                    id: "PD2:p1:b0".to_string(),
                    kind: LayoutBlockKind::Table,
                    bbox: None,
                    text: None,
                    level: None,
                    bold: false,
                    italic: false,
                    ordered: None,
                    marker: None,
                    lines: Vec::new(),
                    lang: None,
                    header: Vec::new(),
                    rows: vec![vec![crate::layout::LayoutCell {
                        text: "10.2. Ulusal Bayram ve Genel Tatiller".to_string(),
                        bbox: None,
                        colspan: None,
                        rowspan: None,
                    }]],
                    header_rows: None,
                    figure_id: None,
                    figure_format: None,
                }],
            }],
        );
        let inventory = inventory(source(Vec::new()));
        let outline = Outline::build(&inventory, &[layout]);
        assert!(outline.items.iter().any(|heading| heading.number == "10.2"));
    }

    #[test]
    fn toc_is_a_fallback_only() {
        let mut source = source(vec![Heading {
            heading: "2.1 Giriş".to_string(),
            page: 3,
            level: None,
            block_id: None,
        }]);
        source.sections = vec![CanonicalSection {
            number: "2.1".to_string(),
            title: "Giriş".to_string(),
            page: 9,
            origin: "toc".to_string(),
        }];
        let inventory = inventory(source);
        let outline = Outline::build(&inventory, &[]);
        let item = outline
            .items
            .iter()
            .find(|heading| heading.number == "2.1")
            .expect("heading");
        assert_eq!(item.page, 3);
        assert_eq!(item.origin, HeadingOrigin::Layout);
    }
}
