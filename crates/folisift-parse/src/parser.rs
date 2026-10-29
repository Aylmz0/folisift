use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use async_trait::async_trait;
use liteparse::config::OutputFormat;
use liteparse::{LiteParse, LiteParseConfig};
use regex::Regex;

use crate::error::{Error, Result};
use crate::inventory::{CanonicalSection, Heading, Inventory, PageRow, SourceInventory};
use crate::pdfium;
use crate::text::{
    LOW_TEXT_THRESHOLD, clean_text, count_chars, count_words, first_line, heading_of,
    meaningful_lines,
};

/// A source inventory producer. Tests can substitute a fixture implementation.
#[async_trait]
pub trait SourceParser: Send + Sync {
    async fn analyze(&self, path: &Path, low_text_threshold: usize) -> Result<SourceInventory>;
}

/// Production parser: liteparse with OCR disabled.
#[derive(Debug, Default, Clone, Copy)]
pub struct LiteparseParser;

impl LiteparseParser {
    fn engine() -> LiteParse {
        LiteParse::new(LiteParseConfig {
            ocr_enabled: false,
            output_format: OutputFormat::Text,
            quiet: true,
            max_pages: 10_000,
            ..Default::default()
        })
    }
}

#[async_trait]
impl SourceParser for LiteparseParser {
    async fn analyze(&self, path: &Path, low_text_threshold: usize) -> Result<SourceInventory> {
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

        let title = pdfium::document_title(path).unwrap_or_default();
        let mut page_map = Vec::with_capacity(result.pages.len());
        let mut headings = Vec::new();
        let mut seen_headings = HashSet::new();
        let mut low_text_pages = Vec::new();
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
            page_map.push(PageRow {
                page: page.page_number,
                first_line: first_line(&lines),
                chars,
                low_text,
            });
            if low_text {
                low_text_pages.push(page.page_number);
            }
            if let Some(heading) = heading_of(&lines) {
                let key = heading
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ")
                    .to_lowercase();
                if seen_headings.insert(key) {
                    headings.push(Heading {
                        heading,
                        page: page.page_number,
                    });
                }
            }
        }

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

        Ok(SourceInventory {
            path,
            filename,
            title,
            pages: result.pages.len(),
            chars: total_chars,
            words: total_words,
            low_text_threshold,
            low_text_pages,
            headings,
            sections,
            page_map,
        })
    }
}

/// Analyze several sources and return the combined inventory.
pub async fn analyze_sources(paths: &[PathBuf], low_text_threshold: usize) -> Result<Inventory> {
    let parser = LiteparseParser;
    let mut sources = Vec::with_capacity(paths.len());
    for path in paths {
        sources.push(parser.analyze(path, low_text_threshold).await?);
    }
    Ok(Inventory::new(low_text_threshold, sources))
}

/// Analyze one source with the default threshold.
pub async fn analyze_source(path: &Path) -> Result<SourceInventory> {
    LiteparseParser.analyze(path, LOW_TEXT_THRESHOLD).await
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

    for (page_number, text) in pages {
        let lines = text.lines().collect::<Vec<_>>();
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
            // Some decks repeat a number for later items (for example
            // "8." on every line of a continuation slide). Use list order in
            // that case; explicit "2.1" entries are left untouched.
            if minor.is_none() && last_major > 0 && major <= last_major {
                major = last_major + 1;
            }
            let number = match minor {
                Some(minor) => format!("{major}.{minor}"),
                None => match prefix {
                    Some(prefix) if !prefix.is_empty() => format!("{prefix}.{major}"),
                    _ => major.to_string(),
                },
            };
            let mut title = captures[3].trim().to_string();
            // Join continuation lines until the next numbered item or a
            // page/section header. This keeps wrapped TOC titles useful for
            // fuzzy matching without inventing content.
            let mut lookahead = index + 1;
            let mut joins = 0;
            while lookahead < lines.len() && joins < 4 {
                let next = lines[lookahead].trim();
                if next.is_empty() {
                    break;
                }
                if TOC_LINE_RE.is_match(next)
                    || next.chars().all(|character| character.is_ascii_digit())
                    || next.starts_with("Bölüm ")
                    || next.starts_with("Chapter ")
                {
                    break;
                }
                title.push(' ');
                title.push_str(next);
                lookahead += 1;
                joins += 1;
            }
            index = lookahead;
            if title.is_empty() || !seen.insert(number.clone()) {
                continue;
            }
            sections.push(CanonicalSection {
                number,
                title,
                page: *page_number,
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
        assert_eq!(sections[1].number, "2.2");
        assert!(sections[1].title.contains("Pseudocode"));
        assert_eq!(sections[2].number, "2.3");
        assert_eq!(
            sections
                .iter()
                .map(|section| section.number.as_str())
                .collect::<Vec<_>>(),
            vec!["2.1", "2.2", "2.3", "2.8", "2.9", "2.10"]
        );
    }
}
