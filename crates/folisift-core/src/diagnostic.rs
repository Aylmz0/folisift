//! Machine-readable diagnostics for host integrations.
//!
//! The engine still returns human-readable errors; this module maps them to a
//! stable code and bounded JSON envelope so a host can branch without parsing
//! prose. Unknown messages are preserved, never guessed as success.

use serde::Serialize;
use serde_json::{Value, json};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    PageBudget,
    Underfilled,
    Overflow,
    Structure,
    Coverage,
    Markdown,
    Visual,
    Diagram,
    ChapterSplit,
    TableWidths,
    DiagramWidth,
    DiagramPageSpan,
    Io,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Diagnostic {
    pub code: ErrorCode,
    pub message: String,
    pub line: Option<usize>,
    pub page: Option<usize>,
    pub block_id: Option<String>,
    pub directive_stack: Vec<String>,
    pub details: Value,
    pub suggestions: Vec<String>,
}

impl Diagnostic {
    pub fn from_message(message: &str) -> Self {
        let lower = message.to_lowercase();
        let code = if lower.contains("e_table_widths") || lower.contains("widths must sum") {
            ErrorCode::TableWidths
        } else if lower.contains("e_diagram_width") || lower.contains("diagram width") {
            ErrorCode::DiagramWidth
        } else if lower.contains("e_diagram_page_span") {
            ErrorCode::DiagramPageSpan
        } else if lower.contains("underfilled") {
            ErrorCode::Underfilled
        } else if lower.contains("page count")
            || lower.contains("target pages")
            || lower.contains("page range")
        {
            ErrorCode::PageBudget
        } else if lower.contains("coverage gate failed")
            || lower.contains("missing") && lower.contains("heading")
        {
            ErrorCode::Coverage
        } else if lower.contains("structure") && lower.contains("failed") {
            ErrorCode::Structure
        } else if lower.contains("e_markdown")
            || lower.contains("markdown")
            || lower.contains("directive")
        {
            ErrorCode::Markdown
        } else if lower.contains("diagram") {
            ErrorCode::Diagram
        } else if lower.contains("visual audit") {
            ErrorCode::Visual
        } else if lower.contains("chapter") && lower.contains("split") {
            ErrorCode::ChapterSplit
        } else if lower.contains("outside")
            || lower.contains("overflow")
            || lower.contains("lines above")
        {
            ErrorCode::Overflow
        } else if lower.contains("i/o error") || lower.contains("no such file") {
            ErrorCode::Io
        } else {
            ErrorCode::Unknown
        };
        let line = message
            .split("line ")
            .nth(1)
            .and_then(|value| {
                value
                    .split(|character: char| !character.is_ascii_digit())
                    .next()
            })
            .and_then(|value| value.parse().ok());
        let page = message
            .split("pages: [")
            .nth(1)
            .and_then(|value| {
                value
                    .split(|character: char| !character.is_ascii_digit())
                    .next()
            })
            .and_then(|value| value.parse().ok())
            .or_else(|| {
                message
                    .split("page ")
                    .nth(1)
                    .and_then(|value| {
                        value
                            .split(|character: char| !character.is_ascii_digit())
                            .next()
                    })
                    .and_then(|value| value.parse().ok())
            });
        let suggestions = match code {
            ErrorCode::Underfilled => {
                vec!["move a figure/table or cut the largest section".to_string()]
            }
            ErrorCode::Coverage => {
                vec!["add the missing section or update the reviewed manifest".to_string()]
            }
            ErrorCode::TableWidths => vec!["make the widths list sum to 100%".to_string()],
            ErrorCode::Structure => {
                vec!["use the heading level declared by the structure pack".to_string()]
            }
            ErrorCode::PageBudget => vec!["run `folisift budget --guide <slug>`".to_string()],
            _ => Vec::new(),
        };
        Self {
            code,
            message: message.to_string(),
            line,
            page,
            block_id: None,
            directive_stack: Vec::new(),
            details: json!({}),
            suggestions,
        }
    }

    pub fn envelope(&self) -> Value {
        json!({
            "ok": false,
            "code": self.code,
            "diagnostic": self,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_widths_map_to_a_stable_code_and_suggestion() {
        let diagnostic = Diagnostic::from_message(
            "E_TABLE_WIDTHS: table widths (2 columns) sum to 62%, expected 100%",
        );
        assert_eq!(diagnostic.code, ErrorCode::TableWidths);
        assert!(diagnostic.suggestions[0].contains("100%"));
        assert_eq!(diagnostic.envelope()["ok"], false);
    }

    #[test]
    fn markdown_errors_map_to_markdown_with_a_line() {
        let diagnostic =
            Diagnostic::from_message("E_MARKDOWN at line 196: diagram \"x\" has no built asset");
        assert_eq!(diagnostic.code, ErrorCode::Markdown);
        assert_eq!(diagnostic.line, Some(196));
    }

    #[test]
    fn underfilled_pages_and_lines_are_extracted() {
        let diagnostic = Diagnostic::from_message("underfilled body pages: [17] on line 42");
        assert_eq!(diagnostic.code, ErrorCode::Underfilled);
        assert_eq!(diagnostic.page, Some(17));
        assert_eq!(diagnostic.line, Some(42));
        assert!(!diagnostic.suggestions.is_empty());
    }
}
