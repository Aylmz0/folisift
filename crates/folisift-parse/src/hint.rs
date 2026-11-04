//! Advisory coverage hints: map source headings to guide headings.
//!
//! Hints are review aids, never proof. A `likely-missing` row means the host
//! should inspect the source section; only the reviewed manifest can mark it
//! covered.

use serde::Serialize;

use crate::inventory::{Heading, Inventory};
use crate::manifest::ManifestRow;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CoverageHint {
    pub source: String,
    pub page: usize,
    pub heading: String,
    pub status: String,
    pub guide_heading: Option<String>,
    pub confidence: String,
    pub score: f64,
    pub note: String,
}

pub fn coverage_hints(inventory: &Inventory, markdown: &str) -> Vec<CoverageHint> {
    coverage_hints_with_manifest(inventory, markdown, &[])
}

pub fn coverage_hints_with_manifest(
    inventory: &Inventory,
    markdown: &str,
    manifest_rows: &[ManifestRow],
) -> Vec<CoverageHint> {
    let guide_headings = guide_headings(markdown);
    let covers = covers_index(markdown);
    let mut hints = Vec::new();
    for source in &inventory.sources {
        let source_id = source.filename.trim_end_matches(".pdf").to_string();
        let manifest_headings = manifest_rows
            .iter()
            .filter(|row| row.source == source_id)
            .map(|row| Heading {
                heading: row.heading.clone(),
                page: row.page,
                level: None,
                block_id: None,
            })
            .collect::<Vec<_>>();
        let mut headings = if manifest_headings.is_empty() {
            source.headings.clone()
        } else {
            manifest_headings
        };
        headings.sort_by_key(|heading| heading.page);
        let mut seen = std::collections::HashSet::new();
        headings.retain(|heading| {
            let key = normalize(&heading.heading);
            seen.insert(key)
        });
        for heading in &headings {
            let number = heading
                .heading
                .split_whitespace()
                .next()
                .unwrap_or_default()
                .trim_end_matches('.');
            if number.matches('.').count() != 1 {
                continue;
            }
            let manifest_covered = covers.contains(&format!("{source_id}:{number}"))
                || manifest_rows.iter().any(|row| {
                    row.source == source_id
                        && row
                            .heading
                            .split_whitespace()
                            .next()
                            .unwrap_or_default()
                            .trim_end_matches('.')
                            == number
                        && !matches!(row.status, crate::manifest::CoverageStatus::Missing)
                });
            let mut best: Option<(&str, f64)> = None;
            for guide in &guide_headings {
                let score = similarity(&heading.heading, guide);
                if best.is_none_or(|(_, current)| score > current) {
                    best = Some((guide, score));
                }
            }
            let (guide_heading, score) = best.unwrap_or(("", 0.0));
            let score = if manifest_covered { 1.0 } else { score };
            let (status, confidence) = if score >= 0.82 {
                ("covered", "high")
            } else if score >= 0.55 {
                ("covered", "medium")
            } else if score >= 0.35 {
                ("likely-missing", "low")
            } else {
                ("likely-missing", "high")
            };
            hints.push(CoverageHint {
                source: source_id.clone(),
                page: heading.page,
                heading: heading.heading.clone(),
                status: status.to_string(),
                guide_heading: (!guide_heading.is_empty()).then(|| guide_heading.to_string()),
                confidence: confidence.to_string(),
                score: (score * 1000.0).round() / 1000.0,
                note: if manifest_covered {
                    "covered by the reviewed manifest; advisory only".to_string()
                } else {
                    "advisory classification; confirm with the reviewed manifest".to_string()
                },
            });
        }
    }
    hints
}

fn covers_index(markdown: &str) -> std::collections::BTreeSet<String> {
    static COVERS_RE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::Regex::new(r"<!--\s*covers:\s*([A-Za-z0-9_.-]+:[0-9.]+)\s*-->")
            .expect("valid covers regex")
    });
    COVERS_RE
        .captures_iter(markdown)
        .map(|capture| capture[1].to_string())
        .collect()
}

/// True when a source number is inside a merged guide heading range such as
/// `2.1–2.2` or `3.1-3.4`.
pub fn heading_range_contains(source_number: &str, guide_heading: &str) -> bool {
    let Some(source) = number_tuple(source_number) else {
        return false;
    };
    static RANGE_RE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::Regex::new(r"(\d+(?:\.\d+)*)\s*[–—\-]\s*(\d+(?:\.\d+)*)")
            .expect("valid heading range regex")
    });
    RANGE_RE.captures_iter(guide_heading).any(|capture| {
        let start = number_tuple(&capture[1]);
        let end = number_tuple(&capture[2]);
        matches!((start, end), (Some(start), Some(end)) if source >= start && source <= end)
    })
}

pub fn heading_has_range(value: &str) -> bool {
    static RANGE_RE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::Regex::new(r"\d+(?:\.\d+)*\s*[–—\-]\s*\d+(?:\.\d+)*")
            .expect("valid heading range regex")
    });
    RANGE_RE.is_match(value)
}

fn number_tuple(value: &str) -> Option<Vec<u32>> {
    let number = value.trim().trim_end_matches('.');
    let parts = number
        .split('.')
        .map(str::parse::<u32>)
        .collect::<std::result::Result<Vec<_>, _>>()
        .ok()?;
    (!parts.is_empty()).then_some(parts)
}

fn guide_headings(markdown: &str) -> Vec<String> {
    markdown
        .lines()
        .filter_map(|line| {
            line.strip_prefix("## ")
                .or_else(|| line.strip_prefix("### "))
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(ToString::to_string)
        })
        .collect()
}

fn similarity(source: &str, guide: &str) -> f64 {
    let source_number = source.split_whitespace().next().unwrap_or_default();
    if heading_range_contains(source_number, guide) {
        return 1.0;
    }
    let source = normalize(source);
    let guide = normalize(guide);
    if source == guide {
        return 1.0;
    }
    let source_number = source.split_whitespace().next().unwrap_or_default();
    let guide_number = guide.split_whitespace().next().unwrap_or_default();
    if !source_number.is_empty() && source_number == guide_number {
        return 0.95;
    }
    let source_terms = terms(&source);
    let guide_terms = terms(&guide);
    if source_terms.is_empty() || guide_terms.is_empty() {
        return 0.0;
    }
    let shared = source_terms
        .iter()
        .filter(|term| guide_terms.contains(*term))
        .count();
    let union = source_terms.len() + guide_terms.len() - shared;
    (shared as f64 / union.max(1) as f64).min(0.9)
}

fn normalize(value: &str) -> String {
    let mut out = String::new();
    for character in value.chars().flat_map(char::to_lowercase) {
        let mapped = match character {
            'i' | 'ı' => 'i',
            'ş' => 's',
            'ğ' => 'g',
            'ç' => 'c',
            'ö' => 'o',
            'ü' => 'u',
            '.' => '.',
            other if other.is_alphanumeric() => other,
            _ => ' ',
        };
        out.push(mapped);
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn terms(value: &str) -> Vec<&str> {
    value
        .split_whitespace()
        .filter(|term| term.chars().count() >= 3)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CanonicalSection, Heading, SourceInventory, Totals};

    fn inventory() -> Inventory {
        Inventory {
            generated_at: "2026-01-01T00:00:00Z".to_string(),
            low_text_threshold: 100,
            totals: Totals {
                pages: 10,
                words: 100,
                low_text_pages: 0,
            },
            sources: vec![SourceInventory {
                path: "sources/PD3.pdf".to_string(),
                filename: "PD3.pdf".to_string(),
                title: "PD3".to_string(),
                pages: 10,
                chars: 1000,
                words: 100,
                low_text_threshold: 100,
                source_digest: None,
                low_text_pages: Vec::new(),
                headings: vec![
                    Heading {
                        page: 3,
                        level: Some(2),
                        block_id: None,
                        heading: "3.1 Giriş".to_string(),
                    },
                    Heading {
                        page: 5,
                        level: Some(2),
                        block_id: None,
                        heading: "3.2 Soyut Sözdizimi".to_string(),
                    },
                ],
                layout: None,
                sections: Vec::<CanonicalSection>::new(),
                page_map: Vec::new(),
            }],
        }
    }

    #[test]
    fn hints_separate_covered_from_likely_missing() {
        let hints = coverage_hints(&inventory(), "## 3.1 Giriş\n\n## 3.3 Sonuç\n");
        let covered = hints
            .iter()
            .find(|hint| hint.heading == "3.1 Giriş")
            .expect("covered");
        assert_eq!(covered.status, "covered");
        assert_eq!(covered.confidence, "high");
        let missing = hints
            .iter()
            .find(|hint| hint.heading == "3.2 Soyut Sözdizimi")
            .expect("missing");
        assert_eq!(missing.status, "likely-missing");
        assert!(missing.note.contains("advisory"));
    }

    #[test]
    fn merged_heading_ranges_are_covered() {
        assert!(heading_range_contains("2.2", "2.1–2.2 Üretim"));
        assert!(heading_range_contains("3.3", "3.1-3.4 DPP"));
        assert!(!heading_range_contains("2.3", "2.1–2.2 Üretim"));
        let hints = coverage_hints(&inventory(), "## 3.1–3.2 Kaynaklar ve sözdizimi\n");
        let row = hints
            .iter()
            .find(|hint| hint.heading == "3.2 Soyut Sözdizimi")
            .expect("row");
        assert_eq!(row.status, "covered");
        assert_eq!(row.score, 1.0);
    }

    #[test]
    fn hints_are_deterministic() {
        let first = coverage_hints(&inventory(), "## 3.1 Giriş\n");
        let second = coverage_hints(&inventory(), "## 3.1 Giriş\n");
        assert_eq!(first, second);
    }
}
