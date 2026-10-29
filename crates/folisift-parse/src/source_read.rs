use serde::Serialize;

use crate::error::{Error, Result};
use crate::parser::parse_page_list;
use crate::source_cache::{PageText, SourceCache, SourceText};
use crate::text::LOW_TEXT_THRESHOLD;

pub const DEFAULT_READ_PAGES: &str = "1-20";
pub const DEFAULT_MAX_CHARS: usize = 24_000;
pub const HARD_MAX_CHARS: usize = 60_000;
pub const DEFAULT_SEARCH_LIMIT: usize = 20;
pub const HARD_SEARCH_LIMIT: usize = 50;

#[derive(Debug, Clone, Serialize)]
pub struct ReadSourceResult {
    pub source: String,
    pub pages: Vec<PageText>,
    pub returned_pages: usize,
    pub chars: usize,
    pub max_chars: usize,
    pub truncated: bool,
    pub next_pages: Option<String>,
    pub cache_hit: bool,
    pub hint: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchHit {
    pub page: usize,
    pub snippet: String,
    pub score: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchSourceResult {
    pub source: String,
    pub query: String,
    pub total: usize,
    pub returned: usize,
    pub hits: Vec<SearchHit>,
    pub searched_pages: usize,
    pub low_text_pages: Vec<usize>,
    pub low_text_total: usize,
    pub low_text_truncated: bool,
    pub truncated: bool,
    pub cache_hit: bool,
    pub hint: Option<String>,
}

pub async fn read_source(
    cache: &SourceCache,
    source: &std::path::Path,
    pages: Option<&str>,
    max_chars: Option<usize>,
) -> Result<ReadSourceResult> {
    let source_text = cache.load(source, false).await?;
    read_from_source(&source_text, pages, max_chars)
}

pub fn read_from_source(
    source: &SourceText,
    pages: Option<&str>,
    max_chars: Option<usize>,
) -> Result<ReadSourceResult> {
    let explicit = pages.is_some();
    let selection = pages.unwrap_or(DEFAULT_READ_PAGES);
    let mut requested = parse_page_list(selection)?;
    let max_chars = max_chars
        .unwrap_or(DEFAULT_MAX_CHARS)
        .clamp(1_000, HARD_MAX_CHARS);
    let page_count = source.pages.len();

    if explicit && requested.iter().any(|page| *page as usize > page_count) {
        return Err(Error::PageSelection {
            value: selection.to_string(),
        });
    }
    requested.retain(|page| *page as usize <= page_count);
    if requested.is_empty() {
        return Err(Error::PageSelection {
            value: selection.to_string(),
        });
    }

    let mut selected = Vec::new();
    let mut used = 0;
    let mut truncated = false;
    let mut next_pages = None;

    for page_number in requested {
        let Some(page) = source
            .pages
            .iter()
            .find(|page| page.page == page_number as usize)
        else {
            continue;
        };
        let overhead = format!("### Page {}\\n", page.page).len();
        let remaining = max_chars.saturating_sub(used + overhead);
        if remaining < 200 && !selected.is_empty() {
            truncated = true;
            next_pages = Some(format!("{page_number}"));
            break;
        }
        let text = if page.text.chars().count() > remaining {
            truncated = true;
            next_pages = Some(format!("{page_number}"));
            truncate_chars(&page.text, remaining.saturating_sub(1).max(1))
        } else {
            page.text.clone()
        };
        let chars = text
            .chars()
            .filter(|character| !character.is_whitespace())
            .count();
        used += overhead + text.chars().count();
        selected.push(PageText {
            page: page.page,
            chars,
            text,
        });
        if truncated {
            break;
        }
    }

    let hint = next_pages
        .as_ref()
        .map(|next| format!("Call read_source with pages={next}."));
    Ok(ReadSourceResult {
        source: source.source.clone(),
        returned_pages: selected.len(),
        pages: selected,
        chars: used,
        max_chars,
        truncated,
        next_pages,
        cache_hit: source.cache_hit,
        hint,
    })
}

pub async fn search_source(
    cache: &SourceCache,
    source: &std::path::Path,
    query: &str,
    limit: Option<usize>,
    context_chars: Option<usize>,
) -> Result<SearchSourceResult> {
    let source_text = cache.load(source, false).await?;
    search_from_source(&source_text, query, limit, context_chars)
}

pub fn search_from_source(
    source: &SourceText,
    query: &str,
    limit: Option<usize>,
    context_chars: Option<usize>,
) -> Result<SearchSourceResult> {
    let query = query.trim();
    if query.is_empty() {
        return Err(Error::PageSelection {
            value: "empty search query".to_string(),
        });
    }
    let limit = limit
        .unwrap_or(DEFAULT_SEARCH_LIMIT)
        .clamp(1, HARD_SEARCH_LIMIT);
    let context_chars = context_chars.unwrap_or(200).clamp(40, 1_000);
    let normalized_query = normalize(query);
    let terms = normalized_query
        .split_whitespace()
        .filter(|term| term.len() >= 2)
        .collect::<Vec<_>>();

    let mut hits = Vec::new();
    let mut total = 0;
    for page in &source.pages {
        let normalized = normalize(&page.text);
        let score = terms
            .iter()
            .filter(|term| normalized.contains(**term))
            .count();
        if score == 0 {
            continue;
        }
        total += 1;
        if hits.len() >= limit {
            continue;
        }
        hits.push(SearchHit {
            page: page.page,
            snippet: snippet(&page.text, &terms, context_chars),
            score,
        });
    }

    let all_low_text_pages = source
        .pages
        .iter()
        .filter(|page| page.chars < LOW_TEXT_THRESHOLD)
        .map(|page| page.page)
        .collect::<Vec<_>>();
    let low_text_total = all_low_text_pages.len();
    let low_text_pages = all_low_text_pages
        .iter()
        .copied()
        .take(40)
        .collect::<Vec<_>>();
    let low_text_truncated = low_text_total > low_text_pages.len();
    let truncated = total > hits.len();
    let hint = if truncated {
        Some(format!(
            "Returned {limit} of {total} pages. Narrow the query or call read_source for a page."
        ))
    } else if hits.is_empty() {
        Some(
            "No text hits. Check low_text_pages and render those pages for visual evidence."
                .to_string(),
        )
    } else {
        None
    };

    Ok(SearchSourceResult {
        source: source.source.clone(),
        query: query.to_string(),
        total,
        returned: hits.len(),
        hits,
        searched_pages: source.pages.len(),
        low_text_pages,
        low_text_total,
        low_text_truncated,
        truncated,
        cache_hit: source.cache_hit,
        hint,
    })
}

fn normalize(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for character in text.chars() {
        let character = match character {
            'ı' | 'İ' | 'I' => 'i',
            other => other.to_lowercase().next().unwrap_or(other),
        };
        if character.is_alphanumeric() || character.is_whitespace() {
            out.push(character);
        } else {
            out.push(' ');
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn snippet(text: &str, terms: &[&str], context_chars: usize) -> String {
    let normalized = normalize(text);
    let Some(first) = terms.iter().filter_map(|term| normalized.find(term)).min() else {
        return truncate_chars(text, context_chars);
    };
    let first = normalized[..first].chars().count();
    let chars: Vec<char> = text.chars().collect();
    let start = first.saturating_sub(context_chars / 2);
    let end = (first + context_chars).min(chars.len());
    let mut snippet = chars[start..end].iter().collect::<String>();
    if start > 0 {
        snippet.insert(0, '…');
    }
    if end < chars.len() {
        snippet.push('…');
    }
    snippet
}

fn truncate_chars(text: &str, limit: usize) -> String {
    let mut out = text.chars().take(limit).collect::<String>();
    if out.chars().count() < text.chars().count() {
        out.push('…');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source() -> SourceText {
        SourceText {
            source: "PD0.pdf".to_string(),
            sha256: "hash".to_string(),
            pages: vec![
                PageText {
                    page: 1,
                    chars: 20,
                    text: "Church-Turing hipotezi".to_string(),
                },
                PageText {
                    page: 2,
                    chars: 20,
                    text: "TIOBE indeksi Java".to_string(),
                },
            ],
            cache_path: "source.txt".into(),
            cache_hit: true,
        }
    }

    #[test]
    fn reads_a_page_range() {
        let result = read_from_source(&source(), Some("1"), None).expect("read");
        assert_eq!(result.returned_pages, 1);
        assert!(result.pages[0].text.contains("Church"));
    }

    #[test]
    fn searches_with_page_numbers() {
        let result = search_from_source(&source(), "tiobe", None, None).expect("search");
        assert_eq!(result.total, 1);
        assert_eq!(result.hits[0].page, 2);
    }

    #[test]
    fn rejects_out_of_range_pages() {
        assert!(read_from_source(&source(), Some("9"), None).is_err());
    }

    #[test]
    fn counts_characters_not_bytes() {
        let mut source = source();
        source.pages[0].text = "ğ".repeat(1_200);
        let result = read_from_source(&source, Some("1"), Some(1_000)).expect("read");
        assert!(result.truncated);
        assert!(result.chars <= 1_000);
    }

    #[test]
    fn default_range_clamps_to_a_short_document() {
        let result = read_from_source(&source(), None, None).expect("read");
        assert_eq!(result.returned_pages, 2);
        assert!(!result.truncated);
    }
}
