//! Built-in structure packs.
//!
//! A structure pack is a small list of required section headings. The engine
//! validates that a guide using the structure contains each required heading;
//! it does not rewrite or reorder the author's content.

use std::collections::HashMap;

use serde::Deserialize;

const IMRAD: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../structures/imrad.json"
));
const IRAC: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../structures/irac.json"
));
const SOAP: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../structures/soap.json"
));
const FORMULA: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../structures/formula-sheet.json"
));
const LITERATURE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../structures/literature-matrix.json"
));
const LECTURE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../structures/lecture-notes.json"
));
const BRIEF: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../structures/executive-brief.json"
));
const REVISION: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../structures/revision-sheet.json"
));
const ORGANISER: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../structures/knowledge-organiser.json"
));
const WORKBOOK: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../structures/workbook.json"
));

#[derive(Debug, Clone, Deserialize)]
pub struct StructurePack {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub sections: Vec<Section>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section {
    pub id: String,
    pub tr: String,
    pub en: String,
    #[serde(default)]
    pub hint: Option<String>,
    /// Optional sections are scaffolded but not required by the structure gate.
    #[serde(default = "default_required")]
    pub required: bool,
}

fn default_required() -> bool {
    true
}

pub fn packs() -> Vec<StructurePack> {
    [
        IMRAD, IRAC, SOAP, FORMULA, LITERATURE, LECTURE, BRIEF, REVISION, ORGANISER, WORKBOOK,
    ]
    .iter()
    .map(|text| serde_json::from_str(text).expect("built-in structure pack must be valid JSON"))
    .collect()
}

pub fn get(id: &str) -> Option<StructurePack> {
    packs().into_iter().find(|pack| pack.id == id)
}

/// Check that every required heading exists in the Markdown.
///
/// `a-g` has no structure pack and therefore no required headings; the audit
/// section is a separate, always-on gate.
pub fn validate(id: &str, language: &str, markdown: &str) -> Result<(), String> {
    if id == "a-g" {
        return Ok(());
    }
    let Some(pack) = get(id) else {
        return Err(format!(
            "unknown structure {id:?}; use a-g or a built-in pack"
        ));
    };
    let headings = headings(markdown);
    let missing = pack
        .sections
        .iter()
        .filter(|section| {
            if !section.required {
                return false;
            }
            let label = if language == "en" {
                &section.en
            } else {
                &section.tr
            };
            let label = normalize(label);
            !headings.iter().any(|heading| heading.contains(&label))
        })
        .map(|section| {
            if language == "en" {
                section.en.clone()
            } else {
                section.tr.clone()
            }
        })
        .collect::<Vec<_>>();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "structure {:?} is missing required sections: {}",
            pack.id,
            missing.join(", ")
        ))
    }
}

/// Required headings for a new guide skeleton.
pub fn scaffold_sections(id: &str, language: &str) -> Vec<(String, String)> {
    let Some(pack) = get(id) else {
        return Vec::new();
    };
    pack.sections
        .iter()
        .map(|section| {
            let title = if language == "en" {
                section.en.clone()
            } else {
                section.tr.clone()
            };
            let hint = section
                .hint
                .clone()
                .unwrap_or_else(|| "Kaynak içeriği buraya gelecek.".to_string());
            (title, hint)
        })
        .collect()
}

fn headings(markdown: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut fenced = false;
    for line in markdown.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            fenced = !fenced;
            continue;
        }
        if fenced || !trimmed.starts_with("## ") {
            continue;
        }
        let text = trimmed.trim_start_matches('#').trim();
        let text = text
            .split_once(' ')
            .filter(|(prefix, _)| {
                prefix.chars().any(|character| character.is_ascii_digit())
                    && prefix
                        .chars()
                        .all(|character| character.is_ascii_alphanumeric() || character == '.')
            })
            .map_or(text, |(_, rest)| rest);
        out.push(normalize(text));
    }
    out
}

/// Lowercase, strip Turkish dotted/dotless i differences and punctuation so
/// `1. Giriş`, `GİRİŞ` and `Giris` compare equal.
fn normalize(text: &str) -> String {
    text.chars()
        .filter(|character| character.is_alphanumeric())
        .flat_map(|character| match character {
            'İ' | 'I' | 'ı' => "i".chars().collect::<Vec<_>>(),
            '\u{0307}' => Vec::new(),
            other => other.to_lowercase().collect::<Vec<_>>(),
        })
        .collect()
}

/// Map structure ids to their required section counts, used by docs and tests.
pub fn summary() -> HashMap<String, usize> {
    packs()
        .into_iter()
        .map(|pack| {
            let required = pack
                .sections
                .iter()
                .filter(|section| section.required)
                .count();
            (pack.id, required)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_structures_fail_instead_of_disabling_the_gate() {
        assert!(validate("not-a-pack", "en", "## Anything").is_err());
        assert!(validate("a-g", "en", "").is_ok());
    }

    #[test]
    fn nested_headings_do_not_satisfy_a_structure() {
        let markdown = "### Introduction\n\n### Methods\n\n### Results\n\n### Discussion\n";
        assert!(validate("imrad", "en", markdown).is_err());
        let markdown = "## Introduction\n\n## Methods\n\n## Results\n\n## Discussion\n";
        assert!(validate("imrad", "en", markdown).is_ok());
    }

    #[test]
    fn every_pack_has_unique_section_ids() {
        for pack in packs() {
            let mut ids = pack
                .sections
                .iter()
                .map(|section| section.id.as_str())
                .collect::<Vec<_>>();
            ids.sort_unstable();
            ids.dedup();
            assert_eq!(ids.len(), pack.sections.len(), "{}", pack.id);
        }
    }

    #[test]
    fn imrad_requires_each_section() {
        let error = validate("imrad", "en", "## Introduction\n\nText.\n").expect_err("missing");
        assert!(error.contains("Methods"));
        let complete = "## Introduction\n\n## Methods\n\n## Results\n\n## Discussion\n";
        assert!(validate("imrad", "en", complete).is_ok());
    }

    #[test]
    fn turkish_titles_ignore_case_and_diacritics() {
        let complete = "## GİRİŞ\n\n## YÖNTEM\n\n## BULGULAR\n\n## TARTIŞMA\n";
        assert!(validate("imrad", "tr", complete).is_ok());
    }

    #[test]
    fn revision_sheet_allows_optional_worked_examples() {
        let complete = "## Kapsam ve harita\n\n## Bilmen gerekenler\n\n## Karşılaştırma\n\n## Sınav tuzakları\n\n## Hatırlama\n\n## Tek sayfa sentez\n\n## Tekrar planı\n";
        assert!(validate("revision-sheet", "tr", complete).is_ok());
        let missing = "## Kapsam ve harita\n\n## Bilmen gerekenler\n";
        let error = validate("revision-sheet", "tr", missing).expect_err("missing");
        assert!(error.contains("Karşılaştırma"), "{error}");
    }

    #[test]
    fn knowledge_organiser_allows_optional_timeline() {
        let complete = "## Büyük sorular\n\n## Anahtar sözcükler\n\n## Harita veya şema\n\n## Kritik ayrımlar\n";
        assert!(validate("knowledge-organiser", "tr", complete).is_ok());
    }

    #[test]
    fn a_g_has_no_extra_required_sections() {
        assert!(validate("a-g", "tr", "# Anything\n").is_ok());
    }
}
