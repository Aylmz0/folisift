//! Versioned style packs. Packs select an engine-owned layout family and
//! supply localized labels; they never inject raw Typst.

use std::collections::BTreeSet;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{Error, Result};

const PAPER_STYLE: &str = include_str!("../../../styles/paper/style.json");
const PAPER_TOKENS: &str = include_str!("../../../styles/paper/tokens.json");
const PAPER_TR: &str = include_str!("../../../styles/paper/strings/tr.json");
const PAPER_EN: &str = include_str!("../../../styles/paper/strings/en.json");

const READER_STYLE: &str = include_str!("../../../styles/reader/style.json");
const READER_TOKENS: &str = include_str!("../../../styles/reader/tokens.json");
const READER_TR: &str = include_str!("../../../styles/reader/strings/tr.json");
const READER_EN: &str = include_str!("../../../styles/reader/strings/en.json");

const JOURNAL_STYLE: &str = include_str!("../../../styles/journal/style.json");
const JOURNAL_TOKENS: &str = include_str!("../../../styles/journal/tokens.json");
const JOURNAL_TR: &str = include_str!("../../../styles/journal/strings/tr.json");
const JOURNAL_EN: &str = include_str!("../../../styles/journal/strings/en.json");

const BRIEF_STYLE: &str = include_str!("../../../styles/brief/style.json");
const BRIEF_TOKENS: &str = include_str!("../../../styles/brief/tokens.json");
const BRIEF_TR: &str = include_str!("../../../styles/brief/strings/tr.json");
const BRIEF_EN: &str = include_str!("../../../styles/brief/strings/en.json");

const ACCESSIBLE_STYLE: &str = include_str!("../../../styles/accessible/style.json");
const ACCESSIBLE_TOKENS: &str = include_str!("../../../styles/accessible/tokens.json");
const ACCESSIBLE_TR: &str = include_str!("../../../styles/accessible/strings/tr.json");
const ACCESSIBLE_EN: &str = include_str!("../../../styles/accessible/strings/en.json");

const REVISION_STYLE: &str = include_str!("../../../styles/revision/style.json");
const REVISION_TOKENS: &str = include_str!("../../../styles/revision/tokens.json");
const REVISION_TR: &str = include_str!("../../../styles/revision/strings/tr.json");
const REVISION_EN: &str = include_str!("../../../styles/revision/strings/en.json");

const CLEAN_STYLE: &str = include_str!("../../../styles/clean/style.json");
const CLEAN_TOKENS: &str = include_str!("../../../styles/clean/tokens.json");
const CLEAN_TR: &str = include_str!("../../../styles/clean/strings/tr.json");
const CLEAN_EN: &str = include_str!("../../../styles/clean/strings/en.json");

const NOTES_STYLE: &str = include_str!("../../../styles/notes/style.json");
const NOTES_TOKENS: &str = include_str!("../../../styles/notes/tokens.json");
const NOTES_TR: &str = include_str!("../../../styles/notes/strings/tr.json");
const NOTES_EN: &str = include_str!("../../../styles/notes/strings/en.json");

const WORKBOOK_STYLE: &str = include_str!("../../../styles/workbook/style.json");
const WORKBOOK_TOKENS: &str = include_str!("../../../styles/workbook/tokens.json");
const WORKBOOK_TR: &str = include_str!("../../../styles/workbook/strings/tr.json");
const WORKBOOK_EN: &str = include_str!("../../../styles/workbook/strings/en.json");

const REFERENCE_STYLE: &str = include_str!("../../../styles/reference/style.json");
const REFERENCE_TOKENS: &str = include_str!("../../../styles/reference/tokens.json");
const REFERENCE_TR: &str = include_str!("../../../styles/reference/strings/tr.json");
const REFERENCE_EN: &str = include_str!("../../../styles/reference/strings/en.json");

const PALETTES: &[(&str, &str)] = &[
    (
        "warm-paper",
        include_str!("../../../styles/palettes/warm-paper.json"),
    ),
    (
        "sepia-archive",
        include_str!("../../../styles/palettes/sepia-archive.json"),
    ),
    ("clay", include_str!("../../../styles/palettes/clay.json")),
    ("berry", include_str!("../../../styles/palettes/berry.json")),
    (
        "rosewood",
        include_str!("../../../styles/palettes/rosewood.json"),
    ),
    (
        "terracotta",
        include_str!("../../../styles/palettes/terracotta.json"),
    ),
    ("cocoa", include_str!("../../../styles/palettes/cocoa.json")),
    (
        "olive-grove",
        include_str!("../../../styles/palettes/olive-grove.json"),
    ),
    (
        "arctic-blue",
        include_str!("../../../styles/palettes/arctic-blue.json"),
    ),
    (
        "plum-slate",
        include_str!("../../../styles/palettes/plum-slate.json"),
    ),
    (
        "indigo-lilac",
        include_str!("../../../styles/palettes/indigo-lilac.json"),
    ),
    (
        "cool-slate",
        include_str!("../../../styles/palettes/cool-slate.json"),
    ),
    (
        "academic-navy",
        include_str!("../../../styles/palettes/academic-navy.json"),
    ),
    (
        "cobalt",
        include_str!("../../../styles/palettes/cobalt.json"),
    ),
    (
        "forest-sage",
        include_str!("../../../styles/palettes/forest-sage.json"),
    ),
    (
        "ocean-teal",
        include_str!("../../../styles/palettes/ocean-teal.json"),
    ),
    (
        "graphite",
        include_str!("../../../styles/palettes/graphite.json"),
    ),
    (
        "mono-print",
        include_str!("../../../styles/palettes/mono-print.json"),
    ),
    (
        "high-contrast",
        include_str!("../../../styles/palettes/high-contrast.json"),
    ),
];

const FONT_PAIRINGS: &[(&str, &str)] = &[
    (
        "editorial",
        include_str!("../../../styles/fonts/editorial.json"),
    ),
    (
        "technical",
        include_str!("../../../styles/fonts/technical.json"),
    ),
];

const EMBEDDED_FAMILIES: &[&str] = &["Noto Serif", "Inter", "Noto Sans Mono"];
const TEXT_ROLES: &[&str] = &["ink", "muted", "soft", "faint", "accent", "secondary"];
const SURFACE_ROLES: &[&str] = &[
    "paper",
    "code_bg",
    "cover_bg",
    "cream",
    "zebra",
    "headerfill",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StylePack {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub status: String,
    pub layout: String,
    pub aesthetic: String,
    pub supported_structures: Vec<String>,
    pub supported_languages: Vec<String>,
    pub supported_domains: Vec<String>,
    pub supported_voices: Vec<String>,
    #[serde(default)]
    pub palette: Option<String>,
    #[serde(default)]
    pub fonts: Option<String>,
    #[serde(default)]
    pub supported_palettes: Vec<String>,
    #[serde(default)]
    pub supported_fonts: Vec<String>,
    pub tokens: String,
    pub strings: Vec<String>,
    pub engine_min: String,
    pub page_target: String,
    pub preview: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleStrings {
    pub toc: String,
    pub figure: String,
    #[serde(default = "default_table")]
    pub table: String,
    #[serde(default)]
    pub page: String,
    #[serde(default = "default_recall")]
    pub recall: String,
    #[serde(default = "default_explain")]
    pub explain: String,
    #[serde(default = "default_review_plan")]
    pub review_plan: String,
    #[serde(default = "default_synthesis")]
    pub synthesis: String,
    #[serde(default = "default_answer")]
    pub answer: String,
    #[serde(default = "default_revision_badge")]
    pub revision_badge: String,
    #[serde(default = "default_organiser_badge")]
    pub organiser_badge: String,
    #[serde(default = "default_summary_label")]
    pub summary: String,
    #[serde(default = "default_workspace_label")]
    pub workspace: String,
    #[serde(default = "default_answer_key_label")]
    pub answer_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaletteDefinition {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub family: String,
    #[serde(default, rename = "use")]
    pub usage: String,
    pub palette: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontPairing {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub status: String,
    pub body: String,
    pub heading: String,
    pub mono: String,
    #[serde(rename = "use")]
    pub usage: String,
}

#[derive(Debug, Clone)]
pub struct LoadedStyle {
    pub pack: StylePack,
    pub tokens: Value,
    pub strings: StyleStrings,
    pub palette: PaletteDefinition,
    pub fonts: FontPairing,
}

/// Built-in pack ids shipped in the binary.
pub fn builtin_ids() -> &'static [&'static str] {
    &[
        "paper",
        "reader",
        "clean",
        "journal",
        "brief",
        "accessible",
        "revision",
        "notes",
        "workbook",
        "reference",
    ]
}

/// List built-in packs plus workspace overrides under `<root>/styles`.
pub fn list(styles_root: &Path) -> Result<Vec<StylePack>> {
    let mut ids: BTreeSet<String> = builtin_ids().iter().map(|id| id.to_string()).collect();
    let dir = styles_root.join("styles");
    if dir.is_dir() {
        for entry in std::fs::read_dir(&dir).map_err(|error| io(&dir, error))? {
            let entry = entry.map_err(|error| io(&dir, error))?;
            if entry.path().join("style.json").is_file()
                && let Some(id) = entry.file_name().to_str()
            {
                ids.insert(id.to_string());
            }
        }
    }
    ids.iter()
        .map(|id| load(styles_root, id).map(|loaded| loaded.pack))
        .collect()
}

/// Load one pack, preferring a workspace override.
pub fn load(styles_root: &Path, id: &str) -> Result<LoadedStyle> {
    load_with(styles_root, id, None, None)
}

/// Load a pack and resolve an optional palette/font override.
///
/// The layout owns geometry; palette and font pairing are independent token
/// layers. Empty overrides fall back to the pack defaults in `style.json`.
#[derive(Debug, Clone, serde::Serialize)]
pub struct StyleCompatibility {
    pub style: String,
    pub name: String,
    pub compatible: bool,
    pub reason: String,
    pub page_target: String,
    pub supported_structures: Vec<String>,
    pub supported_domains: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct StyleEstimate {
    pub style: String,
    pub target_pages: Option<String>,
    pub words: usize,
    pub tables: usize,
    pub diagrams: usize,
    pub estimated_pages: String,
    pub over_budget: bool,
    pub reason: String,
}

/// List every style pack with a compatibility verdict for one structure/domain.
pub fn compatibility(
    styles_root: &Path,
    structure: &str,
    domain: &str,
) -> Result<Vec<StyleCompatibility>> {
    let mut out = Vec::new();
    for summary in list(styles_root)? {
        let loaded = load_with(styles_root, &summary.id, None, None)?;
        let missing_structure = !loaded
            .pack
            .supported_structures
            .iter()
            .any(|value| value == structure);
        let missing_domain = !loaded
            .pack
            .supported_domains
            .iter()
            .any(|value| value == domain);
        let compatible = !missing_structure && !missing_domain;
        let reason = if compatible {
            "structure and domain are supported".to_string()
        } else if missing_structure {
            format!(
                "structure {structure:?} is not supported; use one of {:?}",
                loaded.pack.supported_structures
            )
        } else {
            format!(
                "domain {domain:?} is not supported; use one of {:?}",
                loaded.pack.supported_domains
            )
        };
        out.push(StyleCompatibility {
            style: loaded.pack.id.clone(),
            name: loaded.pack.name.clone(),
            compatible,
            reason,
            page_target: loaded.pack.page_target.clone(),
            supported_structures: loaded.pack.supported_structures.clone(),
            supported_domains: loaded.pack.supported_domains.clone(),
        });
    }
    out.sort_by_key(|item| (!item.compatible, item.style.clone()));
    Ok(out)
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct StyleMatrixEntry {
    pub style: String,
    pub name: String,
    pub status: String,
    pub page_target: String,
    pub supported_structures: Vec<String>,
    pub supported_domains: Vec<String>,
    pub supported_languages: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compatible: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Every style pack with its declared matrix. When a structure/domain pair is
/// supplied, each row also carries a compatibility verdict.
pub fn matrix(
    styles_root: &Path,
    structure: Option<&str>,
    domain: Option<&str>,
) -> Result<Vec<StyleMatrixEntry>> {
    let mut out = Vec::new();
    for summary in list(styles_root)? {
        let loaded = load_with(styles_root, &summary.id, None, None)?;
        let structure_ok = structure.is_none_or(|value| {
            loaded
                .pack
                .supported_structures
                .iter()
                .any(|item| item == value)
        });
        let domain_ok = domain.is_none_or(|value| {
            loaded
                .pack
                .supported_domains
                .iter()
                .any(|item| item == value)
        });
        let compatible =
            (structure.is_some() || domain.is_some()).then_some(structure_ok && domain_ok);
        let reason = match (compatible, structure, domain) {
            (Some(true), _, _) => Some("structure and domain are supported".to_string()),
            (Some(false), Some(structure), _) if !structure_ok => Some(format!(
                "structure {structure:?} is not supported; use one of {:?}",
                loaded.pack.supported_structures
            )),
            (Some(false), _, Some(domain)) if !domain_ok => Some(format!(
                "domain {domain:?} is not supported; use one of {:?}",
                loaded.pack.supported_domains
            )),
            _ => None,
        };
        out.push(StyleMatrixEntry {
            style: loaded.pack.id.clone(),
            name: loaded.pack.name.clone(),
            status: loaded.pack.status.clone(),
            page_target: loaded.pack.page_target.clone(),
            supported_structures: loaded.pack.supported_structures.clone(),
            supported_domains: loaded.pack.supported_domains.clone(),
            supported_languages: loaded.pack.supported_languages.clone(),
            compatible,
            reason,
        });
    }
    out.sort_by(|left, right| left.style.cmp(&right.style));
    Ok(out)
}

/// Validate a profile and make a rejected pair actionable.
pub fn validate_profile_actionable(
    styles_root: &Path,
    pack: &StylePack,
    profile: &folisift_core::StyleProfile,
) -> Result<()> {
    match validate_profile(pack, profile) {
        Ok(()) => Ok(()),
        Err(error) => {
            let compatible = compatibility(styles_root, &profile.structure, &profile.domain)?
                .into_iter()
                .filter(|item| item.compatible)
                .map(|item| item.style)
                .collect::<Vec<_>>();
            Err(Error::StyleConfig(format!(
                "{error}; compatible_styles: {compatible:?}; next: choose a compatible style or update its matrix"
            )))
        }
    }
}

/// Choose the closest compatible style for a structure/domain pair.
pub fn closest_compatible(
    styles_root: &Path,
    structure: &str,
    domain: &str,
) -> Result<Option<StyleCompatibility>> {
    Ok(compatibility(styles_root, structure, domain)?
        .into_iter()
        .find(|item| item.compatible))
}

/// Estimate pages before a full build. The estimate is deliberately coarse and
/// deterministic: it uses body words plus table/diagram costs, never a model.
pub fn estimate(style: &str, markdown: &str, target_pages: Option<&str>) -> StyleEstimate {
    let body = markdown
        .lines()
        .filter(|line| {
            let line = line.trim();
            !line.is_empty()
                && !line.starts_with('#')
                && !line.starts_with(":::")
                && !line.starts_with('|')
                && !line.starts_with("![")
                && !line.starts_with("<!--")
        })
        .collect::<Vec<_>>()
        .join(" ");
    let words = body.split_whitespace().count();
    let mut wrapped_tables = 0usize;
    let mut markdown_tables = 0usize;
    let mut glossary_tables = 0usize;
    let mut in_wrapped = false;
    for line in markdown.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("::: table") {
            wrapped_tables += 1;
            in_wrapped = true;
        } else if trimmed.starts_with("::: glossary") {
            glossary_tables += 1;
            in_wrapped = true;
        } else if trimmed.starts_with(":::") {
            in_wrapped = false;
        } else if !in_wrapped && is_markdown_table_separator(trimmed) {
            markdown_tables += 1;
        }
    }
    let tables = wrapped_tables + markdown_tables + glossary_tables;
    let diagrams = markdown
        .lines()
        .filter(|line| line.trim_start().starts_with("::: diagram"))
        .count();
    let factor = match style {
        "accessible" => 1.35,
        "workbook" => 1.25,
        "notes" => 1.10,
        "revision" => 0.55,
        "reference" => 0.70,
        "clean" | "paper" | "reader" | "journal" | "brief" => 1.0,
        _ => 1.0,
    };
    let page_words = 300.0;
    let base = (words as f64 / page_words * factor).ceil().max(1.0);
    let extra = tables as f64 * 0.20 + diagrams as f64 * 0.35;
    let low = (base + extra).ceil().max(1.0) as usize;
    let high = ((base + extra) * 1.20).ceil().max(1.0) as usize;
    let estimated = if low == high {
        low.to_string()
    } else {
        format!("{low}-{high}")
    };
    let over_budget = target_pages
        .is_some_and(|target| parse_page_target(target).is_some_and(|(_, max)| high > max));
    let reason = if over_budget {
        format!(
            "estimated {estimated} pages exceeds the {target} target; cut or move a section before building",
            target = target_pages.unwrap_or_default()
        )
    } else {
        format!("{words} words, {tables} wrapped tables, {diagrams} diagrams")
    };
    StyleEstimate {
        style: style.to_string(),
        target_pages: target_pages.map(ToString::to_string),
        words,
        tables,
        diagrams,
        estimated_pages: estimated,
        over_budget,
        reason,
    }
}

fn is_markdown_table_separator(line: &str) -> bool {
    line.starts_with('|')
        && line.contains("---")
        && line.chars().all(|character| {
            character == '|' || character == '-' || character == ':' || character.is_whitespace()
        })
}

fn parse_page_target(value: &str) -> Option<(usize, usize)> {
    let value = value.trim();
    if let Some((min, max)) = value.split_once('-') {
        let min = min.trim().parse().ok()?;
        let max = max.trim().parse().ok()?;
        Some((min, max))
    } else {
        let pages = value.parse().ok()?;
        Some((pages, pages))
    }
}

pub fn load_with(
    styles_root: &Path,
    id: &str,
    palette_override: Option<&str>,
    fonts_override: Option<&str>,
) -> Result<LoadedStyle> {
    validate_pack_id(id)?;
    let style_text = read_pack_file(styles_root, id, "style.json")?;
    let pack: StylePack = serde_json::from_str(&style_text).map_err(|error| Error::Style {
        id: id.to_string(),
        message: format!("style.json: {error}"),
    })?;
    if pack.id != id {
        return Err(Error::Style {
            id: id.to_string(),
            message: format!(
                "style.json id {:?} does not match directory {id:?}",
                pack.id
            ),
        });
    }
    if !matches!(pack.schema_version, 1 | 2) {
        return Err(Error::Style {
            id: id.to_string(),
            message: format!("unsupported style schema {}", pack.schema_version),
        });
    }
    let palette_id = choose_design(
        id,
        "palette",
        palette_override,
        pack.palette.as_deref(),
        default_palette(&pack.layout),
        &pack.supported_palettes,
    )?;
    let fonts_id = choose_design(
        id,
        "fonts",
        fonts_override,
        pack.fonts.as_deref(),
        default_fonts(&pack.layout),
        &pack.supported_fonts,
    )?;
    let palette = load_palette(styles_root, &palette_id)?;
    let fonts = load_fonts(styles_root, &fonts_id)?;
    if matches!(pack.layout.as_str(), "clean" | "revision") && fonts.body != fonts.heading {
        return Err(Error::Style {
            id: id.to_string(),
            message: format!(
                "font pairing {:?} needs explicit body/heading markers for layout {:?}",
                fonts.id, pack.layout
            ),
        });
    }
    let tokens_text = read_pack_file(styles_root, id, &pack.tokens)?;
    let mut tokens: Value = serde_json::from_str(&tokens_text).map_err(|error| Error::Style {
        id: id.to_string(),
        message: format!("tokens: {error}"),
    })?;
    let object = tokens.as_object_mut().ok_or_else(|| Error::Style {
        id: id.to_string(),
        message: "tokens must be a JSON object".to_string(),
    })?;
    object.insert("palette".to_string(), palette.palette.clone());
    let typography = object
        .entry("typography".to_string())
        .or_insert_with(|| Value::Object(serde_json::Map::new()));
    if let Some(typography) = typography.as_object_mut() {
        typography.insert("body".to_string(), Value::String(fonts.body.clone()));
        typography.insert("heading".to_string(), Value::String(fonts.heading.clone()));
        typography.insert("mono".to_string(), Value::String(fonts.mono.clone()));
    }
    validate_accessibility(id, &tokens)?;
    let language = pack
        .supported_languages
        .first()
        .cloned()
        .unwrap_or_else(|| "tr".to_string());
    let strings = strings(styles_root, id, &language)?;
    Ok(LoadedStyle {
        pack,
        tokens,
        strings,
        palette,
        fonts,
    })
}

fn choose_design(
    id: &str,
    axis: &str,
    requested: Option<&str>,
    configured: Option<&str>,
    fallback: &str,
    supported: &[String],
) -> Result<String> {
    let choice = requested
        .filter(|value| !value.trim().is_empty())
        .or_else(|| configured.filter(|value| !value.trim().is_empty()))
        .unwrap_or(fallback);
    validate_design_id(choice, axis, id)?;
    if !supported.is_empty() && !supported.iter().any(|value| value == choice) {
        return Err(Error::Style {
            id: id.to_string(),
            message: format!(
                "{axis} {choice:?} is not supported by this pack; use one of {supported:?}"
            ),
        });
    }
    Ok(choice.to_string())
}

fn default_palette(layout: &str) -> &'static str {
    match layout {
        "clean" => "cool-slate",
        "journal" => "academic-navy",
        "accessible" => "high-contrast",
        _ => "warm-paper",
    }
}

fn default_fonts(layout: &str) -> &'static str {
    match layout {
        "clean" | "revision" => "technical",
        _ => "editorial",
    }
}

fn validate_design_id(value: &str, axis: &str, pack: &str) -> Result<()> {
    let valid = !value.is_empty()
        && value.len() <= 64
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-' || byte == b'_'
        });
    if valid {
        Ok(())
    } else {
        Err(Error::Style {
            id: pack.to_string(),
            message: format!("{axis} id must be a simple lowercase name: {value:?}"),
        })
    }
}

/// List built-in and workspace palette definitions.
pub fn list_palettes(styles_root: &Path) -> Result<Vec<PaletteDefinition>> {
    list_definitions(styles_root, "palettes", PALETTES)
        .into_iter()
        .map(|(id, text)| parse_palette(&id, &text))
        .collect()
}

/// List built-in and workspace font pairings.
pub fn list_fonts(styles_root: &Path) -> Result<Vec<FontPairing>> {
    list_definitions(styles_root, "fonts", FONT_PAIRINGS)
        .into_iter()
        .map(|(id, text)| parse_fonts(&id, &text))
        .collect()
}

fn list_definitions(
    styles_root: &Path,
    kind: &str,
    builtins: &'static [(&'static str, &'static str)],
) -> Vec<(String, String)> {
    let mut definitions = builtins
        .iter()
        .map(|(id, text)| ((*id).to_string(), (*text).to_string()))
        .collect::<std::collections::BTreeMap<_, _>>();
    let dir = styles_root.join("styles").join(kind);
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) == Some("json")
                && let Some(id) = path.file_stem().and_then(|value| value.to_str())
                && let Ok(text) = std::fs::read_to_string(&path)
            {
                definitions.insert(id.to_string(), text);
            }
        }
    }
    definitions.into_iter().collect()
}

/// Load one palette by id.
pub fn load_palette(styles_root: &Path, id: &str) -> Result<PaletteDefinition> {
    validate_design_id(id, "palette", id)?;
    let path = styles_root
        .join("styles/palettes")
        .join(format!("{id}.json"));
    let text = if path.is_file() {
        std::fs::read_to_string(&path).map_err(|error| io(&path, error))?
    } else {
        PALETTES
            .iter()
            .find(|(candidate, _)| *candidate == id)
            .map(|(_, text)| (*text).to_string())
            .ok_or_else(|| Error::Style {
                id: id.to_string(),
                message: format!("unknown palette {id:?}"),
            })?
    };
    let palette = parse_palette(id, &text)?;
    validate_palette_contrast(&palette)?;
    Ok(palette)
}

fn parse_palette(id: &str, text: &str) -> Result<PaletteDefinition> {
    let palette: PaletteDefinition = serde_json::from_str(text).map_err(|error| Error::Style {
        id: id.to_string(),
        message: format!("palette {id:?}: {error}"),
    })?;
    if palette.id != id {
        return Err(Error::Style {
            id: id.to_string(),
            message: format!("palette id {:?} does not match {id:?}", palette.id),
        });
    }
    if palette.schema_version != 1 {
        return Err(Error::Style {
            id: id.to_string(),
            message: format!("unsupported palette schema {}", palette.schema_version),
        });
    }
    Ok(palette)
}

/// Load one font pairing by id.
pub fn load_fonts(styles_root: &Path, id: &str) -> Result<FontPairing> {
    validate_design_id(id, "fonts", id)?;
    let path = styles_root.join("styles/fonts").join(format!("{id}.json"));
    let text = if path.is_file() {
        std::fs::read_to_string(&path).map_err(|error| io(&path, error))?
    } else {
        FONT_PAIRINGS
            .iter()
            .find(|(candidate, _)| *candidate == id)
            .map(|(_, text)| (*text).to_string())
            .ok_or_else(|| Error::Style {
                id: id.to_string(),
                message: format!("unknown font pairing {id:?}"),
            })?
    };
    parse_fonts(id, &text)
}

fn parse_fonts(id: &str, text: &str) -> Result<FontPairing> {
    let fonts: FontPairing = serde_json::from_str(text).map_err(|error| Error::Style {
        id: id.to_string(),
        message: format!("font pairing {id:?}: {error}"),
    })?;
    if fonts.id != id {
        return Err(Error::Style {
            id: id.to_string(),
            message: format!("font pairing id {:?} does not match {id:?}", fonts.id),
        });
    }
    if fonts.schema_version != 1 || fonts.status != "embedded" {
        return Err(Error::Style {
            id: id.to_string(),
            message: "only schema 1 embedded pairings are allowed".to_string(),
        });
    }
    for family in [&fonts.body, &fonts.heading, &fonts.mono] {
        if !EMBEDDED_FAMILIES.contains(&family.as_str()) {
            return Err(Error::Style {
                id: id.to_string(),
                message: format!(
                    "font family {family:?} is not embedded; embedded: {EMBEDDED_FAMILIES:?}"
                ),
            });
        }
    }
    Ok(fonts)
}

fn validate_palette_contrast(palette: &PaletteDefinition) -> Result<()> {
    let color = |role: &str| -> Result<(u8, u8, u8)> {
        palette
            .palette
            .get(role)
            .and_then(Value::as_str)
            .and_then(parse_hex)
            .ok_or_else(|| Error::Style {
                id: palette.id.clone(),
                message: format!("palette.{role} is missing or not #RRGGBB"),
            })
    };
    let paper = color("paper")?;
    for role in TEXT_ROLES {
        let foreground = color(role)?;
        let ratio = contrast_ratio(foreground, paper);
        if ratio < 4.5 {
            return Err(Error::Style {
                id: palette.id.clone(),
                message: format!("palette.{role}/paper contrast is {ratio:.2}:1, need 4.5:1"),
            });
        }
    }
    let ink = color("ink")?;
    for role in SURFACE_ROLES {
        let background = color(role)?;
        let ratio = contrast_ratio(ink, background);
        if ratio < 4.5 {
            return Err(Error::Style {
                id: palette.id.clone(),
                message: format!("palette.ink/{role} contrast is {ratio:.2}:1, need 4.5:1"),
            });
        }
    }
    Ok(())
}

/// Load localized labels for a pack.
pub fn strings(styles_root: &Path, id: &str, language: &str) -> Result<StyleStrings> {
    validate_pack_id(id)?;
    if !matches!(language, "tr" | "en") {
        return Err(Error::Style {
            id: id.to_string(),
            message: format!("unsupported language {language:?}"),
        });
    }
    let path = format!("strings/{language}.json");
    let text = read_pack_file(styles_root, id, &path)?;
    serde_json::from_str(&text).map_err(|error| Error::Style {
        id: id.to_string(),
        message: format!("{path}: {error}"),
    })
}

/// Load the canonical preview Markdown for a pack and language.
pub fn preview(styles_root: &Path, id: &str, language: &str) -> Result<String> {
    let loaded = load(styles_root, id)?;
    let localized = format!("preview.{language}.md");
    if read_pack_file(styles_root, id, &localized).is_ok() {
        return read_pack_file(styles_root, id, &localized);
    }
    read_pack_file(styles_root, id, &loaded.pack.preview)
}

fn validate_accessibility(id: &str, tokens: &Value) -> Result<()> {
    let palette = tokens
        .get("palette")
        .and_then(Value::as_object)
        .ok_or_else(|| Error::Style {
            id: id.to_string(),
            message: "tokens.palette is required".to_string(),
        })?;
    let color = |key: &str| -> Result<(u8, u8, u8)> {
        let value = palette
            .get(key)
            .and_then(Value::as_str)
            .ok_or_else(|| Error::Style {
                id: id.to_string(),
                message: format!("tokens.palette.{key} is required"),
            })?;
        parse_hex(value).ok_or_else(|| Error::Style {
            id: id.to_string(),
            message: format!("tokens.palette.{key} is not #RRGGBB"),
        })
    };
    let ink = color("ink")?;
    let paper = color("paper")?;
    if contrast_ratio(ink, paper) < 4.5 {
        return Err(Error::Style {
            id: id.to_string(),
            message: "ink/paper contrast is below 4.5:1".to_string(),
        });
    }
    let size = tokens
        .get("typography")
        .and_then(|value| value.get("body_size"))
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    if size < 9.0 {
        return Err(Error::Style {
            id: id.to_string(),
            message: "body_size must be at least 9 pt".to_string(),
        });
    }
    Ok(())
}

fn parse_hex(value: &str) -> Option<(u8, u8, u8)> {
    let value = value.strip_prefix('#')?;
    if value.len() != 6 {
        return None;
    }
    let channel = |range: std::ops::Range<usize>| u8::from_str_radix(&value[range], 16).ok();
    Some((channel(0..2)?, channel(2..4)?, channel(4..6)?))
}

fn contrast_ratio(first: (u8, u8, u8), second: (u8, u8, u8)) -> f64 {
    let luminance = |color: (u8, u8, u8)| {
        let channel = |value: u8| {
            let value = f64::from(value) / 255.0;
            if value <= 0.039_28 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * channel(color.0) + 0.7152 * channel(color.1) + 0.0722 * channel(color.2)
    };
    let first = luminance(first);
    let second = luminance(second);
    let (lighter, darker) = if first > second {
        (first, second)
    } else {
        (second, first)
    };
    (lighter + 0.05) / (darker + 0.05)
}

fn validate_pack_id(id: &str) -> Result<()> {
    let valid = !id.is_empty()
        && id.len() <= 64
        && id.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-' || byte == b'_'
        });
    if valid {
        Ok(())
    } else {
        Err(Error::Style {
            id: id.to_string(),
            message: "style id must be a simple lowercase name".to_string(),
        })
    }
}

fn io(path: &Path, source: std::io::Error) -> Error {
    Error::Io {
        path: path.to_path_buf(),
        source,
    }
}

fn read_pack_file(styles_root: &Path, id: &str, file: &str) -> Result<String> {
    validate_pack_id(id)?;
    let path = styles_root.join("styles").join(id).join(file);
    if path.is_file() {
        return std::fs::read_to_string(&path).map_err(|error| io(&path, error));
    }
    embedded(id, file)
        .map(ToString::to_string)
        .ok_or_else(|| Error::Style {
            id: id.to_string(),
            message: format!("missing {file} (workspace override or built-in)"),
        })
}

fn embedded(id: &str, file: &str) -> Option<&'static str> {
    match (id, file) {
        ("paper", "style.json") => Some(PAPER_STYLE),
        ("paper", "tokens.json") => Some(PAPER_TOKENS),
        ("paper", "strings/tr.json") => Some(PAPER_TR),
        ("paper", "strings/en.json") => Some(PAPER_EN),
        ("paper", "preview.md") => Some(include_str!("../../../styles/paper/preview.md")),
        ("paper", "preview.en.md") => Some(include_str!("../../../styles/paper/preview.en.md")),
        ("reader", "style.json") => Some(READER_STYLE),
        ("reader", "tokens.json") => Some(READER_TOKENS),
        ("reader", "strings/tr.json") => Some(READER_TR),
        ("reader", "strings/en.json") => Some(READER_EN),
        ("reader", "preview.md") => Some(include_str!("../../../styles/reader/preview.md")),
        ("reader", "preview.en.md") => Some(include_str!("../../../styles/reader/preview.en.md")),
        ("journal", "style.json") => Some(JOURNAL_STYLE),
        ("journal", "tokens.json") => Some(JOURNAL_TOKENS),
        ("journal", "strings/tr.json") => Some(JOURNAL_TR),
        ("journal", "strings/en.json") => Some(JOURNAL_EN),
        ("journal", "preview.md") => Some(include_str!("../../../styles/journal/preview.md")),
        ("journal", "preview.en.md") => Some(include_str!("../../../styles/journal/preview.en.md")),
        ("brief", "style.json") => Some(BRIEF_STYLE),
        ("brief", "tokens.json") => Some(BRIEF_TOKENS),
        ("brief", "strings/tr.json") => Some(BRIEF_TR),
        ("brief", "strings/en.json") => Some(BRIEF_EN),
        ("brief", "preview.md") => Some(include_str!("../../../styles/brief/preview.md")),
        ("brief", "preview.en.md") => Some(include_str!("../../../styles/brief/preview.en.md")),
        ("accessible", "style.json") => Some(ACCESSIBLE_STYLE),
        ("accessible", "tokens.json") => Some(ACCESSIBLE_TOKENS),
        ("accessible", "strings/tr.json") => Some(ACCESSIBLE_TR),
        ("accessible", "strings/en.json") => Some(ACCESSIBLE_EN),
        ("accessible", "preview.md") => Some(include_str!("../../../styles/accessible/preview.md")),
        ("accessible", "preview.en.md") => {
            Some(include_str!("../../../styles/accessible/preview.en.md"))
        }
        ("revision", "style.json") => Some(REVISION_STYLE),
        ("revision", "tokens.json") => Some(REVISION_TOKENS),
        ("revision", "strings/tr.json") => Some(REVISION_TR),
        ("revision", "strings/en.json") => Some(REVISION_EN),
        ("revision", "preview.md") => Some(include_str!("../../../styles/revision/preview.md")),
        ("revision", "preview.en.md") => {
            Some(include_str!("../../../styles/revision/preview.en.md"))
        }
        ("notes", "style.json") => Some(NOTES_STYLE),
        ("notes", "tokens.json") => Some(NOTES_TOKENS),
        ("notes", "strings/tr.json") => Some(NOTES_TR),
        ("notes", "strings/en.json") => Some(NOTES_EN),
        ("notes", "preview.md") => Some(include_str!("../../../styles/notes/preview.md")),
        ("notes", "preview.en.md") => Some(include_str!("../../../styles/notes/preview.en.md")),
        ("workbook", "style.json") => Some(WORKBOOK_STYLE),
        ("workbook", "tokens.json") => Some(WORKBOOK_TOKENS),
        ("workbook", "strings/tr.json") => Some(WORKBOOK_TR),
        ("workbook", "strings/en.json") => Some(WORKBOOK_EN),
        ("workbook", "preview.md") => Some(include_str!("../../../styles/workbook/preview.md")),
        ("workbook", "preview.en.md") => {
            Some(include_str!("../../../styles/workbook/preview.en.md"))
        }
        ("reference", "style.json") => Some(REFERENCE_STYLE),
        ("reference", "tokens.json") => Some(REFERENCE_TOKENS),
        ("reference", "strings/tr.json") => Some(REFERENCE_TR),
        ("reference", "strings/en.json") => Some(REFERENCE_EN),
        ("reference", "preview.md") => Some(include_str!("../../../styles/reference/preview.md")),
        ("reference", "preview.en.md") => {
            Some(include_str!("../../../styles/reference/preview.en.md"))
        }
        ("clean", "style.json") => Some(CLEAN_STYLE),
        ("clean", "tokens.json") => Some(CLEAN_TOKENS),
        ("clean", "strings/tr.json") => Some(CLEAN_TR),
        ("clean", "strings/en.json") => Some(CLEAN_EN),
        ("clean", "preview.md") => Some(include_str!("../../../styles/clean/preview.md")),
        ("clean", "preview.en.md") => Some(include_str!("../../../styles/clean/preview.en.md")),
        _ => None,
    }
}

/// Apply palette tokens to a layout template.
///
/// The layout owns geometry and component structure; tokens own the named
/// palette roles. Unknown roles are left at the template defaults.
pub fn apply_tokens(template: &str, tokens: &Value, layout: &str) -> Result<String> {
    let mut out = apply_palette(template, tokens, layout)?;
    let typography = tokens.get("typography");
    for (key, original) in [
        ("body", "\"Noto Serif\""),
        ("heading", "\"Inter\""),
        ("mono", "\"Noto Sans Mono\""),
    ] {
        if let Some(family) = typography
            .and_then(|value| value.get(key))
            .and_then(Value::as_str)
        {
            out = out.replace(original, &format!("\"{}\"", family.replace('"', "")));
        }
    }
    if matches!(
        layout,
        "paper" | "clean" | "notes" | "workbook" | "reference"
    ) {
        let code_block = typography
            .and_then(|value| value.get("code_block_size"))
            .and_then(Value::as_f64)
            .unwrap_or(7.4);
        let code_inline = typography
            .and_then(|value| value.get("code_inline_size"))
            .and_then(Value::as_f64)
            .unwrap_or(8.4);
        out = out.replace("__CODE_BLOCK_SIZE__", &format!("{code_block:.1}pt"));
        out = out.replace("__CODE_INLINE_SIZE__", &format!("{code_inline:.1}pt"));
    }
    if let Some(leading) = typography
        .and_then(|value| value.get("leading_em"))
        .and_then(Value::as_f64)
        && (0.4..=1.2).contains(&leading)
    {
        out = out.replace("leading: 0.62em", &format!("leading: {leading}em"));
    }
    if let Some(layout) = tokens.get("layout") {
        let top = layout.get("margin_top_mm").and_then(Value::as_f64);
        let bottom = layout.get("margin_bottom_mm").and_then(Value::as_f64);
        let side = layout.get("margin_side_mm").and_then(Value::as_f64);
        if let (Some(top), Some(bottom), Some(side)) = (top, bottom, side) {
            out = out.replace(
                "margin: (top: 16mm, bottom: 14mm, left: 16.5mm, right: 16.5mm)",
                &format!(
                    "margin: (top: {top}mm, bottom: {bottom}mm, left: {side}mm, right: {side}mm)"
                ),
            );
        }
    }
    if let Some(spacing) = tokens.get("spacing") {
        if let Some(par) = spacing.get("par_mm").and_then(Value::as_f64) {
            out = out.replace("spacing: 1.8mm", &format!("spacing: {par}mm"));
        }
        if let Some(list) = spacing.get("list_mm").and_then(Value::as_f64) {
            out = out.replace("spacing: 3.2mm", &format!("spacing: {list}mm"));
        }
    }
    if let Some(toc) = tokens.get("toc") {
        if let Some(inset) = toc.get("row_inset_mm").and_then(Value::as_f64) {
            out = out.replace("inset: (y: 1.9mm)", &format!("inset: (y: {inset}mm)"));
        }
        if let Some(title) = toc.get("title_pt").and_then(Value::as_f64)
            && let Some(sub) = toc.get("sub_pt").and_then(Value::as_f64)
        {
            out = out.replace(
                "size: if sub { 8.9pt } else { 9.6pt }",
                &format!("size: if sub {{ {sub}pt }} else {{ {title}pt }}"),
            );
        }
        if let Some(page) = toc.get("page_pt").and_then(Value::as_f64) {
            out = out.replace("size: 8.2pt", &format!("size: {page}pt"));
        }
    }
    out = apply_type_placeholders(out, tokens);
    Ok(out)
}

/// Replace explicit role placeholders with the resolved type scale. Every
/// layout uses the same role names; a pack may pin legacy or accessibility
/// sizes through `typography.sizes` without a separate multiplier.
fn apply_type_placeholders(mut out: String, tokens: &Value) -> String {
    let typography = tokens.get("typography");
    let body = typography
        .and_then(|value| value.get("body_size"))
        .and_then(Value::as_f64)
        .unwrap_or(11.0)
        .clamp(9.0, 18.0);
    let ratio = |key: &str, default: f64| {
        typography
            .and_then(|value| value.get("scale"))
            .and_then(|value| value.get(key))
            .and_then(Value::as_f64)
            .unwrap_or(default)
    };
    let floor = |key: &str, default: f64| {
        typography
            .and_then(|value| value.get("floor"))
            .and_then(|value| value.get(key))
            .and_then(Value::as_f64)
            .unwrap_or(default)
    };
    let component = floor("component_pt", 8.0);
    let table_floor = floor("table_pt", 9.0);
    let explicit = |key: &str| {
        typography
            .and_then(|value| value.get("sizes"))
            .and_then(|value| value.get(key))
            .and_then(Value::as_f64)
    };
    let format_size = |value: f64, precise: bool| {
        let rounded = if precise {
            (value * 100.0).round() / 100.0
        } else {
            (value * 10.0).round() / 10.0
        };
        if precise && (rounded * 10.0).fract().abs() > 0.001 {
            format!("{rounded:.2}pt")
        } else {
            format!("{rounded:.1}pt")
        }
    };
    let role = |key: &str, default_ratio: f64, floor: f64| {
        explicit(key).map_or_else(
            || format_size((body * ratio(key, default_ratio)).max(floor), false),
            |value| format_size(value, true),
        )
    };
    let body_size = format_size(body, false);

    for (placeholder, value) in [
        ("__BODY_SIZE__", body_size),
        ("__FIG_LABEL_SIZE__", role("fig_label", 0.716, component)),
        ("__RUNNING_SIZE__", role("running", 0.737, component)),
        ("__TABLE_SIZE__", role("table", 0.86, table_floor)),
        ("__CAPTION_SIZE__", role("caption", 0.86, table_floor)),
        ("__LABEL_SIZE__", role("label", 0.78, component)),
        ("__SOURCE_SIZE__", role("source", 0.78, component)),
        ("__MICRO_SIZE__", role("micro", 0.758, component)),
        ("__SMALL_SIZE__", role("small", 0.779, component)),
        ("__UI_SIZE__", role("ui", 0.842, component)),
        ("__H3_SIZE__", role("h3", 1.1, component)),
        ("__NOTE_SIZE__", role("note", 0.937, component)),
        ("__NOTE_SMALL_SIZE__", role("note_small", 0.905, component)),
        ("__BOX_SIZE__", role("box", 0.989, component)),
        ("__TWO_COL_SIZE__", role("two_col", 0.868, component)),
        ("__LEAD_SIZE__", role("lead", 1.12, component)),
        ("__LEAD_LARGE_SIZE__", role("lead_large", 1.179, component)),
        ("__H2_SIZE__", role("h2", 1.4, component)),
        ("__H1_SIZE__", role("h1", 2.0, component)),
        ("__H1_LARGE_SIZE__", role("h1_large", 2.211, component)),
        ("__TINY_SIZE__", role("tiny", 0.695, component)),
        (
            "__COVER_TITLE_SIZE__",
            role("cover_title", 3.474, component),
        ),
        (
            "__COVER_SUBTITLE_SIZE__",
            role("cover_subtitle", 1.789, component),
        ),
        ("__CODE_SIZE__", role("code_block", 0.92, table_floor)),
        (
            "__CODE_INLINE_SIZE__",
            role("code_inline", 0.95, table_floor),
        ),
        ("__TOC_TITLE_SIZE__", role("toc_title", 0.95, component)),
        ("__TOC_SUB_SIZE__", role("toc_sub", 0.88, component)),
        ("__TOC_PAGE_SIZE__", role("toc_page", 0.82, component)),
    ] {
        out = out.replace(placeholder, &value);
    }

    let leading = typography
        .and_then(|value| value.get("leading_em"))
        .and_then(Value::as_f64)
        .unwrap_or(0.72)
        .clamp(0.4, 1.2);
    out = out.replace("__LEADING__", &format!("{leading}em"));

    let par = tokens
        .get("spacing")
        .and_then(|value| value.get("par_mm"))
        .and_then(Value::as_f64)
        .unwrap_or(2.4);
    let list = tokens
        .get("spacing")
        .and_then(|value| value.get("list_mm"))
        .and_then(Value::as_f64)
        .unwrap_or(3.6);
    out = out.replace("__PAR_SPACING__", &format!("{par}mm"));
    out = out.replace("__LIST_SPACING__", &format!("{list}mm"));

    let toc_inset = tokens
        .get("toc")
        .and_then(|value| value.get("row_inset_mm"))
        .and_then(Value::as_f64)
        .unwrap_or(1.4);
    out = out.replace("__TOC_ROW_INSET__", &format!("{toc_inset}mm"));

    let layout = tokens.get("layout");
    for (placeholder, key, default) in [
        ("__MARGIN_TOP__", "margin_top_mm", 18.0),
        ("__MARGIN_BOTTOM__", "margin_bottom_mm", 16.0),
        ("__MARGIN_SIDE__", "margin_side_mm", 32.0),
    ] {
        let value = layout
            .and_then(|value| value.get(key))
            .and_then(Value::as_f64)
            .unwrap_or(default);
        out = out.replace(placeholder, &format!("{value}mm"));
    }
    out
}

pub fn apply_palette(template: &str, tokens: &Value, layout: &str) -> Result<String> {
    let palette = tokens
        .get("palette")
        .and_then(Value::as_object)
        .ok_or_else(|| Error::Style {
            id: layout.to_string(),
            message: "tokens.palette is required".to_string(),
        })?;
    let defaults: &[(&str, &str)] = match layout {
        "paper" => &[
            ("paper", "#FAF6EC"),
            ("ink", "#211E19"),
            ("muted", "#5E574B"),
            ("soft", "#6B6152"),
            ("faint", "#75664E"),
            ("accent", "#8A3E2F"),
            ("secondary", "#42594F"),
            ("gold", "#B08A4F"),
            ("rule", "#D8CEBD"),
            ("code_bg", "#F1EADB"),
            ("cover_bg", "#F3EBDD"),
            ("cream", "#F4ECDE"),
            ("zebra", "#F7F1E5"),
            ("headerfill", "#EFE7D8"),
        ],
        "revision" => &[
            ("paper", "#FAF6EC"),
            ("ink", "#211E19"),
            ("muted", "#5E574B"),
            ("soft", "#6B6152"),
            ("faint", "#75664E"),
            ("accent", "#8A3E2F"),
            ("secondary", "#42594F"),
            ("gold", "#B08A4F"),
            ("rule", "#D8CEBD"),
            ("code_bg", "#F1EADB"),
            ("cover_bg", "#F3EBDD"),
            ("cream", "#F4ECDE"),
            ("zebra", "#F7F1E5"),
            ("headerfill", "#EFE7D8"),
        ],
        "clean" | "notes" | "workbook" | "reference" => &[
            ("paper", "#FFFFFF"),
            ("ink", "#111827"),
            ("muted", "#4B5563"),
            ("soft", "#374151"),
            ("faint", "#6B7280"),
            ("accent", "#1D4ED8"),
            ("secondary", "#0F766E"),
            ("gold", "#B45309"),
            ("rule", "#D1D5DB"),
            ("code_bg", "#F3F4F6"),
            ("cover_bg", "#FFFFFF"),
            ("cream", "#EFF6FF"),
            ("zebra", "#F9FAFB"),
            ("headerfill", "#E5E7EB"),
        ],
        _ => &[],
    };
    let mut out = template.to_string();
    for (role, old) in defaults {
        if let Some(value) = palette.get(*role).and_then(Value::as_str) {
            let value = normalize_hex(value).ok_or_else(|| Error::Style {
                id: layout.to_string(),
                message: format!("tokens.palette.{role} is not #RRGGBB"),
            })?;
            if !value.eq_ignore_ascii_case(old) {
                out = out.replace(old, &value);
            }
        }
    }
    Ok(out)
}

fn normalize_hex(value: &str) -> Option<String> {
    let value = value.strip_prefix('#')?;
    if value.len() != 6 || !value.chars().all(|character| character.is_ascii_hexdigit()) {
        return None;
    }
    Some(format!("#{}", value.to_ascii_uppercase()))
}

/// Validate that a pack supports the requested profile.
pub fn validate_profile(pack: &StylePack, profile: &folisift_core::StyleProfile) -> Result<()> {
    let fail = |axis: &str, value: &str, supported: &[String]| {
        Err(Error::Style {
            id: pack.id.clone(),
            message: format!("{axis} {value:?} is not supported; expected one of {supported:?}"),
        })
    };
    if !pack.supported_structures.contains(&profile.structure) {
        return fail("structure", &profile.structure, &pack.supported_structures);
    }
    if !pack.supported_languages.contains(&profile.language) {
        return fail("language", &profile.language, &pack.supported_languages);
    }
    if !pack.supported_domains.contains(&profile.domain) {
        return fail("domain", &profile.domain, &pack.supported_domains);
    }
    if !pack.supported_voices.contains(&profile.voice) {
        return fail("voice", &profile.voice, &pack.supported_voices);
    }
    if !profile.palette.is_empty()
        && !pack.supported_palettes.is_empty()
        && !pack.supported_palettes.contains(&profile.palette)
    {
        return fail("palette", &profile.palette, &pack.supported_palettes);
    }
    if !profile.fonts.is_empty()
        && !pack.supported_fonts.is_empty()
        && !pack.supported_fonts.contains(&profile.fonts)
    {
        return fail("fonts", &profile.fonts, &pack.supported_fonts);
    }
    Ok(())
}

fn default_table() -> String {
    "Tablo".to_string()
}

fn default_recall() -> String {
    "HATIRLA".to_string()
}

fn default_explain() -> String {
    "AÇIKLA".to_string()
}

fn default_review_plan() -> String {
    "TEKRAR PLANI".to_string()
}

fn default_synthesis() -> String {
    "TEK SAYFA SENTEZ".to_string()
}

fn default_answer() -> String {
    "CEVAP".to_string()
}

fn default_revision_badge() -> String {
    "SINAV TEKRARI".to_string()
}

fn default_organiser_badge() -> String {
    "BİLGİ ORGANİZATÖRÜ".to_string()
}

fn default_summary_label() -> String {
    "ÖZET".to_string()
}

fn default_workspace_label() -> String {
    "ÇALIŞMA ALANI".to_string()
}

fn default_answer_key_label() -> String {
    "CEVAP ANAHTARI".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compatibility_filters_irac_law_to_reader_and_accessible() {
        let compatible = compatibility(Path::new(""), "irac", "law").expect("compatibility");
        let styles = compatible
            .iter()
            .filter(|item| item.compatible)
            .map(|item| item.style.as_str())
            .collect::<Vec<_>>();
        assert_eq!(styles, vec!["accessible", "clean", "reader", "workbook"]);
    }

    #[test]
    fn rejected_pairs_list_compatible_styles() {
        let loaded = load(Path::new(""), "paper").expect("paper");
        let profile = folisift_core::StyleProfile {
            aesthetic: "paper".to_string(),
            structure: "workbook".to_string(),
            domain: "law".to_string(),
            language: "tr".to_string(),
            source_language: "tr".to_string(),
            voice: "academic".to_string(),
            ..folisift_core::StyleProfile::default()
        };
        let error = validate_profile_actionable(Path::new(""), &loaded.pack, &profile)
            .expect_err("unsupported pair")
            .to_string();
        assert!(error.contains("compatible_styles"), "{error}");
        assert!(error.contains("workbook"), "{error}");
        assert!(error.contains("next:"), "{error}");
    }

    #[test]
    fn matrix_lists_every_pack_and_its_declared_pairs() {
        let rows = matrix(Path::new(""), Some("workbook"), Some("law")).expect("matrix");
        assert_eq!(rows.len(), builtin_ids().len());
        let workbook = rows
            .iter()
            .find(|row| row.style == "workbook")
            .expect("workbook");
        assert_eq!(workbook.compatible, Some(true));
        assert!(
            workbook
                .supported_structures
                .iter()
                .any(|value| value == "irac")
        );
        let clean = rows.iter().find(|row| row.style == "clean").expect("clean");
        assert!(clean.supported_domains.iter().any(|value| value == "law"));
        for row in &rows {
            if row.compatible != Some(true) {
                continue;
            }
            let loaded = load(Path::new(""), &row.style).expect("load");
            for structure in &row.supported_structures {
                for domain in &row.supported_domains {
                    let profile = folisift_core::StyleProfile {
                        aesthetic: loaded.pack.aesthetic.clone(),
                        structure: structure.clone(),
                        domain: domain.clone(),
                        language: loaded.pack.supported_languages[0].clone(),
                        source_language: loaded.pack.supported_languages[0].clone(),
                        voice: loaded.pack.supported_voices[0].clone(),
                        ..folisift_core::StyleProfile::default()
                    };
                    validate_profile(&loaded.pack, &profile).unwrap_or_else(|error| {
                        panic!("{} {structure}+{domain}: {error}", row.style)
                    });
                }
            }
        }
    }

    #[test]
    fn estimate_flags_an_over_budget_accessible_guide() {
        let markdown = format!("## 1.1 Giriş\n\n{}\n", "kelime ".repeat(4000));
        let estimate = estimate("accessible", &markdown, Some("2-4"));
        assert!(estimate.over_budget, "{estimate:?}");
        assert!(estimate.estimated_pages.starts_with('1'));
    }

    #[test]
    fn loads_all_palettes_and_font_pairings() {
        let palettes = list_palettes(Path::new("")).expect("palettes");
        assert_eq!(palettes.len(), 19);
        let fonts = list_fonts(Path::new("")).expect("fonts");
        assert_eq!(fonts.len(), 2);
        for palette in palettes {
            let loaded = load_palette(Path::new(""), &palette.id).expect("contrast");
            assert_eq!(loaded.id, palette.id);
        }
    }

    #[test]
    fn pack_defaults_resolve_to_the_legacy_colours() {
        let reader = load(Path::new(""), "reader").expect("reader");
        assert_eq!(reader.palette.id, "warm-paper");
        assert_eq!(reader.fonts.id, "editorial");
        let revision = load(Path::new(""), "revision").expect("revision");
        assert_eq!(revision.palette.id, "warm-paper");
        assert_eq!(revision.fonts.id, "technical");
        let clean = load(Path::new(""), "clean").expect("clean");
        assert_eq!(clean.palette.id, "cool-slate");
        assert_eq!(clean.fonts.id, "technical");
        let accessible = load(Path::new(""), "accessible").expect("accessible");
        assert_eq!(accessible.palette.id, "high-contrast");
    }

    #[test]
    fn palette_and_font_overlays_change_tokens() {
        let loaded = load_with(
            Path::new(""),
            "revision",
            Some("forest-sage"),
            Some("technical"),
        )
        .expect("overlay");
        assert_eq!(loaded.palette.id, "forest-sage");
        assert_eq!(loaded.fonts.id, "technical");
        assert_eq!(loaded.tokens["palette"]["accent"].as_str(), Some("#2F5D50"));
        let applied = apply_tokens(
            include_str!("../../../templates/revision.typ"),
            &loaded.tokens,
            "revision",
        )
        .expect("apply");
        assert!(applied.contains("rgb(\"#2F5D50\")"));

        let accessible = load_with(
            Path::new(""),
            "accessible",
            Some("mono-print"),
            Some("technical"),
        )
        .expect("accessible overlay");
        assert_eq!(accessible.palette.id, "mono-print");
        assert_eq!(accessible.fonts.body, "Inter");
        assert_eq!(
            accessible.tokens["palette"]["accent"].as_str(),
            Some("#000000")
        );
    }

    #[test]
    fn rejects_unknown_and_unlisted_designs() {
        assert!(load_palette(Path::new(""), "does-not-exist").is_err());
        assert!(load_fonts(Path::new(""), "does-not-exist").is_err());
        assert!(load_with(Path::new(""), "accessible", Some("berry"), None).is_err());
        assert!(load_palette(Path::new(""), "../paper").is_err());
    }

    #[test]
    fn rejects_path_like_pack_ids() {
        for id in ["../paper", "a/b", "..", "Paper", ""] {
            assert!(load(Path::new(""), id).is_err(), "{id}");
        }
    }

    #[test]
    fn applies_palette_tokens() {
        let tokens = serde_json::json!({"palette": {"paper": "#123456", "accent": "#abcdef"}});
        let template = "fill: #FAF6EC; stroke: #8A3E2F;";
        let applied = apply_tokens(template, &tokens, "paper").expect("palette");
        assert!(applied.contains("#123456"));
        assert!(applied.contains("#ABCDEF"));
        assert!(!applied.contains("#FAF6EC"));
    }

    #[test]
    fn computes_contrast_and_parses_hex() {
        let black = parse_hex("#000000").expect("black");
        let white = parse_hex("#FFFFFF").expect("white");
        assert!((contrast_ratio(black, white) - 21.0).abs() < 0.01);
        assert!(parse_hex("not-a-colour").is_none());
    }

    #[test]
    fn builtins_pass_accessibility() {
        let paper = load(Path::new(""), "paper").expect("paper");
        assert_eq!(paper.pack.status, "stable");
        let clean = load(Path::new(""), "clean").expect("clean");
        assert_eq!(clean.pack.status, "experimental");
        let reader = load(Path::new(""), "reader").expect("reader");
        assert_eq!(reader.pack.layout, "paper");
        assert_eq!(reader.tokens["typography"]["body_size"], 10.0);
        for id in ["journal", "brief", "accessible", "revision"] {
            assert!(load(Path::new(""), id).is_ok(), "{id}");
        }
    }

    #[test]
    fn revision_code_roles_use_the_scale() {
        let tokens: Value = serde_json::from_str(REVISION_TOKENS).expect("tokens");
        let applied = apply_tokens(
            include_str!("../../../templates/revision.typ"),
            &tokens,
            "revision",
        )
        .expect("revision scale");
        assert!(applied.contains("#let code-size = 9.7pt"), "{applied}");
        assert!(
            applied.contains("#let code-inline-size = 10.0pt"),
            "{applied}"
        );
        assert!(applied.contains("#let body-size = 10.5pt"));
    }

    #[test]
    fn paper_pins_legacy_sizes_and_reader_retunes_components() {
        let paper = load(Path::new(""), "paper").expect("paper");
        let paper_tpl = apply_tokens(
            include_str!("../../../templates/paper.typ"),
            &paper.tokens,
            "paper",
        )
        .expect("paper");
        for expected in ["size: 19.0pt", "size: 7.9pt", "size: 7.8pt", "size: 6.9pt"] {
            assert!(paper_tpl.contains(expected), "missing {expected}");
        }
        let reader = load(Path::new(""), "reader").expect("reader");
        let reader_tpl = apply_tokens(
            include_str!("../../../templates/paper.typ"),
            &reader.tokens,
            "paper",
        )
        .expect("reader");
        for expected in ["size: 20.0pt", "size: 8.6pt", "size: 8.0pt", "size: 9.5pt"] {
            assert!(reader_tpl.contains(expected), "missing {expected}");
        }
    }

    #[test]
    fn accessible_sizes_scale_without_a_multiplier() {
        let accessible = load(Path::new(""), "accessible").expect("accessible");
        assert!(accessible.tokens.get("component_scale").is_none());
        let applied = apply_tokens(
            include_str!("../../../templates/paper.typ"),
            &accessible.tokens,
            "paper",
        )
        .expect("accessible");
        for expected in [
            "size: 24.0pt",
            "size: 10.3pt",
            "size: 9.6pt",
            "size: 11.0pt",
        ] {
            assert!(applied.contains(expected), "missing {expected}");
        }
    }

    #[test]
    fn applies_reader_rhythm_tokens() {
        let tokens = serde_json::json!({
            "palette": {},
            "typography": {"body_size": 10.2, "leading_em": 0.72},
            "layout": {"margin_top_mm": 18, "margin_bottom_mm": 16, "margin_side_mm": 18},
            "spacing": {"par_mm": 2.4, "list_mm": 3.8}
        });
        let template = "size: __BODY_SIZE__; leading: 0.62em; margin: (top: 16mm, bottom: 14mm, left: 16.5mm, right: 16.5mm); spacing: 1.8mm; spacing: 3.2mm;";
        let applied = apply_tokens(template, &tokens, "paper").expect("rhythm");
        assert!(applied.contains("10.2pt"));
        assert!(applied.contains("leading: 0.72em"));
        assert!(applied.contains("margin: (top: 18mm, bottom: 16mm, left: 18mm, right: 18mm)"));
        assert!(applied.contains("spacing: 2.4mm"));
        assert!(applied.contains("spacing: 3.8mm"));
    }
}
