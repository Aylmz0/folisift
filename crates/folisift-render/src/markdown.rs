use std::sync::LazyLock;

use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use regex::Regex;
use serde::Deserialize;

use crate::error::{Error, Result};

static SOURCE_REF_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[(PD\d+[^\]]*s\.[^\]]*)\]").expect("valid source reference regex")
});

static SOURCE_REF_MARKER_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new("\u{E000}([^\u{E001}]*)\u{E001}").expect("valid source reference marker regex")
});

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Frontmatter {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub course: Option<String>,
    #[serde(default)]
    pub sources: Vec<String>,
    #[serde(default)]
    pub language: Option<String>,
}

/// Split YAML frontmatter from the Markdown body.
pub fn parse_document(input: &str) -> Result<(Frontmatter, String)> {
    let input = input.replace("\r\n", "\n");
    let Some(rest) = input.strip_prefix("---\n") else {
        return Ok((Frontmatter::default(), input));
    };
    let Some((yaml, body)) = rest.split_once("\n---\n") else {
        return Err(Error::Frontmatter(
            "opening --- has no closing ---".to_string(),
        ));
    };
    let frontmatter =
        serde_yaml::from_str(yaml).map_err(|error| Error::Frontmatter(error.to_string()))?;
    Ok((frontmatter, body.to_string()))
}

/// Convert guide Markdown to Typst content.
pub fn markdown_to_typst(markdown: &str) -> Result<String> {
    let lines: Vec<&str> = markdown.lines().collect();
    let mut out = String::new();
    let mut index = 0;

    while index < lines.len() {
        if let Some(kind) = directive_start(lines[index]) {
            let start_line = index + 1;
            let mut end = index + 1;
            let mut inner = Vec::new();
            while end < lines.len() && lines[end].trim() != ":::" {
                inner.push(lines[end]);
                end += 1;
            }
            if end >= lines.len() {
                return Err(Error::Markdown {
                    line: start_line,
                    message: format!("unclosed directive ::: {kind}"),
                });
            }
            if !matches!(kind, "definition" | "critical" | "example" | "note") {
                return Err(Error::Markdown {
                    line: start_line,
                    message: format!("unknown directive ::: {kind}"),
                });
            }
            let inner_typst = markdown_to_typst(&inner.join("\n"))?;
            out.push_str(&format!("#{kind}[{inner_typst}]\n\n"));
            index = end + 1;
            continue;
        }

        let start = index;
        while index < lines.len() && directive_start(lines[index]).is_none() {
            index += 1;
        }
        let segment = lines[start..index].join("\n");
        out.push_str(&render_segment(&segment)?);
    }

    Ok(out)
}

fn directive_start(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    let rest = trimmed.strip_prefix(":::")?;
    let kind = rest.trim();
    (!kind.is_empty()).then_some(kind)
}

fn render_segment(markdown: &str) -> Result<String> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let markdown = markdown
        .lines()
        .map(|line| {
            if line.trim_start().starts_with("![") {
                line.to_string()
            } else {
                SOURCE_REF_RE
                    .replace_all(line, "\u{E000}$1\u{E001}")
                    .to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    let parser = Parser::new_ext(&markdown, options);
    let mut renderer = Renderer::default();
    for event in parser {
        renderer.event(event)?;
    }
    Ok(renderer.finish())
}

#[derive(Default)]
struct Renderer {
    out: String,
    list_stack: Vec<bool>,
    in_item: bool,
    table: Option<TableState>,
    image: Option<ImageState>,
    code: Option<String>,
    code_lang: String,
}

#[derive(Default)]
struct TableState {
    header: Option<Vec<String>>,
    rows: Vec<Vec<String>>,
    current_row: Vec<String>,
    cell: Option<String>,
    in_header: bool,
}

struct ImageState {
    dest: String,
    alt: String,
}

impl Renderer {
    fn event(&mut self, event: Event<'_>) -> Result<()> {
        match event {
            Event::Start(Tag::Paragraph) => {
                if self.table.is_none() && !self.in_item {
                    self.out.push_str("\n\n");
                }
            }
            Event::End(TagEnd::Paragraph) => {
                if self.table.is_none() && !self.in_item {
                    self.out.push('\n');
                }
            }
            Event::Start(Tag::Heading { level, .. }) => {
                if self.table.is_none() {
                    self.out.push_str("\n\n");
                    self.out.push_str(&"=".repeat(heading_level(level)));
                    self.out.push(' ');
                }
            }
            Event::End(TagEnd::Heading(_)) => {
                if self.table.is_none() {
                    self.out.push_str("\n\n");
                }
            }
            Event::Start(Tag::Strong) => self.push("#strong["),
            Event::End(TagEnd::Strong) => self.push("]"),
            Event::Start(Tag::Emphasis) => self.push("#emph["),
            Event::End(TagEnd::Emphasis) => self.push("]"),
            Event::Start(Tag::Strikethrough) => self.push("#strike["),
            Event::End(TagEnd::Strikethrough) => self.push("]"),
            Event::Start(Tag::List(ordered)) => {
                self.list_stack.push(ordered.is_some());
                if self.table.is_none() {
                    self.out.push('\n');
                }
            }
            Event::End(TagEnd::List(_)) => {
                self.list_stack.pop();
                if self.table.is_none() {
                    self.out.push('\n');
                }
            }
            Event::Start(Tag::Item) => {
                self.in_item = true;
                if self.table.is_none() {
                    self.out.push('\n');
                    for _ in 1..self.list_stack.len() {
                        self.out.push_str("  ");
                    }
                    let ordered = self.list_stack.last().copied().unwrap_or(false);
                    self.out.push_str(if ordered { "+ " } else { "- " });
                }
            }
            Event::End(TagEnd::Item) => {
                self.in_item = false;
                if self.table.is_none() {
                    self.out.push('\n');
                }
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                self.code = Some(String::new());
                self.code_lang = match kind {
                    CodeBlockKind::Fenced(language) => language.to_string(),
                    CodeBlockKind::Indented => String::new(),
                };
            }
            Event::End(TagEnd::CodeBlock) => {
                if let Some(code) = self.code.take() {
                    let mut call = format!("#raw({}, block: true", typst_string(&code));
                    if !self.code_lang.is_empty() {
                        call.push_str(&format!(", lang: {}", typst_string(&self.code_lang)));
                    }
                    call.push(')');
                    self.out.push_str("\n\n");
                    self.out.push_str(&call);
                    self.out.push_str("\n\n");
                }
            }
            Event::Start(Tag::Table(_)) => {
                self.out.push_str("\n\n");
                self.table = Some(TableState::default());
            }
            Event::End(TagEnd::Table) => {
                if let Some(table) = self.table.take() {
                    self.emit_table(table);
                }
            }
            Event::Start(Tag::TableHead) => {
                if let Some(table) = &mut self.table {
                    table.in_header = true;
                    table.current_row.clear();
                }
            }
            Event::End(TagEnd::TableHead) => {
                if let Some(table) = &mut self.table {
                    let row = std::mem::take(&mut table.current_row);
                    if !row.is_empty() {
                        table.header = Some(row);
                    }
                    table.in_header = false;
                }
            }
            Event::Start(Tag::TableRow) => {
                if let Some(table) = &mut self.table {
                    table.current_row.clear();
                }
            }
            Event::End(TagEnd::TableRow) => {
                if let Some(table) = &mut self.table {
                    let row = std::mem::take(&mut table.current_row);
                    if table.in_header {
                        table.header = Some(row);
                    } else {
                        table.rows.push(row);
                    }
                }
            }
            Event::Start(Tag::TableCell) => {
                if let Some(table) = &mut self.table {
                    table.cell = Some(String::new());
                }
            }
            Event::End(TagEnd::TableCell) => {
                if let Some(table) = &mut self.table
                    && let Some(cell) = table.cell.take()
                {
                    table.current_row.push(cell);
                }
            }
            Event::Start(Tag::Image { dest_url, .. }) => {
                self.image = Some(ImageState {
                    dest: dest_url.to_string(),
                    alt: String::new(),
                });
            }
            Event::End(TagEnd::Image) => {
                if let Some(image) = self.image.take() {
                    self.push("#figure(");
                    self.push(&format!(
                        "image({}, width: 100%)",
                        typst_string(&image.dest)
                    ));
                    self.push(", caption: [");
                    self.push(&format!("#text({})", typst_string(&image.alt)));
                    self.push("], numbering: none)\n\n");
                }
            }
            Event::Start(Tag::Link { dest_url, .. }) => {
                self.push(&format!("#link({})[", typst_string(&dest_url)));
            }
            Event::End(TagEnd::Link) => self.push("]"),
            Event::Start(Tag::BlockQuote(_)) => self.push("#quote["),
            Event::End(TagEnd::BlockQuote(_)) => self.push("]\n\n"),
            Event::Text(text) => {
                if let Some(code) = &mut self.code {
                    code.push_str(&text);
                } else if let Some(image) = &mut self.image {
                    image.alt.push_str(&text);
                } else {
                    self.push_text(&text);
                }
            }
            Event::Code(code) => self.push(&format!("#raw({})", typst_string(&code))),
            Event::SoftBreak => self.push(" "),
            Event::HardBreak => self.push(" #linebreak() "),
            Event::Rule => self.push("\n\n#line(length: 100%, stroke: 0.5pt)\n\n"),
            Event::Html(_)
            | Event::InlineHtml(_)
            | Event::FootnoteReference(_)
            | Event::TaskListMarker(_)
            | Event::InlineMath(_)
            | Event::DisplayMath(_) => {}
            Event::Start(_) | Event::End(_) => {}
        }
        Ok(())
    }

    fn finish(mut self) -> String {
        if let Some(table) = self.table.take() {
            self.emit_table(table);
        }
        self.out
    }

    fn push(&mut self, value: &str) {
        if let Some(table) = &mut self.table
            && let Some(cell) = &mut table.cell
        {
            cell.push_str(value);
        } else {
            self.out.push_str(value);
        }
    }

    fn push_text(&mut self, text: &str) {
        let mut last = 0;
        for capture in SOURCE_REF_MARKER_RE.captures_iter(text) {
            let Some(full) = capture.get(0) else {
                continue;
            };
            self.push_plain(&text[last..full.start()]);
            let body = capture.get(1).map_or("", |value| value.as_str());
            self.push(&format!("#source_ref({})", typst_string(body)));
            last = full.end();
        }
        self.push_plain(&text[last..]);
    }

    fn push_plain(&mut self, text: &str) {
        let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
        if collapsed.is_empty() {
            if !text.is_empty() {
                self.push(" ");
            }
            return;
        }
        let leading = text.chars().next().is_some_and(char::is_whitespace);
        let trailing = text.chars().last().is_some_and(char::is_whitespace);
        if leading {
            self.push(" ");
        }
        self.push(&format!("#text({})", typst_string(&collapsed)));
        if trailing {
            self.push(" ");
        }
    }

    fn emit_table(&mut self, table: TableState) {
        let header = table.header.unwrap_or_default();
        let rows = table.rows;
        self.out.push_str(&table_typst(&header, &rows));
        self.out.push_str("\n\n");
    }
}

fn table_typst(header: &[String], rows: &[Vec<String>]) -> String {
    let columns = header
        .len()
        .max(rows.iter().map(Vec::len).max().unwrap_or(0))
        .max(1);
    let mut out = String::from("#table(\n");
    out.push_str(&format!("  columns: {columns},\n"));
    out.push_str("  stroke: none,\n");
    out.push_str("  align: left,\n");
    if !header.is_empty() {
        out.push_str("  table.header(");
        for (index, cell) in header.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            out.push_str("[#strong[");
            out.push_str(cell);
            out.push_str("]]");
        }
        out.push_str("),\n");
    }
    for row in rows {
        for index in 0..columns {
            let cell = row.get(index).map_or("", String::as_str);
            out.push('[');
            out.push_str(cell);
            out.push_str("], ");
        }
        out.push('\n');
    }
    out.push(')');
    out
}

fn heading_level(level: HeadingLevel) -> usize {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn typst_string(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_frontmatter() {
        let (front, body) =
            parse_document("---\ntitle: Demo\nsources: [PD0]\n---\n\n# Hi\n").expect("frontmatter");
        assert_eq!(front.title.as_deref(), Some("Demo"));
        assert_eq!(front.sources, vec!["PD0"]);
        assert!(body.contains("# Hi"));
    }

    #[test]
    fn renders_directives() {
        let typst = markdown_to_typst("::: definition\n**Tanım**\n:::\n").expect("typst");
        assert!(typst.contains("#definition["));
        assert!(typst.contains("#strong["));
    }

    #[test]
    fn renders_source_refs() {
        let typst = markdown_to_typst("Metin [PD0, s. 3].").expect("typst");
        assert!(typst.contains("#source_ref(\"PD0, s. 3\")"));
    }

    #[test]
    fn renders_tables() {
        let typst = markdown_to_typst("| A | B |\n|---|---|\n| 1 | 2 |\n").expect("typst");
        assert!(typst.contains("#table("));
        assert!(typst.contains("table.header("));
    }

    #[test]
    fn rejects_unknown_directives() {
        let error = markdown_to_typst("::: mystery\nx\n:::\n").expect_err("must reject");
        assert!(matches!(error, Error::Markdown { .. }));
    }
}
