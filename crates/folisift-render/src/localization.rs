//! Localization guard for generated guides.
//!
//! The rule is intentionally narrow: an English guide must not leak Turkish
//! engine labels (`Şekil`, `Tablo`, `İÇİNDEKİLER`, ...). Guide prose may still
//! quote Turkish source terms, so only known UI labels are rejected.

use std::sync::LazyLock;

use regex::Regex;

use crate::error::{Error, Result};

static TURKISH_UI: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)(?:\bİÇİNDEKİLER\b|\bKapsam denetimi\b|\bŞekil\b|\bTablo\b|\bBölüm\b|\bKısaltmalar\b|\bKaynak:\s*\[)",
    )
    .expect("valid localization regex")
});

pub fn lint(markdown: &str, language: &str) -> Result<()> {
    if language != "en" {
        return Ok(());
    }
    let mut fenced = false;
    for (index, line) in markdown.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            fenced = !fenced;
            continue;
        }
        if fenced {
            continue;
        }
        if let Some(found) = TURKISH_UI.find(line) {
            return Err(Error::Markdown {
                line: index + 1,
                message: format!(
                    "Turkish UI label {:?} in an English guide; localize it or set language=tr",
                    found.as_str()
                ),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_turkish_ui_in_english() {
        let error = lint("## Şekil 1\n", "en").expect_err("must reject");
        assert!(error.to_string().contains("Şekil"));
    }

    #[test]
    fn allows_turkish_ui_in_turkish_and_quotes_in_code() {
        assert!(lint("## Şekil 1\n", "tr").is_ok());
        assert!(lint("```\nŞekil\n```\n", "en").is_ok());
    }
}
