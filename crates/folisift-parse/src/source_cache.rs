use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::error::{Error, Result, io};
use crate::pdfium;
use crate::text::clean_text;

const CACHE_VERSION: &str = env!("CARGO_PKG_VERSION");
const PAGE_MARKER: &str = "\n===== PAGE ";

#[derive(Debug, Clone, Serialize)]
pub struct PageText {
    pub page: usize,
    pub chars: usize,
    pub text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SourceText {
    pub source: String,
    pub sha256: String,
    pub pages: Vec<PageText>,
    pub cache_path: PathBuf,
    pub cache_hit: bool,
}

#[derive(Debug, Clone)]
pub struct SourceCache {
    root: PathBuf,
}

impl SourceCache {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Load cached page text or parse and cache it.
    pub async fn load(&self, source: &Path, force: bool) -> Result<SourceText> {
        let bytes = std::fs::read(source).map_err(|error| io(source, error))?;
        let hash = blake3::hash(&bytes).to_hex().to_string();
        let key = format!("{hash}-{CACHE_VERSION}");
        let dir = self.root.join(&key);
        let path = dir.join("source.txt");

        if !force
            && path.is_file()
            && let Ok(text) = std::fs::read_to_string(&path)
        {
            let pages = parse_cached(&text);
            if !pages.is_empty() {
                return Ok(SourceText {
                    source: source.display().to_string(),
                    sha256: hash,
                    pages,
                    cache_path: path,
                    cache_hit: true,
                });
            }
        }

        let pages = parse_pages(source).await?;
        std::fs::create_dir_all(&dir).map_err(|error| io(&dir, error))?;
        std::fs::write(&path, render_cached(source, &pages)).map_err(|error| io(&path, error))?;
        Ok(SourceText {
            source: source.display().to_string(),
            sha256: hash,
            pages,
            cache_path: path,
            cache_hit: false,
        })
    }
}

async fn parse_pages(source: &Path) -> Result<Vec<PageText>> {
    pdfium::init(None)?;
    let parser = liteparse::LiteParse::new(liteparse::LiteParseConfig {
        ocr_enabled: false,
        quiet: true,
        max_pages: 10_000,
        ..Default::default()
    });
    let source_text = source.to_str().ok_or_else(|| Error::Parse {
        path: source.to_path_buf(),
        message: "source path is not valid UTF-8".to_string(),
    })?;
    let result = parser
        .parse(source_text)
        .await
        .map_err(|error| Error::Parse {
            path: source.to_path_buf(),
            message: error.to_string(),
        })?;
    if result.pages.is_empty() {
        return Err(Error::NoPages {
            path: source.to_path_buf(),
        });
    }
    Ok(result
        .pages
        .iter()
        .map(|page| {
            let text = clean_text(&page.text);
            PageText {
                page: page.page_number,
                chars: text.chars().filter(|c| !c.is_whitespace()).count(),
                text,
            }
        })
        .collect())
}

fn render_cached(source: &Path, pages: &[PageText]) -> String {
    let mut out = format!("# {}\n# pages: {}\n", source.display(), pages.len());
    for page in pages {
        out.push_str(&format!("\n===== PAGE {} =====\n{}", page.page, page.text));
    }
    out.push('\n');
    out
}

fn parse_cached(text: &str) -> Vec<PageText> {
    text.split(PAGE_MARKER)
        .skip(1)
        .filter_map(|chunk| {
            let (number, body) = chunk.split_once(" =====")?;
            let page = number.trim().parse().ok()?;
            let text = body.trim().to_string();
            Some(PageText {
                page,
                chars: text.chars().filter(|c| !c.is_whitespace()).count(),
                text,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cached_pages_round_trip() {
        let pages = vec![
            PageText {
                page: 1,
                chars: 3,
                text: "abc".to_string(),
            },
            PageText {
                page: 2,
                chars: 2,
                text: "de".to_string(),
            },
        ];
        let text = render_cached(Path::new("PD0.pdf"), &pages);
        let parsed = parse_cached(&text);
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].page, 1);
        assert_eq!(parsed[1].text, "de");
    }
}
