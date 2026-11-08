use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use async_trait::async_trait;
use liteparse::config::OutputFormat;
use liteparse::{LiteParse, LiteParseConfig};
use regex::Regex;

use crate::error::{Error, Result};
use crate::heading::{normalize_heading, recover_inline_headings};
use crate::inventory::{CanonicalSection, Heading, Inventory, PageRow, SourceInventory};
use crate::layout::{LayoutBlock, LayoutBlockKind, PageLayout, Rect, SourceLayout};
use crate::pdfium;
use crate::text::{
    LOW_TEXT_THRESHOLD, clean_text, count_chars, count_words, first_line, heading_of,
    meaningful_lines,
};

/// The two products of one source parse.
#[derive(Debug, Clone)]
pub struct AnalyzedSource {
    pub inventory: SourceInventory,
    pub layout: Option<SourceLayout>,
}

/// The combined result of analyzing several sources.
#[derive(Debug, Clone)]
pub struct Analysis {
    pub inventory: Inventory,
    pub layouts: Vec<SourceLayout>,
}

/// A source inventory producer. Tests can substitute a fixture implementation.
#[async_trait]
pub trait SourceParser: Send + Sync {
    async fn analyze(&self, path: &Path, low_text_threshold: usize) -> Result<AnalyzedSource>;
}

/// Production parser: liteparse with OCR disabled.
#[derive(Debug, Default, Clone, Copy)]
pub struct LiteparseParser;

impl LiteparseParser {
    fn engine() -> LiteParse {
        LiteParse::new(LiteParseConfig {
            ocr_enabled: false,
            output_format: OutputFormat::Json,
            extract_blocks: true,
            quiet: true,
            max_pages: 10_000,
            ..Default::default()
        })
    }
}

#[async_trait]
impl SourceParser for LiteparseParser {
    async fn analyze(&self, path: &Path, low_text_threshold: usize) -> Result<AnalyzedSource> {
        pdfium::init(None)?;

        let path_text = path.to_str().ok_or_else(|| Error::Parse {
            path: path.to_path_buf(),
            message: "source path is not valid UTF-8".to_string(),
        })?;
        let result = Self::engine()
            .parse(path_text)
            .await
            .map_err(|error| Error::Parse {
                path: path.to_path_buf(),
                message: error.to_string(),
            })?;
        if result.pages.is_empty() {
            return Err(Error::NoPages {
                path: path.to_path_buf(),
            });
        }

        let source_id = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("source")
            .trim_end_matches(".pdf")
            .to_string();
        let title = pdfium::document_title(path).unwrap_or_default();
        let mut page_map = Vec::with_capacity(result.pages.len());
        let mut fallback_headings = Vec::new();
        let mut layout_headings = Vec::new();
        let mut low_text_pages = Vec::new();
        let mut layout_pages = Vec::with_capacity(result.pages.len());
        let mut layout_enabled = false;
        let mut total_chars = 0;
        let mut total_words = 0;

        for page in &result.pages {
            let text = clean_text(&page.text);
            let chars = count_chars(&text);
            let words = count_words(&text);
            total_chars += chars;
            total_words += words;

            let lines = meaningful_lines(&page.text);
            let low_text = chars < low_text_threshold;
            let mut page_layout = None;
            if let Some(blocks) = &page.blocks {
                layout_enabled = true;
                let mapped = blocks
                    .iter()
                    .enumerate()
                    .map(|(index, block)| {
                        LayoutBlock::from_liteparse(
                            format!("{source_id}:p{}:b{index}", page.page_number),
                            block,
                        )
                    })
                    .collect::<Vec<_>>();
                let block_heading_count = mapped
                    .iter()
                    .filter(|block| block.kind == LayoutBlockKind::Heading)
                    .count();
                let table_count = mapped.iter().filter(|block| block.kind.is_table()).count();
                for block in &mapped {
                    if block.kind == LayoutBlockKind::Heading
                        && let Some(heading) = block.text.as_deref().map(str::trim)
                        && !heading.is_empty()
                    {
                        layout_headings.push(Heading {
                            heading: normalize_heading(heading),
                            page: page.page_number,
                            level: block.level,
                            block_id: Some(block.id.clone()),
                        });
                    }
                }
                page_map.push(PageRow {
                    page: page.page_number,
                    first_line: first_line(&lines),
                    chars,
                    low_text,
                    blocks: mapped.len(),
                    headings: block_heading_count,
                    tables: table_count,
                });
                page_layout = Some(PageLayout {
                    page: page.page_number,
                    page_label: page.page_label.clone(),
                    width: page.page_width,
                    height: page.page_height,
                    content_bounds: page.content_bounds.clone().map(Rect::from),
                    blocks: mapped,
                });
            } else {
                page_map.push(PageRow {
                    page: page.page_number,
                    first_line: first_line(&lines),
                    chars,
                    low_text,
                    blocks: 0,
                    headings: 0,
                    tables: 0,
                });
                if let Some(heading) = heading_of(&lines) {
                    fallback_headings.push(Heading {
                        heading,
                        page: page.page_number,
                        level: None,
                        block_id: None,
                    });
                }
            }
            if low_text {
                low_text_pages.push(page.page_number);
            }
            if let Some(page_layout) = page_layout {
                layout_pages.push(page_layout);
            }
        }

        layout_headings.extend(recover_inline_headings(&layout_pages));
        let numbered_layout_headings = layout_headings
            .iter()
            .filter(|heading| is_navigational_heading(&heading.heading))
            .cloned()
            .collect::<Vec<_>>();
        let headings = if !numbered_layout_headings.is_empty() {
            dedupe_headings(numbered_layout_headings)
        } else if !layout_headings.is_empty() {
            dedupe_headings(layout_headings)
        } else {
            dedupe_headings(fallback_headings)
        };
        let path = path
            .canonicalize()
            .unwrap_or_else(|_| path.to_path_buf())
            .display()
            .to_string();
        let filename = path.rsplit('/').next().unwrap_or(&path).to_string();
        let prefix = headings.iter().find_map(|heading| {
            let first = heading.heading.split('.').next()?;
            first
                .chars()
                .all(|character| character.is_ascii_digit())
                .then(|| first.to_string())
        });
        let toc_pages = result
            .pages
            .iter()
            .take(6)
            .map(|page| (page.page_number, page.text.as_str()))
            .collect::<Vec<_>>();
        let sections = canonical_sections(&toc_pages, prefix.as_deref());
        let source_digest = std::fs::read(&path)
            .ok()
            .map(|bytes| blake3::hash(&bytes).to_hex().to_string());
        let layout = layout_enabled
            .then(|| SourceLayout::new(source_id.clone(), result.pages.len(), layout_pages));

        Ok(AnalyzedSource {
            inventory: SourceInventory {
                path,
                filename,
                title,
                pages: result.pages.len(),
                chars: total_chars,
                words: total_words,
                low_text_threshold,
                source_digest,
                low_text_pages,
                headings,
                layout: layout.as_ref().map(SourceLayout::summary),
                sections,
                page_map,
            },
            layout,
        })
    }
}

fn is_navigational_heading(heading: &str) -> bool {
    let heading = heading.trim();
    if heading.starts_with("Bölüm ") || heading.starts_with("Chapter ") {
        return true;
    }
    let mut chars = heading.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_digit() {
        return false;
    }
    heading
        .trim_start_matches(|character: char| character.is_ascii_digit())
        .chars()
        .next()
        .is_some_and(|character| character == '.' || character.is_whitespace())
}

fn dedupe_headings(headings: Vec<Heading>) -> Vec<Heading> {
    let mut seen = HashSet::new();
    headings
        .into_iter()
        .filter(|heading| {
            let key = heading
                .heading
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .to_lowercase();
            seen.insert(key)
        })
        .collect()
}

/// Analyze several sources and return the combined inventory.
pub async fn analyze_sources(paths: &[PathBuf], low_text_threshold: usize) -> Result<Inventory> {
    Ok(analyze_sources_with_layout(paths, low_text_threshold)
        .await?
        .inventory)
}

/// Analyze several sources and return both inventory and full layouts.
pub async fn analyze_sources_with_layout(
    paths: &[PathBuf],
    low_text_threshold: usize,
) -> Result<Analysis> {
    let parser = LiteparseParser;
    let mut sources = Vec::with_capacity(paths.len());
    let mut layouts = Vec::new();
    for path in paths {
        let analyzed = parser.analyze(path, low_text_threshold).await?;
        sources.push(analyzed.inventory);
        if let Some(layout) = analyzed.layout {
            layouts.push(layout);
        }
    }
    Ok(Analysis {
        inventory: Inventory::new(low_text_threshold, sources),
        layouts,
    })
}

/// Analyze one source with the default threshold.
pub async fn analyze_source(path: &Path) -> Result<SourceInventory> {
    Ok(LiteparseParser
        .analyze(path, LOW_TEXT_THRESHOLD)
        .await?
        .inventory)
}

static TOC_LINE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*(\d{1,3})(?:\.(\d{1,3}))?\.?\s*(.+?)\s*$").expect("valid TOC regex")
});

/// Recover the source's own topic-list sections from the first pages.
///
/// The source TOC is the canonical coverage set. Heading extraction is a page
/// map fallback; it can miss headings with unusual styling or line wraps.
fn canonical_sections(pages: &[(usize, &str)], prefix: Option<&str>) -> Vec<CanonicalSection> {
    let mut sections = Vec::new();
    let mut seen = HashSet::new();
    let mut last_major = 0usize;
    let chapter_local = prefix.is_some_and(|prefix| {
        pages.iter().any(|(_, text)| {
            text.contains(&format!("Bölüm {prefix}")) || text.contains(&format!("Chapter {prefix}"))
        })
    });

    for (_page_number, text) in pages {
        let lines = text.lines().map(str::trim).collect::<Vec<_>>();
        let numbered = lines
            .iter()
            .filter(|line| {
                TOC_LINE_RE
                    .captures(line)
                    .and_then(|captures| captures.get(3))
                    .is_some_and(|title| title.as_str().chars().count() <= 160)
            })
            .count();
        let marker = lines.iter().any(|line| {
            matches!(
                line.to_lowercase()
                    .trim_matches(|character: char| !character.is_alphanumeric()),
                "konular" | "içindekiler" | "contents" | "table of contents"
            )
        }) || lines
            .iter()
            .any(|line| line.starts_with("Bölüm ") || line.starts_with("Chapter "));
        // A body page can contain the word "Konular" or numbered legal text;
        // require either a real TOC marker plus several entries, or a dense
        // numbered list. This prevents body prose from becoming a TOC title.
        if !((marker && numbered >= 3) || numbered >= 8) {
            continue;
        }
        let mut index = 0;
        while index < lines.len() {
            let line = lines[index].trim();
            let Some(captures) = TOC_LINE_RE.captures(line) else {
                index += 1;
                continue;
            };
            let major = captures[1].parse::<usize>().ok();
            let minor = captures.get(2).map(|value| value.as_str());
            let Some(mut major) = major else {
                index += 1;
                continue;
            };
            if major > 99 {
                index += 1;
                continue;
            }
            // Some decks repeat a number for later items (for example
            // "8." on every line of a continuation slide). This is a
            // heuristic, not a source fact; record it as `toc-sequence`.
            let mut origin = "toc";
            if minor.is_none() && last_major > 0 && major <= last_major {
                major = last_major + 1;
                origin = "toc-sequence";
            }
            let number = match minor {
                Some(minor) => format!("{major}.{minor}"),
                None => match prefix {
                    Some(prefix) if chapter_local && !prefix.is_empty() => {
                        format!("{prefix}.{major}")
                    }
                    _ => major.to_string(),
                },
            };
            let mut title = captures[3].trim().to_string();
            // Only join a continuation that looks like a wrapped title, not
            // body prose. A body sentence normally starts uppercase; a
            // wrapped TOC line continues lowercase or punctuation.
            let mut lookahead = index + 1;
            let mut joins = 0;
            while lookahead < lines.len() && joins < 3 {
                let next = lines[lookahead].trim();
                if next.is_empty()
                    || TOC_LINE_RE.is_match(next)
                    || next.chars().all(|character| character.is_ascii_digit())
                    || next.starts_with("Bölüm ")
                    || next.starts_with("Chapter ")
                    || !next
                        .chars()
                        .next()
                        .is_some_and(|character| !character.is_uppercase())
                {
                    break;
                }
                if title.chars().count() + next.chars().count() + 1 > 160 {
                    break;
                }
                title.push(' ');
                title.push_str(next);
                lookahead += 1;
                joins += 1;
            }
            index = lookahead;
            if title.is_empty() || title.chars().count() > 160 || !seen.insert(number.clone()) {
                continue;
            }
            sections.push(CanonicalSection {
                number,
                title,
                page: 0,
                origin: origin.to_string(),
            });
            last_major = major;
            if sections.len() >= 100 {
                return sections;
            }
        }
    }
    sections
}

/// Parse `1,3-5,9` into sorted, unique 1-based page numbers.
pub fn parse_page_list(value: &str) -> Result<Vec<u32>> {
    let mut pages = Vec::new();
    for part in value.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if let Some((start, end)) = part.split_once('-') {
            let start: u32 = start.trim().parse().map_err(|_| Error::PageSelection {
                value: value.to_string(),
            })?;
            let end: u32 = end.trim().parse().map_err(|_| Error::PageSelection {
                value: value.to_string(),
            })?;
            if start == 0 || end < start {
                return Err(Error::PageSelection {
                    value: value.to_string(),
                });
            }
            pages.extend(start..=end);
        } else {
            let page: u32 = part.parse().map_err(|_| Error::PageSelection {
                value: value.to_string(),
            })?;
            if page == 0 {
                return Err(Error::PageSelection {
                    value: value.to_string(),
                });
            }
            pages.push(page);
        }
    }
    pages.sort_unstable();
    pages.dedup();
    if pages.is_empty() {
        return Err(Error::PageSelection {
            value: value.to_string(),
        });
    }
    Ok(pages)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn parses_layout_blocks_headings_and_tables() {
        let path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/pdf/blocks.pdf");
        let analyzed = LiteparseParser
            .analyze(&path, 120)
            .await
            .expect("layout parse");
        let layout = analyzed.layout.expect("blocks enabled");
        assert_eq!(layout.page_count, 2);
        assert_eq!(layout.pages.len(), 2);
        assert_eq!(layout.pages[0].page, 1);
        assert!(
            layout.pages[0]
                .blocks
                .iter()
                .any(|block| block.kind == LayoutBlockKind::Heading && block.level == Some(1))
        );
        let table = layout.pages[1]
            .blocks
            .iter()
            .find(|block| block.kind == LayoutBlockKind::Table)
            .expect("table block");
        assert_eq!(
            table
                .header
                .iter()
                .map(|cell| cell.text.as_str())
                .collect::<Vec<_>>(),
            vec!["Term", "Meaning"]
        );
        assert_eq!(
            table.rows[0]
                .iter()
                .map(|cell| cell.text.as_str())
                .collect::<Vec<_>>(),
            vec!["Atom", "Indivisible unit"]
        );
        assert!(table.header.iter().all(|cell| cell.bbox.is_some()));
        let summary = analyzed.inventory.layout.expect("layout summary");
        assert_eq!(summary.headings, 3);
        assert_eq!(summary.tables, 1);
        assert!(
            analyzed
                .inventory
                .headings
                .iter()
                .any(|heading| heading.heading == "1 Layout fixture" && heading.level == Some(1))
        );
    }

    #[test]
    fn separated_and_glued_turkish_headings_normalize() {
        assert_eq!(normalize_heading("1.İş"), "1 İş");
        assert_eq!(normalize_heading("5.2.İş"), "5.2 İş");
        assert_eq!(normalize_heading("13.4.Tutulacak"), "13.4 Tutulacak");
        assert_eq!(normalize_heading("1. Giriş"), "1 Giriş");
    }

    #[test]
    fn parses_page_lists() {
        assert_eq!(parse_page_list("3,1-2,2").expect("pages"), vec![1, 2, 3]);
        assert_eq!(parse_page_list("7").expect("pages"), vec![7]);
    }

    #[test]
    fn rejects_bad_page_lists() {
        for value in ["", "0", "3-1", "a", "1-"] {
            assert!(parse_page_list(value).is_err(), "{value:?}");
        }
    }

    #[test]
    fn recovers_wrapped_and_repeated_toc_sections() {
        let first = "Bölüm 2 Konular\n1.  Zuse'nin Plankalkül'ü\n2.  Minimum Donanım Programlama: Sözde\nkod (Pseudocode)\n3.  IBM 704 ve Fortran\n";
        let second = "Bölüm 2 Konular (devamı)\n8. Herkes için Herşey: PL/I\n8. İlk iki Dinamik Dil: APL ve SNOBOL\n8. Veri Soyutlama\n";
        let sections = canonical_sections(&[(2, first), (3, second)], Some("2"));
        assert_eq!(sections[0].number, "2.1");
        assert_eq!(sections[0].title, "Zuse'nin Plankalkül'ü");
        assert_eq!(sections[0].page, 0);
        assert_eq!(sections[0].origin, "toc");
        assert_eq!(sections[1].number, "2.2");
        assert!(sections[1].title.contains("Pseudocode"));
        assert_eq!(sections[2].number, "2.3");
        assert_eq!(sections[4].origin, "toc-sequence");
        assert_eq!(
            sections
                .iter()
                .map(|section| section.number.as_str())
                .collect::<Vec<_>>(),
            vec!["2.1", "2.2", "2.3", "2.8", "2.9", "2.10"]
        );
    }

    #[test]
    fn preserves_major_numbers_without_a_chapter_marker() {
        let toc = "Contents\n1. Introduction\n2. Methods\n3. Results\n";
        let sections = canonical_sections(&[(2, toc)], Some("1"));
        assert_eq!(
            sections
                .iter()
                .map(|section| section.number.as_str())
                .collect::<Vec<_>>(),
            vec!["1", "2", "3"]
        );
    }
}
