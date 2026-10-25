use std::sync::LazyLock;

use regex::Regex;

/// Pages below this many non-whitespace characters are low-text.
pub const LOW_TEXT_THRESHOLD: usize = 120;

static HEADING_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?P<num>\d+(?:\.\d+){0,3})\.?\s+(?P<title>[A-ZÇĞİÖŞÜ][^\n]{2,150})$")
        .expect("valid heading regex")
});
static CHAPTER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(?:BÖLÜM|Bölüm)\s+\d+\b.*$").expect("valid chapter regex"));
static PAGE_NO_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*\d{1,4}\s*$").expect("valid page regex"));
static BULLET_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[\s•●▪◦\-–—*]+").expect("valid bullet regex"));

/// Normalize extracted page text without changing its line order.
pub fn clean_text(text: &str) -> String {
    let text = text.replace('\u{000C}', "\n").replace('\u{00A0}', " ");
    let lines: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    lines.join("\n")
}

/// Lines that can carry a heading or a page-map entry.
pub fn meaningful_lines(text: &str) -> Vec<String> {
    clean_text(text)
        .lines()
        .map(str::trim)
        .filter(|line| {
            !line.is_empty()
                && !PAGE_NO_RE.is_match(line)
                && !line.starts_with("http://")
                && !line.starts_with("https://")
        })
        .map(ToOwned::to_owned)
        .collect()
}

/// Extract the first numbered/chapter-like heading from a page.
///
/// Leading slide bullets are ignored so that liteparse's projected text
/// (`• 0.1. Giriş`) matches the layout-based inventory (`0.1. Giriş`).
pub fn heading_of(lines: &[String]) -> Option<String> {
    for line in lines.iter().take(8) {
        let candidate = BULLET_RE.replace(line, "");
        let candidate = candidate.trim();
        if CHAPTER_RE.is_match(candidate) {
            return Some(candidate.to_string());
        }
        let Some(captures) = HEADING_RE.captures(candidate) else {
            continue;
        };
        let number = captures.name("num")?.as_str();
        let title = captures.name("title")?.as_str().trim();
        let word_count = title.split_whitespace().count();
        if word_count <= 14
            && !title.ends_with('.')
            && !title.ends_with(':')
            && !title.ends_with(';')
        {
            return Some(format!("{number} {title}"));
        }
    }
    None
}

/// First meaningful line, truncated to 120 characters.
pub fn first_line(lines: &[String]) -> String {
    for line in lines.iter().take(6) {
        if line.chars().count() >= 3 {
            return line.chars().take(120).collect();
        }
    }
    String::new()
}

/// Non-whitespace character count used by the low-text rule.
pub fn count_chars(text: &str) -> usize {
    text.chars()
        .filter(|character| !character.is_whitespace())
        .count()
}

/// Whitespace-separated word count.
pub fn count_words(text: &str) -> usize {
    text.split_whitespace().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleans_form_feeds_and_blank_lines() {
        assert_eq!(clean_text("  A  \n\n\u{000C} B \n"), "A\nB");
    }

    #[test]
    fn ignores_bullets_before_a_heading() {
        let lines = meaningful_lines("• 0.1. Giriş ve Amaçlar\nMetin");
        assert_eq!(heading_of(&lines).as_deref(), Some("0.1 Giriş ve Amaçlar"));
    }

    #[test]
    fn accepts_chapter_headings() {
        let lines = meaningful_lines("Bölüm 1: Giriş\nMetin");
        assert_eq!(heading_of(&lines).as_deref(), Some("Bölüm 1: Giriş"));
    }

    #[test]
    fn rejects_sentence_like_lines() {
        let lines = meaningful_lines("1 Bu bir cümledir.\nMetin");
        assert_eq!(heading_of(&lines), None);
    }

    #[test]
    fn keeps_bullets_in_the_first_line() {
        let lines = meaningful_lines("• Şu ana kadar 2500 dil.\nMetin");
        assert_eq!(first_line(&lines), "• Şu ana kadar 2500 dil.");
    }

    #[test]
    fn counts_non_whitespace_characters() {
        assert_eq!(count_chars("a b\nc"), 3);
    }
}
