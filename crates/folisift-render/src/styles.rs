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

const CLEAN_STYLE: &str = include_str!("../../../styles/clean/style.json");
const CLEAN_TOKENS: &str = include_str!("../../../styles/clean/tokens.json");
const CLEAN_TR: &str = include_str!("../../../styles/clean/strings/tr.json");
const CLEAN_EN: &str = include_str!("../../../styles/clean/strings/en.json");

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
    #[serde(default)]
    pub page: String,
}

#[derive(Debug, Clone)]
pub struct LoadedStyle {
    pub pack: StylePack,
    pub tokens: Value,
    pub strings: StyleStrings,
}

/// Built-in pack ids shipped in the binary.
pub fn builtin_ids() -> &'static [&'static str] {
    &["paper", "clean"]
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
    if pack.schema_version != 1 {
        return Err(Error::Style {
            id: id.to_string(),
            message: format!("unsupported style schema {}", pack.schema_version),
        });
    }
    let tokens_text = read_pack_file(styles_root, id, &pack.tokens)?;
    let tokens: Value = serde_json::from_str(&tokens_text).map_err(|error| Error::Style {
        id: id.to_string(),
        message: format!("tokens: {error}"),
    })?;
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
    })
}

/// Load localized labels for a pack.
pub fn strings(styles_root: &Path, id: &str, language: &str) -> Result<StyleStrings> {
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

fn io(path: &Path, source: std::io::Error) -> Error {
    Error::Io {
        path: path.to_path_buf(),
        source,
    }
}

fn read_pack_file(styles_root: &Path, id: &str, file: &str) -> Result<String> {
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
        ("clean", "style.json") => Some(CLEAN_STYLE),
        ("clean", "tokens.json") => Some(CLEAN_TOKENS),
        ("clean", "strings/tr.json") => Some(CLEAN_TR),
        ("clean", "strings/en.json") => Some(CLEAN_EN),
        ("clean", "preview.md") => Some(include_str!("../../../styles/clean/preview.md")),
        ("clean", "preview.en.md") => Some(include_str!("../../../styles/clean/preview.en.md")),
        _ => None,
    }
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
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
    }
}
