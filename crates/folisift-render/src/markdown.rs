use std::collections::{BTreeMap, HashMap};
use std::sync::LazyLock;

use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use regex::Regex;
use serde::Deserialize;

use crate::diagram::DiagramAsset;
use crate::error::{Error, Result};

static SOURCE_REF_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[([^\[\]]+?,\s*(?:s|p)\.\s*\d+(?:\s*[–-]\s*\d+)?)\]")
        .expect("valid source reference regex")
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

#[derive(Debug, Clone)]
struct RenderConfig {
    table_widths: Option<Vec<String>>,
    table_kind: Option<String>,
    table_caption: Option<String>,
    table_source: Option<String>,
    table_id: Option<String>,
    table_number: bool,
    table_keep_with_next: bool,
    table_allow_page_break: bool,
    table_min_space_mm: Option<f32>,
    figure_label: String,
    toc_label: String,
    table_label: String,
    recall_label: String,
    explain_label: String,
    review_plan_label: String,
    synthesis_label: String,
    answer_label: String,
    revision_badge: String,
    organiser_badge: String,
    summary_label: String,
    workspace_label: String,
    answer_key_label: String,
    diagram_assets: HashMap<String, DiagramAsset>,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            table_widths: None,
            table_kind: None,
            table_caption: None,
            table_source: None,
            table_id: None,
            table_number: true,
            table_keep_with_next: false,
            table_allow_page_break: false,
            table_min_space_mm: None,
            figure_label: "Şekil".to_string(),
            toc_label: "İÇİNDEKİLER".to_string(),
            table_label: "Tablo".to_string(),
            recall_label: "HATIRLA".to_string(),
            explain_label: "AÇIKLA".to_string(),
            review_plan_label: "TEKRAR PLANI".to_string(),
            synthesis_label: "TEK SAYFA SENTEZ".to_string(),
            answer_label: "CEVAP".to_string(),
            revision_badge: "SINAV TEKRARI".to_string(),
            organiser_badge: "BİLGİ ORGANİZATÖRÜ".to_string(),
            summary_label: "ÖZET".to_string(),
            workspace_label: "ÇALIŞMA ALANI".to_string(),
            answer_key_label: "CEVAP ANAHTARI".to_string(),
            diagram_assets: HashMap::new(),
        }
    }
}

/// Convert guide Markdown to Typst content.
pub fn markdown_to_typst(markdown: &str) -> Result<String> {
    lint_markdown(markdown)?;
    markdown_to_typst_with(markdown, &RenderConfig::default())
}

/// Convert Markdown to Typst with style-owned labels.
pub fn markdown_to_typst_styled(
    markdown: &str,
    figure_label: &str,
    toc_label: &str,
) -> Result<String> {
    markdown_to_typst_styled_with_table(markdown, figure_label, toc_label, "Tablo")
}

pub fn markdown_to_typst_styled_with_table(
    markdown: &str,
    figure_label: &str,
    toc_label: &str,
    table_label: &str,
) -> Result<String> {
    markdown_to_typst_styled_with_diagrams(
        markdown,
        figure_label,
        toc_label,
        table_label,
        HashMap::new(),
    )
}

pub fn markdown_to_typst_styled_with_diagrams(
    markdown: &str,
    figure_label: &str,
    toc_label: &str,
    table_label: &str,
    diagram_assets: HashMap<String, DiagramAsset>,
) -> Result<String> {
    lint_markdown(markdown)?;
    markdown_to_typst_with(
        markdown,
        &RenderConfig {
            figure_label: figure_label.to_string(),
            toc_label: toc_label.to_string(),
            table_label: table_label.to_string(),
            diagram_assets,
            ..RenderConfig::default()
        },
    )
}

/// Convert Markdown with every style-owned label, including revision and
/// organiser component labels.
pub fn markdown_to_typst_styled_with_strings(
    markdown: &str,
    strings: &crate::styles::StyleStrings,
    diagram_assets: HashMap<String, DiagramAsset>,
) -> Result<String> {
    lint_markdown(markdown)?;
    markdown_to_typst_with(
        markdown,
        &RenderConfig {
            figure_label: strings.figure.clone(),
            toc_label: strings.toc.clone(),
            table_label: strings.table.clone(),
            recall_label: strings.recall.clone(),
            explain_label: strings.explain.clone(),
            review_plan_label: strings.review_plan.clone(),
            synthesis_label: strings.synthesis.clone(),
            answer_label: strings.answer.clone(),
            revision_badge: strings.revision_badge.clone(),
            organiser_badge: strings.organiser_badge.clone(),
            summary_label: strings.summary.clone(),
            workspace_label: strings.workspace.clone(),
            answer_key_label: strings.answer_key.clone(),
            diagram_assets,
            ..RenderConfig::default()
        },
    )
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
                    && !matches!(nested, "columnbreak" | "pagebreak")
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
                "recall" | "explain" | "review-plan" => {
                    let inner_typst = markdown_to_typst_with(&inner_markdown, config)?;
                    let (function, default_label) = match kind {
                        "recall" => ("recall", config.recall_label.as_str()),
                        "explain" => ("explain", config.explain_label.as_str()),
                        _ => ("review_plan", config.review_plan_label.as_str()),
                    };
                    let label = attr(&attrs, "label").unwrap_or(default_label);
                    out.push_str(&format!(
                        "#{function}(label: {})[{inner_typst}]\n\n",
                        typst_string(label)
                    ));
                }
                "must-know" => {
                    let inner_typst = markdown_to_typst_with(&inner_markdown, config)?;
                    out.push_str(&format!("#must_know[{inner_typst}]\n\n"));
                }
                "synthesis" => {
                    let inner_typst = markdown_to_typst_with(&inner_markdown, config)?;
                    out.push_str(&format!(
                        "#synthesis_page(title: {}, badge: {})[{inner_typst}]\n\n",
                        typst_string(attr(&attrs, "title").unwrap_or(&config.synthesis_label)),
                        typst_string(&config.revision_badge)
                    ));
                }
                "answer" => {
                    let inner_typst = markdown_to_typst_with(&inner_markdown, config)?;
                    out.push_str(&format!(
                        "#answer(label: {})[{inner_typst}]\n\n",
                        typst_string(attr(&attrs, "label").unwrap_or(&config.answer_label))
                    ));
                }
                "organiser" => {
                    let inner_typst = markdown_to_typst_with(&inner_markdown, config)?;
                    out.push_str(&format!(
                        "#organiser_page(title: {}, source: {}, badge: {})[{inner_typst}]\n\n",
                        typst_string(attr(&attrs, "title").unwrap_or("")),
                        optional_attr(&attrs, "source"),
                        typst_string(&config.organiser_badge)
                    ));
                }
                "cue" => {
                    let inner_typst = markdown_to_typst_with(&inner_markdown, config)?;
                    out.push_str(&format!(
                        "#cue_block(label: {})[{inner_typst}]\n\n",
                        optional_attr(&attrs, "label")
                    ));
                }
                "summary" => {
                    let inner_typst = markdown_to_typst_with(&inner_markdown, config)?;
                    out.push_str(&format!(
                        "#summary_block(label: {})[{inner_typst}]\n\n",
                        typst_string(&config.summary_label)
                    ));
                }
                "problem" => {
                    let inner_typst = markdown_to_typst_with(&inner_markdown, config)?;
                    out.push_str(&format!(
                        "#problem_block(title: {})[{inner_typst}]\n\n",
                        optional_attr(&attrs, "title")
                    ));
                }
                "workspace" => {
                    let inner_typst = markdown_to_typst_with(&inner_markdown, config)?;
                    let lines = attr(&attrs, "lines")
                        .and_then(|value| value.parse::<usize>().ok())
                        .unwrap_or(4)
                        .clamp(1, 20);
                    out.push_str(&format!(
                        "#workspace_block(label: {}, lines: {lines})[{inner_typst}]\n\n",
                        attr(&attrs, "label")
                            .map_or_else(|| typst_string(&config.workspace_label), typst_string)
                    ));
                }
                "answer-key" => {
                    let inner_typst = markdown_to_typst_with(&inner_markdown, config)?;
                    out.push_str(&format!(
                        "#answer_key(label: {})[{inner_typst}]\n\n",
                        typst_string(&config.answer_key_label)
                    ));
                }
                "formula" => {
                    let inner_typst = markdown_to_typst_with(&inner_markdown, config)?;
                    out.push_str(&format!(
                        "#formula_card(title: {}, source: {})[{inner_typst}]\n\n",
                        optional_attr(&attrs, "title"),
                        optional_attr(&attrs, "source")
                    ));
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
                    // A front note may contain one table (the pilot guides do).
                    // Documented form: close the table first, then close front.
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
                    let mut args = vec![
                        format!(
                            "title: {}",
                            typst_string(attr(&attrs, "title").unwrap_or(""))
                        ),
                        format!("note: {}", typst_string(attr(&attrs, "note").unwrap_or(""))),
                        format!("label: {}", typst_string(&config.toc_label)),
                    ];
                    if let Some(value) = attr(&attrs, "subs") {
                        args.push(format!("subs: {}", value == "true"));
                    }
                    for (attr_key, arg_key) in
                        [("include", "toc_include"), ("exclude", "toc_exclude")]
                    {
                        if let Some(value) = attr(&attrs, attr_key) {
                            let items = value
                                .split(',')
                                .map(str::trim)
                                .filter(|item| !item.is_empty())
                                .map(typst_string)
                                .collect::<Vec<_>>();
                            let body = if items.len() == 1 {
                                format!("{},", items[0])
                            } else {
                                items.join(", ")
                            };
                            args.push(format!("{arg_key}: ({body})"));
                        }
                    }
                    if let Some(value) = attr(&attrs, "toc_depth") {
                        let depth: u8 = value.parse().map_err(|_| Error::Markdown {
                            line: start_line,
                            message: format!("toc_depth is not a number: {value}"),
                        })?;
                        if !(1..=2).contains(&depth) {
                            return Err(Error::Markdown {
                                line: start_line,
                                message: format!("toc_depth must be 1 or 2, got {value}"),
                            });
                        }
                        args.push(format!("depth: {depth}"));
                    }
                    out.push_str(&format!("#toc_page({})\n\n", args.join(", ")));
                }
                "glossary" => {
                    let mut glossary_config = config.clone();
                    glossary_config.table_kind = Some("glossary".to_string());
                    let inner_typst = markdown_to_typst_with(&inner_markdown, &glossary_config)?;
                    out.push_str(&inner_typst);
                    out.push_str("\n\n");
                }
                "columns" => {
                    let inner_typst = markdown_to_typst_with(&inner_markdown, config)?;
                    let mut args = Vec::new();
                    for key in ["flow", "keep_with_next"] {
                        if let Some(value) = attr(&attrs, key) {
                            args.push(format!("{key}: {}", value == "true"));
                        }
                    }
                    if let Some(value) = attr(&attrs, "min_rows_before_break") {
                        let rows: usize = value.parse().map_err(|_| Error::Markdown {
                            line: start_line,
                            message: format!("min_rows_before_break is not a number: {value}"),
                        })?;
                        if rows > 0 {
                            return Err(Error::Markdown {
                                line: start_line,
                                message: "min_rows_before_break > 0 is not supported; use ::: columnbreak at row boundaries".to_string(),
                            });
                        }
                    }
                    if args.is_empty() {
                        out.push_str(&format!("#two_col[{inner_typst}]\n\n"));
                    } else {
                        out.push_str(&format!("#two_col({})[{inner_typst}]\n\n", args.join(", ")));
                    }
                }
                "figure" => {
                    if !inner_markdown.trim().is_empty() {
                        return Err(Error::Markdown {
                            line: start_line,
                            message: "figure takes no body; use src=\"<asset>\"".to_string(),
                        });
                    }
                    let src = attr(&attrs, "src").ok_or_else(|| Error::Markdown {
                        line: start_line,
                        message: "figure needs src=\"<asset>\"".to_string(),
                    })?;
                    let source = attr(&attrs, "source").ok_or_else(|| Error::Markdown {
                        line: start_line,
                        message: "figure needs source=\"<source, s. N>\"".to_string(),
                    })?;
                    let alt = attr(&attrs, "alt").ok_or_else(|| Error::Markdown {
                        line: start_line,
                        message: "figure needs alt text".to_string(),
                    })?;
                    let caption = attr(&attrs, "caption").ok_or_else(|| Error::Markdown {
                        line: start_line,
                        message: "figure needs caption text".to_string(),
                    })?;
                    let width = attr(&attrs, "width").unwrap_or("90%").trim();
                    let valid_width = width
                        .strip_suffix('%')
                        .and_then(|value| value.parse::<u8>().ok())
                        .is_some_and(|value| (50..=100).contains(&value));
                    if !valid_width {
                        return Err(Error::Markdown {
                            line: start_line,
                            message: format!(
                                "figure width must be a percentage from 50% to 100%, got {width:?}"
                            ),
                        });
                    }
                    out.push_str(&format!(
                        "#figure(fitted_image({}, width: {}, alt: {}), caption: [{}], numbering: none)\n\n",
                        typst_string(src),
                        width,
                        typst_string(alt),
                        caption_typst(&format!("{caption} [{}]", source)),
                    ));
                }
                "diagram" => {
                    if !inner_markdown.trim().is_empty() {
                        return Err(Error::Markdown {
                            line: start_line,
                            message: "diagram takes no body; the spec lives in diagrams/<id>.json"
                                .to_string(),
                        });
                    }
                    let id = attr(&attrs, "id").ok_or_else(|| Error::Markdown {
                        line: start_line,
                        message: "diagram needs id=\"<id>\"".to_string(),
                    })?;
                    let asset = config.diagram_assets.get(id).ok_or_else(|| Error::Markdown {
                        line: start_line,
                        message: format!(
                            "diagram {id:?} has no built asset; run `folisift diagram build --guide <slug> --id {id}`"
                        ),
                    })?;
                    let width = attr(&attrs, "width")
                        .or(asset.width.as_deref())
                        .unwrap_or("90%")
                        .trim();
                    let valid_width = width
                        .strip_suffix('%')
                        .and_then(|value| value.parse::<u8>().ok())
                        .is_some_and(|value| (50..=100).contains(&value));
                    if !valid_width {
                        return Err(Error::Markdown {
                            line: start_line,
                            message: format!(
                                "diagram width must be a percentage from 50% to 100%, got {width:?}"
                            ),
                        });
                    }
                    let max_width = attr(&attrs, "max_width")
                        .or(asset.max_width.as_deref())
                        .unwrap_or("100%")
                        .trim();
                    let valid_max_width = max_width
                        .strip_suffix('%')
                        .and_then(|value| value.parse::<u8>().ok())
                        .is_some_and(|value| (50..=100).contains(&value));
                    if !valid_max_width {
                        return Err(Error::Markdown {
                            line: start_line,
                            message: format!(
                                "diagram max_width must be a percentage from 50% to 100%, got {max_width:?}"
                            ),
                        });
                    }
                    let requested = width
                        .strip_suffix('%')
                        .and_then(|value| value.parse::<u8>().ok())
                        .unwrap_or(90);
                    let cap = max_width
                        .strip_suffix('%')
                        .and_then(|value| value.parse::<u8>().ok())
                        .unwrap_or(100);
                    let width = format!("{}%", requested.min(cap));
                    let max_height = attr(&attrs, "max_height")
                        .or_else(|| attr(&attrs, "height"))
                        .or(asset.max_height.as_deref())
                        .unwrap_or("14cm")
                        .trim();
                    if !valid_css_length(max_height) {
                        return Err(Error::Markdown {
                            line: start_line,
                            message: format!(
                                "diagram max_height must be a cm/mm/in/pt length, got {max_height:?}"
                            ),
                        });
                    }
                    let fit = attr(&attrs, "fit")
                        .or(asset.fit.as_deref())
                        .unwrap_or("contain")
                        .trim();
                    if !matches!(fit, "contain" | "width" | "height") {
                        return Err(Error::Markdown {
                            line: start_line,
                            message: format!(
                                "diagram fit must be contain, width or height, got {fit:?}"
                            ),
                        });
                    }
                    let figure = format!(
                        "#figure(fitted_image({}, width: {}, max_height: {}, fit: {}, alt: {}), caption: [{}], supplement: [{}], numbering: \"1\")",
                        typst_string(&asset.path),
                        width,
                        max_height,
                        typst_string(fit),
                        typst_string(&asset.alt),
                        caption_typst(&asset.caption),
                        typst_string(&config.figure_label),
                    );
                    if attr(&attrs, "keep_with_next") == Some("true") {
                        out.push_str(&format!("#block(sticky: true)[{figure}]\n\n"));
                    } else {
                        out.push_str(&format!("{figure}\n\n"));
                    }
                }
                "table" => {
                    let mut table_config = config.clone();
                    table_config.table_widths = match attr(&attrs, "widths") {
                        Some(widths) => {
                            let widths = parse_widths(widths, start_line)?;
                            let columns = markdown_table_columns(&inner_markdown);
                            if columns > 0 && widths.len() != columns {
                                return Err(Error::Markdown {
                                    line: start_line,
                                    message: format!(
                                        "table widths has {} values but the table has {columns} columns",
                                        widths.len()
                                    ),
                                });
                            }
                            Some(widths)
                        }
                        None => None,
                    };
                    table_config.table_kind = attr(&attrs, "kind").map(ToString::to_string);
                    table_config.table_caption = attr(&attrs, "caption").map(ToString::to_string);
                    table_config.table_source = attr(&attrs, "source").map(ToString::to_string);
                    table_config.table_id = match attr(&attrs, "id") {
                        Some(id) => Some(validate_table_id(id, start_line)?),
                        None => None,
                    };
                    if let Some(value) =
                        attr(&attrs, "caption_number").or_else(|| attr(&attrs, "number"))
                    {
                        table_config.table_number =
                            !matches!(value.trim().to_lowercase().as_str(), "false" | "0" | "no");
                    }
                    if let Some(value) = attr(&attrs, "keep_with_next") {
                        table_config.table_keep_with_next = value == "true";
                    }
                    if let Some(value) = attr(&attrs, "allow_page_break") {
                        table_config.table_allow_page_break = value == "true";
                    }
                    if let Some(value) = attr(&attrs, "min_space_mm") {
                        let mm: f32 = value.parse().map_err(|_| Error::Markdown {
                            line: start_line,
                            message: format!("table min_space_mm is not a number: {value}"),
                        })?;
                        if !(0.0..=100.0).contains(&mm) {
                            return Err(Error::Markdown {
                                line: start_line,
                                message: format!("table min_space_mm out of range: {value}"),
                            });
                        }
                        table_config.table_min_space_mm = Some(mm);
                    }
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
    for key in [
        "keep_with_next",
        "break_before",
        "break_after",
        "allow_split",
        "full_page_opener",
    ] {
        if let Some(value) = attr(attrs, key) {
            args.push(format!("{key}: {}", value == "true"));
        }
    }
    format!("#chapter({})", args.join(", "))
}

fn valid_css_length(value: &str) -> bool {
    ["cm", "mm", "in", "pt"].iter().any(|unit| {
        value
            .strip_suffix(unit)
            .and_then(|number| number.trim().parse::<f32>().ok())
            .is_some_and(|number| number > 0.0 && number < 1000.0)
    })
}

fn markdown_table_columns(markdown: &str) -> usize {
    for line in markdown.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('|') {
            return trimmed.trim_matches('|').split('|').count();
        }
    }
    0
}

fn parse_widths(value: &str, line: usize) -> Result<Vec<String>> {
    if value.trim().is_empty() {
        return Err(Error::Markdown {
            line,
            message: "table widths is empty".to_string(),
        });
    }
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
    let sum = widths
        .iter()
        .filter_map(|width| width.trim_end_matches('%').parse::<f32>().ok())
        .sum::<f32>();
    if !(99.0..=101.0).contains(&sum) {
        return Err(Error::Markdown {
            line,
            message: format!(
                "E_TABLE_WIDTHS: table widths ({} columns) sum to {sum}%, expected 100%",
                widths.len()
            ),
        });
    }
    Ok(widths)
}

fn lint_markdown(markdown: &str) -> Result<()> {
    let mut fence: Option<String> = None;
    for (index, line) in markdown.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            if fence.is_some() {
                fence = None;
                continue;
            }
            let language = trimmed.trim_start_matches(['`', '~']).trim().to_lowercase();
            if language == "typst" {
                return Err(Error::Markdown {
                    line: index + 1,
                    message: "raw Typst is not allowed in guide content; use directives"
                        .to_string(),
                });
            }
            fence = Some(language);
            continue;
        }
        if fence.is_some() {
            continue;
        }
        if ["#let ", "#set ", "#show ", "#import ", "#include "]
            .iter()
            .any(|prefix| trimmed.starts_with(prefix))
            || trimmed.contains("<style")
            || trimmed.contains("<script")
            || trimmed.starts_with("@page")
        {
            return Err(Error::Markdown {
                line: index + 1,
                message: "raw Typst/HTML/CSS is not allowed in guide content".to_string(),
            });
        }
    }
    Ok(())
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

pub(crate) fn parse_attrs(input: &str) -> std::result::Result<BTreeMap<String, String>, String> {
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

enum GlossaryRow {
    Group(String),
    Cells(Vec<String>),
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
    table_caption: Option<String>,
    table_source: Option<String>,
    table_id: Option<String>,
    table_number: bool,
    table_keep_with_next: bool,
    table_allow_page_break: bool,
    table_min_space_mm: Option<f32>,
    figure_label: String,
    table_label: String,
    glossary_header: Option<Vec<String>>,
    glossary_rows: Vec<GlossaryRow>,
    glossary_pending_group: Option<String>,
    glossary_heading: Option<String>,
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
            table_caption: config.table_caption.clone(),
            table_source: config.table_source.clone(),
            table_id: config.table_id.clone(),
            table_number: config.table_number,
            table_keep_with_next: config.table_keep_with_next,
            table_allow_page_break: config.table_allow_page_break,
            table_min_space_mm: config.table_min_space_mm,
            figure_label: config.figure_label.clone(),
            table_label: config.table_label.clone(),
            glossary_header: None,
            glossary_rows: Vec::new(),
            glossary_pending_group: None,
            glossary_heading: None,
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
                    if self.table_kind.as_deref() == Some("glossary") {
                        self.glossary_heading = Some(String::new());
                    } else {
                        self.out.push_str("\n\n");
                        self.out.push_str(&"=".repeat(heading_level(level)));
                        self.out.push(' ');
                    }
                }
            }
            Event::End(TagEnd::Heading(_)) => {
                if self.table.is_none() {
                    if self.table_kind.as_deref() == Some("glossary") {
                        if let Some(heading) = self.glossary_heading.take() {
                            let heading = heading.trim();
                            if !heading.is_empty() {
                                self.glossary_pending_group = Some(heading.to_string());
                            }
                        }
                    } else {
                        self.out.push_str("\n\n");
                    }
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
                if self.table_kind.as_deref() != Some("glossary") {
                    self.out.push_str("\n\n");
                }
                self.table = Some(TableState::default());
            }
            Event::End(TagEnd::Table) => {
                if let Some(table) = self.table.take() {
                    if self.table_kind.as_deref() == Some("glossary") {
                        self.collect_glossary_table(table);
                    } else {
                        self.emit_table(table);
                    }
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
                            "#fig_label({}, word: {}) {}",
                            typst_string(&number),
                            typst_string(&self.figure_label),
                            caption_typst(&caption)
                        ),
                        None => caption_typst(&caption),
                    };
                    self.push(&format!(
                        "#figure(fitted_image({}, alt: {}), caption: [{caption}], numbering: none, alt: {})\n\n",
                        typst_string(&image.dest),
                        typst_string(&image.alt),
                        typst_string(&image.alt)
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
            Event::Html(raw) | Event::InlineHtml(raw) => {
                if !raw.trim_start().starts_with("<!--") {
                    return Err(Error::Markdown {
                        line: 0,
                        message:
                            "raw HTML is not supported; use Markdown, a code fence or a directive"
                                .to_string(),
                    });
                }
            }
            Event::InlineMath(_) | Event::DisplayMath(_) => {
                return Err(Error::Markdown {
                    line: 0,
                    message: "math delimiters are not supported; use inline code or a code fence"
                        .to_string(),
                });
            }
            Event::FootnoteReference(_) | Event::TaskListMarker(_) => {
                return Err(Error::Markdown {
                    line: 0,
                    message: "footnotes and task-list markers are not supported".to_string(),
                });
            }
            Event::Start(_) | Event::End(_) => {}
        }
        Ok(())
    }

    fn finish(mut self) -> String {
        if let Some(table) = self.table.take() {
            if self.table_kind.as_deref() == Some("glossary") {
                self.collect_glossary_table(table);
            } else {
                self.emit_table(table);
            }
        }
        if self.table_kind.as_deref() == Some("glossary") && !self.glossary_rows.is_empty() {
            self.out.push_str(&self.glossary_typst());
        }
        self.out
    }

    fn push(&mut self, value: &str) {
        if let Some(heading) = &mut self.glossary_heading {
            heading.push_str(value);
        } else if let Some(table) = &mut self.table
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

    fn collect_glossary_table(&mut self, table: TableState) {
        if self.glossary_header.is_none() {
            self.glossary_header = table.header;
        }
        if let Some(group) = self.glossary_pending_group.take() {
            self.glossary_rows.push(GlossaryRow::Group(group));
        }
        for row in table.rows {
            self.glossary_rows.push(GlossaryRow::Cells(row));
        }
    }

    fn glossary_typst(&self) -> String {
        let columns = self
            .glossary_header
            .as_ref()
            .map_or(4, Vec::len)
            .max(
                self.glossary_rows
                    .iter()
                    .filter_map(|row| match row {
                        GlossaryRow::Cells(cells) => Some(cells.len()),
                        GlossaryRow::Group(_) => None,
                    })
                    .max()
                    .unwrap_or(4),
            )
            .max(1);
        let header = self
            .glossary_header
            .as_ref()
            .map(|cells| {
                cells
                    .iter()
                    .map(|cell| format!("[{cell}]"))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let body = self
            .glossary_rows
            .iter()
            .map(|row| match row {
                GlossaryRow::Group(group) => format!(
                    "table.cell(colspan: {columns}, fill: sage.lighten(88%), inset: (x: 1.5mm, y: 1.0mm))[#glossary_row[{group}]]"
                ),
                GlossaryRow::Cells(cells) => (0..columns)
                    .map(|index| format!("[{}]", cells.get(index).map_or("", String::as_str)))
                    .collect::<Vec<_>>()
                    .join(", "),
            })
            .collect::<Vec<_>>();
        format!(
            "#glossary_table(\n  columns: {columns},\n  widths: (18%, 32%, 18%, 32%),\n  header: ({header}),\n  body: ({body}),\n)\n\n",
            header = header.join(", "),
            body = body.join(",\n  "),
        )
    }

    fn emit_table(&mut self, table: TableState) {
        let header = table.header.unwrap_or_default();
        let rows = table.rows;
        let facts = self.table_kind.as_deref() == Some("facts");
        let glossary = self.table_kind.as_deref() == Some("glossary");
        if facts {
            self.out.push_str(&facts_typst(&rows));
            if let Some(id) = &self.table_id {
                self.out.push_str(&format!(" <{id}>"));
            }
        } else {
            self.out.push_str(&table_typst(
                &header,
                &rows,
                self.table_widths.as_deref(),
                self.table_caption.as_deref(),
                self.table_source.as_deref(),
                &self.table_label,
                self.table_id.as_deref(),
                glossary,
                self.table_number,
                self.table_keep_with_next,
                self.table_allow_page_break,
                self.table_min_space_mm,
            ));
        }
        self.out.push_str("\n\n");
    }
}

fn caption_typst(caption: &str) -> String {
    let mut out = String::new();
    let mut last = 0;
    for capture in SOURCE_REF_RE.captures_iter(caption) {
        let Some(full) = capture.get(0) else {
            continue;
        };
        out.push_str(&format!(
            "#text({}) ",
            typst_string(&caption[last..full.start()])
        ));
        let body = capture.get(1).map_or("", |value| value.as_str());
        out.push_str(&format!("#source_ref({}) ", typst_string(body)));
        last = full.end();
    }
    out.push_str(&format!("#text({})", typst_string(&caption[last..])));
    out
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

#[allow(clippy::too_many_arguments)]
fn table_typst(
    header: &[String],
    rows: &[Vec<String>],
    widths: Option<&[String]>,
    caption: Option<&str>,
    source: Option<&str>,
    table_label: &str,
    id: Option<&str>,
    glossary: bool,
    number: bool,
    keep_with_next: bool,
    allow_page_break: bool,
    min_space_mm: Option<f32>,
) -> String {
    let columns = header
        .len()
        .max(rows.iter().map(Vec::len).max().unwrap_or(0))
        .max(1);
    let mut out = String::from("#data_table(\n");
    out.push_str(&format!("  columns: {columns},\n"));
    let glossary_widths = [
        "18%".to_string(),
        "32%".to_string(),
        "18%".to_string(),
        "32%".to_string(),
    ];
    let widths = widths.or(if glossary {
        Some(&glossary_widths[..])
    } else {
        None
    });
    if let Some(widths) = widths {
        out.push_str(&format!(
            "  widths: {},\n",
            array_literal(widths.iter().map(|width| width.to_string()).collect())
        ));
    }
    if glossary {
        out.push_str("  compact: true,\n");
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
    let body = if caption.is_some() || source.is_some() {
        let caption = caption.map_or_else(|| "none".to_string(), typst_string);
        let source = source.map_or_else(|| "none".to_string(), typst_string);
        format!(
            "#table_block(caption: {caption}, source: {source}, label: {}, number: {}, keep_with_next: {keep_with_next}, allow_page_break: {allow_page_break}, min_space_mm: {})[{}]",
            typst_string(table_label),
            number,
            min_space_mm.map_or_else(|| "none".to_string(), |value| format!("{value}")),
            out
        )
    } else {
        out
    };
    match id {
        Some(id) => format!("{body} <{id}>"),
        None => body,
    }
}

fn validate_table_id(id: &str, line: usize) -> Result<String> {
    if id.is_empty()
        || !id.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | ':')
        })
    {
        return Err(Error::Markdown {
            line,
            message: format!("table id must use letters, digits, '-', '_' or ':': {id:?}"),
        });
    }
    Ok(id.to_string())
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
    fn renders_a_table_label() {
        let typst = markdown_to_typst(
            "::: table caption=\"Derleme\" source=\"PD0, s. 106\" id=\"tab-derleme\"\n| A | B |\n|---|---|\n| 1 | 2 |\n:::\n",
        )
        .expect("typst");
        assert!(typst.contains("<tab-derleme>"));
    }

    #[test]
    fn source_figure_directive_emits_provenance() {
        let typst = markdown_to_typst(
            "::: figure src=\"data/crops/abc.png\" caption=\"ROC eğrisi\" source=\"PD3, s. 45\" alt=\"ROC eğrisi\" width=\"80%\"\n:::",
        )
        .expect("figure");
        assert!(
            typst.contains("fitted_image(\"data/crops/abc.png\""),
            "{typst}"
        );
        assert!(typst.contains("#source_ref(\"PD3, s. 45\")"), "{typst}");
        assert!(typst.contains("width: 80%"));
    }

    #[test]
    fn figure_directive_requires_caption() {
        let error = markdown_to_typst(
            "::: figure src=\"data/crops/abc.png\" source=\"PD3, s. 45\" alt=\"ROC eğrisi; eksenler\"\n:::",
        )
        .expect_err("caption");
        assert!(error.to_string().contains("caption"), "{error}");
    }

    #[test]
    fn diagram_size_options_are_explicit_and_validated() {
        let asset = DiagramAsset {
            id: "d1".to_string(),
            path: "assets/generated/d1.svg".to_string(),
            caption: "Karar".to_string(),
            alt: "Karar ağacı".to_string(),
            source: "PD3, s. 3".to_string(),
            width: Some("70%".to_string()),
            max_width: Some("80%".to_string()),
            max_height: Some("10cm".to_string()),
            fit: Some("contain".to_string()),
        };
        let mut assets = HashMap::new();
        assets.insert("d1".to_string(), asset);
        let typst = markdown_to_typst_styled_with_diagrams(
            "::: diagram id=\"d1\"\n:::",
            "Şekil",
            "İÇİNDEKİLER",
            "Tablo",
            assets,
        )
        .expect("diagram");
        assert!(typst.contains("width: 70%"), "{typst}");
        assert!(typst.contains("max_height: 10cm"));
        let error = markdown_to_typst_styled_with_diagrams(
            "::: diagram id=\"d1\" fit=\"stretch\"\n:::",
            "Şekil",
            "İÇİNDEKİLER",
            "Tablo",
            HashMap::from([(
                "d1".to_string(),
                DiagramAsset {
                    id: "d1".to_string(),
                    path: "assets/generated/d1.svg".to_string(),
                    caption: "Karar".to_string(),
                    alt: "Karar ağacı".to_string(),
                    source: "PD3, s. 3".to_string(),
                    width: None,
                    max_width: None,
                    max_height: None,
                    fit: None,
                },
            )]),
        )
        .expect_err("fit");
        assert!(error.to_string().contains("fit"));
    }

    #[test]
    fn table_placement_options_are_emitted() {
        let typst = markdown_to_typst(
            "::: table caption=\"T\" keep_with_next=\"true\" allow_page_break=\"true\" min_space_mm=\"6\"\n| A | B |\n|---|---|\n| 1 | 2 |\n:::\n",
        )
        .expect("typst");
        assert!(typst.contains("keep_with_next: true"), "{typst}");
        assert!(typst.contains("allow_page_break: true"));
        assert!(typst.contains("min_space_mm: 6"));
    }

    #[test]
    fn caption_numbering_can_be_disabled_without_dropping_the_id() {
        let typst = markdown_to_typst(
            "::: table caption=\"Şekil 2’nin dalları\" caption_number=\"false\" id=\"tab-branches\"\n| A | B |\n|---|---|\n| 1 | 2 |\n:::\n",
        )
        .expect("typst");
        assert!(typst.contains("number: false"));
        assert!(typst.contains("<tab-branches>"));
    }

    #[test]
    fn rejects_an_unsafe_table_id() {
        let error =
            markdown_to_typst("::: table id=\"bad id\"\n| A | B |\n|---|---|\n| 1 | 2 |\n:::\n")
                .expect_err("must reject");
        assert!(error.to_string().contains("table id"));
    }

    #[test]
    fn rejects_table_widths_that_do_not_sum_to_100() {
        let error =
            markdown_to_typst("::: table widths=\"20,30\"\n| A | B |\n|---|---|\n| 1 | 2 |\n:::\n")
                .expect_err("must reject");
        assert!(error.to_string().contains("E_TABLE_WIDTHS"));
        assert!(error.to_string().contains("sum to 50%"));
    }

    #[test]
    fn rejects_mismatched_table_widths() {
        let error = markdown_to_typst(
            "::: table widths=\"20,80\"\n| A | B | C |\n|---|---|---|\n| 1 | 2 | 3 |\n:::\n",
        )
        .expect_err("must reject");
        assert!(error.to_string().contains("3 columns"));
    }

    #[test]
    fn glossary_groups_share_one_table() {
        let typst = markdown_to_typst(
            "::: glossary\n\n### Bir\n\n| Terim | Tanım | Terim | Tanım |\n|---|---|---|---|\n| **A:** | Bir. | **B:** | İki. |\n\n### İki\n\n| Terim | Tanım | Terim | Tanım |\n|---|---|---|---|\n| **C:** | Üç. | **D:** | Dört. |\n\n:::",
        )
        .expect("glossary");
        assert_eq!(typst.matches("#glossary_table(").count(), 1, "{typst}");
        assert_eq!(typst.matches("#glossary_row[").count(), 2, "{typst}");
        assert_eq!(typst.matches("#data_table(").count(), 0, "{typst}");
    }

    #[test]
    fn renders_table_widths() {
        let typst =
            markdown_to_typst("::: table widths=\"5,95\"\n| A | B |\n|---|---|\n| 1 | 2 |\n:::\n")
                .expect("typst");
        assert!(typst.contains("widths: (5%, 95%)"));
    }

    #[test]
    fn renders_a_glossary_as_a_compact_four_column_table() {
        let typst = markdown_to_typst(
            "::: glossary\n\n### Sözdizimi\n\n| Terim | Tanım | Terim | Tanım |\n|---|---|---|---|\n| **A:** | Bir. | **B:** | İki. |\n\n:::",
        )
        .expect("typst");
        assert!(typst.contains("widths: (18%, 32%, 18%, 32%)"), "{typst}");
        assert!(typst.contains("#glossary_table("), "{typst}");
        assert!(typst.contains("#glossary_row["), "{typst}");
        assert!(!typst.contains("#data_table("), "{typst}");
        assert!(!typst.contains("#glossary_group["), "{typst}");
        assert!(typst.contains("Sözdizimi"));
    }

    #[test]
    fn toc_depth_and_subs_are_emitted_or_rejected() {
        let typst = markdown_to_typst("::: toc title=\"X\" subs=\"false\" toc_depth=\"1\" include=\"Bölüm 3, Giriş\" exclude=\"Ek\"\n:::")
            .expect("toc");
        assert!(typst.contains("subs: false"));
        assert!(typst.contains("depth: 1"));
        assert!(typst.contains("toc_include: (\"Bölüm 3\", \"Giriş\")"));
        assert!(typst.contains("toc_exclude: (\"Ek\",)"));
        let error = markdown_to_typst("::: toc toc_depth=\"3\"\n:::").expect_err("depth");
        assert!(error.to_string().contains("toc_depth"));
    }

    #[test]
    fn chapter_and_columns_layout_options_are_emitted_or_rejected() {
        let typst = markdown_to_typst(
            "::: chapter title=\"Bölüm\" keep_with_next=\"false\" break_before=\"true\" break_after=\"true\" allow_split=\"true\" full_page_opener=\"true\"\n:::\n\n::: columns flow=\"true\" keep_with_next=\"false\"\nA\n:::",
        )
        .expect("typst");
        assert!(typst.contains("keep_with_next: false"));
        assert!(typst.contains("break_before: true"));
        assert!(typst.contains("break_after: true"));
        assert!(typst.contains("allow_split: true"));
        assert!(typst.contains("full_page_opener: true"));
        assert!(typst.contains("#two_col(flow: true, keep_with_next: false)"));
    }

    #[test]
    fn unsupported_min_rows_is_rejected_instead_of_silently_ignored() {
        let error = markdown_to_typst("::: columns min_rows_before_break=\"2\"\nA\n:::")
            .expect_err("min rows");
        assert!(error.to_string().contains("min_rows_before_break"));
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
    fn renders_evidence_based_components() {
        let markdown = "::: recall\nSoru?\n:::\n\n::: explain label=\"NEDEN\"\nAçıklama.\n:::\n\n::: review-plan\n1 gün, 3 gün.\n:::\n";
        let typst = markdown_to_typst(markdown).expect("typst");
        assert!(typst.contains("#recall("));
        assert!(typst.contains("HATIRLA"));
        assert!(typst.contains("#explain("));
        assert!(typst.contains("NEDEN"));
        assert!(typst.contains("#review_plan("));
    }

    #[test]
    fn english_revision_labels_are_not_turkish() {
        let strings =
            crate::styles::strings(std::path::Path::new(""), "revision", "en").expect("en strings");
        let markdown = "::: recall\nQ\n:::\n\n::: synthesis\n- A\n:::\n\n::: organiser title=\"Unit\" source=\"PD0, s. 1\"\n## Topic\nText.\n:::\n";
        let typst = markdown_to_typst_styled_with_strings(markdown, &strings, HashMap::new())
            .expect("convert");
        assert!(typst.contains("RECALL"), "{typst}");
        assert!(typst.contains("EXAM REVISION"), "{typst}");
        assert!(typst.contains("KNOWLEDGE ORGANISER"), "{typst}");
        assert!(!typst.contains("HATIRLA"), "{typst}");
        assert!(!typst.contains("SINAV TEKRARI"), "{typst}");
        assert!(!typst.contains("BİLGİ ORGANİZATÖRÜ"), "{typst}");
    }

    #[test]
    fn rejects_raw_html_but_allows_comments() {
        assert!(markdown_to_typst("A <span>lost</span> word.").is_err());
        assert!(markdown_to_typst("<!-- covers: PD0:1.1 -->\n\nText.").is_ok());
    }

    #[test]
    fn allows_typst_like_text_inside_code_fences() {
        let markdown = "```c\n#include <stdio.h>\nint main(void) { return 0; }\n```\n";
        assert!(markdown_to_typst(markdown).is_ok());
    }

    #[test]
    fn renders_revision_components() {
        let markdown = "::: must-know\n- **A:** B.\n:::\n\n::: synthesis title=\"SENTEZ\"\n- A\n:::\n\n::: answer label=\"CEVAP\"\nB.\n:::\n\n::: organiser title=\"UNITE\" source=\"PD0, s. 1\"\n## Konu\nMetin.\n:::\n";
        let typst = markdown_to_typst(markdown).expect("typst");
        assert!(typst.contains("#must_know["));
        assert!(typst.contains("#synthesis_page("));
        assert!(typst.contains("SENTEZ"));
        assert!(typst.contains("#answer("));
        assert!(typst.contains("#organiser_page("));
        assert!(typst.contains("UNITE"));
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
    fn renders_table_metadata() {
        let typst = markdown_to_typst(
            "::: table caption=\"Derleme\" source=\"PD0, s. 106\" widths=\"20,80\"\n| A | B |\n|---|---|\n| 1 | 2 |\n:::\n",
        )
        .expect("typst");
        assert!(typst.contains("#table_block(caption: \"Derleme\""));
        assert!(typst.contains("source: \"PD0, s. 106\""));
        assert!(typst.contains("label: \"Tablo\""));
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
    fn rejects_the_checked_in_raw_typst_fixture() {
        let fixture = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/raw-typst.md"
        ));
        let error = markdown_to_typst(fixture).expect_err("must reject");
        assert!(error.to_string().contains("raw Typst"));
    }

    #[test]
    fn rejects_raw_typst_and_html() {
        let typst = markdown_to_typst("#let x = 1\n").expect_err("must reject");
        assert!(typst.to_string().contains("raw Typst"));
        let html = markdown_to_typst("<style>body{}</style>\n").expect_err("must reject");
        assert!(html.to_string().contains("raw Typst/HTML/CSS"));
        let fenced = markdown_to_typst("```typst\n#let x = 1\n```\n").expect_err("must reject");
        assert!(fenced.to_string().contains("raw Typst"));
    }

    #[test]
    fn rejects_unknown_directives() {
        let error = markdown_to_typst("::: mystery\nx\n:::\n").expect_err("must reject");
        assert!(matches!(error, Error::Markdown { .. }));
    }
}
