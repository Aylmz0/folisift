use std::collections::HashSet;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use liteparse::config::OutputFormat;
use liteparse::{LiteParse, LiteParseConfig};

use crate::error::{Error, Result};
use crate::inventory::{Heading, Inventory, PageRow, SourceInventory};
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
}
