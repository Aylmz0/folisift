use std::collections::BTreeMap;
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

static FIGURE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?:Şekil|Figure|Figür)\s+([0-9]+(?:\.[0-9]+)*)\s*(.*)$")
        .expect("valid figure caption regex")
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
    #[serde(default)]
    pub running: Option<String>,
    #[serde(default)]
    pub pilot: Option<String>,
    #[serde(default)]
    pub deck: Option<String>,
    #[serde(default)]
    pub cover_kicker: Option<String>,
    #[serde(default)]
    pub cover_title: Option<String>,
    #[serde(default)]
    pub cover_motif: Option<String>,
    #[serde(default)]
    pub cover_notes: Vec<String>,
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

#[derive(Debug, Clone, Default)]
struct RenderConfig {
    table_widths: Option<Vec<String>>,
    table_kind: Option<String>,
}

/// Convert guide Markdown to Typst content.
pub fn markdown_to_typst(markdown: &str) -> Result<String> {
    markdown_to_typst_with(markdown, &RenderConfig::default())
}

fn markdown_to_typst_with(markdown: &str, config: &RenderConfig) -> Result<String> {
    let lines: Vec<&str> = markdown.lines().collect();
    let mut out = String::new();
    let mut index = 0;

    while index < lines.len() {
        if lines[index].trim() == ":::" {
            return Err(Error::Markdown {
                line: index + 1,
                message: "closing ::: without an opening directive".to_string(),
            });
        }
        if let Some((kind, rest)) = directive_start(lines[index]) {
            let start_line = index + 1;
            let attrs = parse_attrs(rest).map_err(|message| Error::Markdown {
                line: start_line,
                message,
            })?;

            if kind == "pagebreak" || kind == "columnbreak" {
                if kind == "pagebreak" {
                    if !rest.trim().is_empty() {
                        return Err(Error::Markdown {
                            line: start_line,
                            message: "pagebreak takes no attributes".to_string(),
                        });
                    }
                    out.push_str("\n#pagebreak()\n\n");
                    index += 1;
                    continue;
                }
                if !rest.trim().is_empty() {
                    return Err(Error::Markdown {
                        line: start_line,
                        message: "columnbreak takes no attributes".to_string(),
                    });
                }
                out.push_str("\n#colbreak()\n\n");
                index += 1;
                continue;
            }

            let mut end = index + 1;
            let mut inner = Vec::new();
            let mut depth = 0usize;
            while end < lines.len() {
                if lines[end].trim() == ":::" {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                } else if let Some((nested, _)) = directive_start(lines[end])
                    && nested != "columnbreak"
                {
                    depth += 1;
                }
                inner.push(lines[end]);
                end += 1;
            }
            if end >= lines.len() {
                return Err(Error::Markdown {
                    line: start_line,
                    message: format!("unclosed directive ::: {kind}"),
                });
            }
            let inner_markdown = inner.join("\n");

            match kind {
                "definition" | "critical" | "example" | "note" => {
                    let inner_typst = markdown_to_typst_with(&inner_markdown, config)?;
                    if let Some(part) = attr(&attrs, "toc") {
                        let label = attr(&attrs, "label").unwrap_or(kind);
                        out.push_str(&format!(
                            "#toc_entry(level: 1, body: [#text({})], part: {})\n\n",
                            typst_string(label),
                            typst_string(part),
                        ));
                    }
                    match attr(&attrs, "label") {
                        Some(label) => out.push_str(&format!(
                            "#{kind}(label: {})[{inner_typst}]\n\n",
                            typst_string(label)
                        )),
                        None => out.push_str(&format!("#{kind}[{inner_typst}]\n\n")),
                    }
                }
                "chapter" => {
                    if !inner_markdown.trim().is_empty() {
                        return Err(Error::Markdown {
                            line: start_line,
                            message: "chapter is an opener; put content after the directive"
                                .to_string(),
                        });
                    }
                    out.push_str(&chapter_call(&attrs));
                    out.push_str("\n\n");
                }
                "front" => {
                    let inner_typst = markdown_to_typst_with(&inner_markdown, config)?;
                    out.push_str(&format!(
                        "#front(kicker: {}, title: {}, lead: {})[{inner_typst}]\n\n",
                        typst_string(attr(&attrs, "kicker").unwrap_or("")),
                        typst_string(attr(&attrs, "title").unwrap_or("")),
                        optional_attr(&attrs, "lead"),
                    ));
                }
                "toc" => {
                    if !inner_markdown.trim().is_empty() {
                        return Err(Error::Markdown {
                            line: start_line,
                            message: "toc takes no body".to_string(),
                        });
                    }
                    out.push_str(&format!(
                        "#toc_page(title: {}, note: {})\n\n",
                        typst_string(attr(&attrs, "title").unwrap_or("")),
                        typst_string(attr(&attrs, "note").unwrap_or("")),
                    ));
                }
                "columns" => {
                    let inner_typst = markdown_to_typst_with(&inner_markdown, config)?;
                    out.push_str(&format!("#two_col[{inner_typst}]\n\n"));
                }
                "table" => {
                    let mut table_config = config.clone();
                    table_config.table_widths = match attr(&attrs, "widths") {
                        Some(widths) => Some(parse_widths(widths, start_line)?),
                        None => None,
                    };
                    table_config.table_kind = attr(&attrs, "kind").map(ToString::to_string);
                    out.push_str(&markdown_to_typst_with(&inner_markdown, &table_config)?);
                }
                other => {
                    return Err(Error::Markdown {
                        line: start_line,
                        message: format!("unknown directive ::: {other}"),
                    });
                }
            }
            index = end + 1;
            continue;
        }

        let start = index;
        while index < lines.len()
            && lines[index].trim() != ":::"
            && directive_start(lines[index]).is_none()
        {
            index += 1;
        }
        let segment = lines[start..index].join("\n");
        out.push_str(&render_segment(&segment, config)?);
    }

    Ok(out)
}

fn chapter_call(attrs: &BTreeMap<String, String>) -> String {
    let mut args = Vec::new();
    for (key, default) in [
        ("part", ""),
        ("side", ""),
        ("kicker", ""),
        ("title", ""),
        ("running", ""),
    ] {
        let value = attr(attrs, key).unwrap_or(default);
        args.push(format!("{key}: {}", typst_string(value)));
    }
    let title = attr(attrs, "title").unwrap_or("");
    args.push(format!(
        "toc: {}",
        typst_string(attr(attrs, "toc").unwrap_or(title))
    ));
    if let Some(value) = attr(attrs, "lead") {
        args.push(format!("lead: {}", typst_string(value)));
    }
    if let Some(value) = attr(attrs, "range") {
        args.push(format!("range: {}", typst_string(value)));
    }
    if let Some(value) = attr(attrs, "flow") {
        args.push(format!("flow: {}", value == "true"));
    }
    if let Some(value) = attr(attrs, "compact") {
        args.push(format!("compact: {}", value == "true"));
    }
    if let Some(value) = attr(attrs, "subs") {
        args.push(format!("subs: {}", value == "true"));
    }
    format!("#chapter({})", args.join(", "))
}

fn parse_widths(value: &str, line: usize) -> Result<Vec<String>> {
    let widths = value
        .split([',', ' '])
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| {
            let number: f32 = part.parse().map_err(|_| Error::Markdown {
                line,
                message: format!("table width is not a number: {part}"),
            })?;
            if !(0.0..=100.0).contains(&number) || number == 0.0 {
                return Err(Error::Markdown {
                    line,
                    message: format!("table width out of range: {part}"),
                });
            }
            Ok(format!("{number}%"))
        })
        .collect::<Result<Vec<_>>>()?;
    if widths.is_empty() {
        return Err(Error::Markdown {
            line,
            message: "table widths is empty".to_string(),
        });
    }
    Ok(widths)
}

fn directive_start(line: &str) -> Option<(&str, &str)> {
    let trimmed = line.trim();
    let rest = trimmed.strip_prefix(":::")?;
    let rest = rest.trim();
    if rest.is_empty() {
        return None;
    }
    let split = rest.find(char::is_whitespace).unwrap_or(rest.len());
    let kind = &rest[..split];
    let attrs = rest[split..].trim();
    Some((kind, attrs))
}

fn parse_attrs(input: &str) -> std::result::Result<BTreeMap<String, String>, String> {
    let mut attrs = BTreeMap::new();
    let chars: Vec<char> = input.chars().collect();
    let mut index = 0;
    while index < chars.len() {
        while index < chars.len() && chars[index].is_whitespace() {
            index += 1;
        }
        if index >= chars.len() {
            break;
        }
        let key_start = index;
        while index < chars.len() && chars[index] != '=' && !chars[index].is_whitespace() {
            index += 1;
        }
        if index >= chars.len() || chars[index] != '=' {
            return Err(format!(
                "directive attribute has no value: {}",
                chars[key_start..index].iter().collect::<String>()
            ));
        }
        let key: String = chars[key_start..index].iter().collect();
        index += 1;
        let value = if index < chars.len() && chars[index] == '"' {
            index += 1;
            let value_start = index;
            while index < chars.len() && chars[index] != '"' {
                index += 1;
            }
            if index >= chars.len() {
                return Err(format!("unclosed quote in directive attribute: {key}"));
            }
            let value: String = chars[value_start..index].iter().collect();
            index += 1;
            value
        } else {
            let value_start = index;
            while index < chars.len() && !chars[index].is_whitespace() {
                index += 1;
            }
            chars[value_start..index].iter().collect()
        };
        attrs.insert(key, value);
    }
    Ok(attrs)
}

fn attr<'a>(attrs: &'a BTreeMap<String, String>, key: &str) -> Option<&'a str> {
    attrs.get(key).map(String::as_str)
}

fn optional_attr(attrs: &BTreeMap<String, String>, key: &str) -> String {
    attr(attrs, key).map_or_else(|| "none".to_string(), typst_string)
}

fn render_segment(markdown: &str, config: &RenderConfig) -> Result<String> {
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
    let mut renderer = Renderer::new(config);
    for event in parser {
        renderer.event(event)?;
    }
    Ok(renderer.finish())
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

struct Renderer {
    out: String,
    list_stack: Vec<bool>,
    in_item: bool,
    table: Option<TableState>,
    image: Option<ImageState>,
    code: Option<String>,
    code_lang: String,
    table_widths: Option<Vec<String>>,
    table_kind: Option<String>,
}

impl Renderer {
    fn new(config: &RenderConfig) -> Self {
        Self {
            out: String::new(),
            list_stack: Vec::new(),
            in_item: false,
            table: None,
            image: None,
            code: None,
            code_lang: String::new(),
            table_widths: config.table_widths.clone(),
            table_kind: config.table_kind.clone(),
        }
    }

    fn event(&mut self, event: Event<'_>) -> Result<()> {
        match event {
            Event::Start(Tag::Paragraph) => {
                if self.table.is_none() && !self.in_item {
                    self.out.push_str("\n\n#block(breakable: false)[");
                }
            }
            Event::End(TagEnd::Paragraph) => {
                if self.table.is_none() && !self.in_item {
                    self.out.push_str("]\n");
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
                    let (number, caption) = figure_parts(&image.alt);
                    let caption = match number {
                        Some(number) => format!(
                            "#fig_label({}) #text({})",
                            typst_string(&number),
                            typst_string(&caption)
                        ),
                        None => format!("#text({})", typst_string(&caption)),
                    };
                    self.push(&format!(
                        "#figure(image({}, width: 100%), caption: [{caption}], numbering: none)\n\n",
                        typst_string(&image.dest)
                    ));
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
            Event::Code(code) => {
                if let Some(image) = &mut self.image {
                    image.alt.push_str(&code);
                } else {
                    self.push(&format!("#raw({})", typst_string(&code)));
                }
            }
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
        let facts = self.table_kind.as_deref() == Some("facts");
        if facts {
            self.out.push_str(&facts_typst(&rows));
        } else {
            self.out
                .push_str(&table_typst(&header, &rows, self.table_widths.as_deref()));
        }
        self.out.push_str("\n\n");
    }
}

fn figure_parts(alt: &str) -> (Option<String>, String) {
    let alt = alt.trim();
    if let Some(captures) = FIGURE_RE.captures(alt) {
        let number = captures.get(1).map(|value| value.as_str().to_string());
        let caption = captures
            .get(2)
            .map(|value| value.as_str().trim().to_string())
            .unwrap_or_default();
        (number, caption)
    } else {
        (None, alt.to_string())
    }
}

fn table_typst(header: &[String], rows: &[Vec<String>], widths: Option<&[String]>) -> String {
    let columns = header
        .len()
        .max(rows.iter().map(Vec::len).max().unwrap_or(0))
        .max(1);
    let mut out = String::from("#data_table(\n");
    out.push_str(&format!("  columns: {columns},\n"));
    if let Some(widths) = widths {
        out.push_str(&format!(
            "  widths: {},\n",
            array_literal(widths.iter().map(|width| width.to_string()).collect())
        ));
    }
    let header_cells = header.iter().map(|cell| format!("[{cell}]")).collect();
    out.push_str(&format!("  header: {},\n", array_literal(header_cells)));
    let row_literals = rows
        .iter()
        .map(|row| {
            array_literal(
                (0..columns)
                    .map(|index| format!("[{}]", row.get(index).map_or("", String::as_str)))
                    .collect(),
            )
        })
        .collect();
    out.push_str(&format!("  rows: {},\n", array_literal(row_literals)));
    out.push(')');
    out
}

fn facts_typst(rows: &[Vec<String>]) -> String {
    let row_literals = rows
        .iter()
        .map(|row| {
            array_literal(vec![
                format!("[{}]", row.first().map_or("", String::as_str)),
                format!("[{}]", row.get(1).map_or("", String::as_str)),
            ])
        })
        .collect();
    format!("#facts_table({})", array_literal(row_literals))
}

fn array_literal(items: Vec<String>) -> String {
    if items.len() == 1 {
        format!("({},)", items[0])
    } else {
        format!("({})", items.join(", "))
    }
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
        let typst =
            markdown_to_typst("::: definition label=\"Tanım\"\nMetin\n:::\n").expect("typst");
        assert!(typst.contains("#definition(label: \"Tanım\")["));
        assert!(typst.contains("#text(\"Metin\")"));
    }

    #[test]
    fn renders_source_refs() {
        let typst = markdown_to_typst("Metin [PD0, s. 3].").expect("typst");
        assert!(typst.contains("#source_ref(\"PD0, s. 3\")"));
    }

    #[test]
    fn renders_tables() {
        let typst = markdown_to_typst("| A | B |\n|---|---|\n| 1 | 2 |\n").expect("typst");
        assert!(typst.contains("#data_table("));
        assert!(typst.contains("header:"));
    }

    #[test]
    fn renders_table_widths() {
        let typst =
            markdown_to_typst("::: table widths=\"5,95\"\n| A | B |\n|---|---|\n| 1 | 2 |\n:::\n")
                .expect("typst");
        assert!(typst.contains("widths: (5%, 95%)"));
    }

    #[test]
    fn renders_chapter_opener() {
        let typst = markdown_to_typst(
            "::: chapter part=\"B\" title=\"Hazırlık\" running=\"Bölüm 0 · Hazırlık\"\n:::\n",
        )
        .expect("typst");
        assert!(typst.contains("#chapter("));
        assert!(typst.contains("part: \"B\""));
        assert!(typst.contains("title: \"Hazırlık\""));
        assert!(typst.contains("toc: \"Hazırlık\""));
    }

    #[test]
    fn renders_front_toc_and_chapter() {
        let typst = markdown_to_typst(
            "::: front kicker=\"Ön kısım\" title=\"Hakkında\"\nMetin\n:::\n\n::: toc title=\"Harita\"\n:::\n\n::: chapter part=\"B\" title=\"Hazırlık\" compact=true\n:::\n",
        )
        .expect("typst");
        assert!(typst.contains("#front(kicker: \"Ön kısım\", title: \"Hakkında\""));
        assert!(typst.contains("#toc_page(title: \"Harita\""));
        assert!(typst.contains("compact: true"));
    }

    #[test]
    fn renders_columns_and_pagebreak() {
        let typst =
            markdown_to_typst("::: columns\nSol\n::: columnbreak\nSağ\n:::\n\n::: pagebreak\n")
                .expect("typst");
        assert!(typst.contains("#two_col["));
        assert!(typst.contains("#colbreak()"));
        assert!(typst.contains("#pagebreak()"));
    }

    #[test]
    fn renders_facts_table() {
        let typst = markdown_to_typst(
            "::: table kind=\"facts\"\n|  |  |\n|---|---|\n| Kaynaklar | PD0 |\n:::\n",
        )
        .expect("typst");
        assert!(typst.contains("#facts_table("));
        assert!(!typst.contains("#data_table("));
    }

    #[test]
    fn rejects_stray_closing_directive() {
        let error = markdown_to_typst(":::\n").expect_err("must reject");
        assert!(matches!(error, Error::Markdown { .. }));
    }

    #[test]
    fn rejects_unknown_directives() {
        let error = markdown_to_typst("::: mystery\nx\n:::\n").expect_err("must reject");
        assert!(matches!(error, Error::Markdown { .. }));
    }
}
