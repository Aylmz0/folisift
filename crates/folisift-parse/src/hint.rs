//! Advisory coverage hints: map source headings to guide headings.
//!
//! Hints are review aids, never proof. A `likely-missing` row means the host
//! should inspect the source section; only the reviewed manifest can mark it
//! covered.

use serde::Serialize;

use crate::inventory::Inventory;

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
    let guide_headings = guide_headings(markdown);
    let mut hints = Vec::new();
    for source in &inventory.sources {
        let source_id = source.filename.trim_end_matches(".pdf").to_string();
        for heading in &source.headings {
            let number = heading
                .heading
                .split_whitespace()
                .next()
                .unwrap_or_default();
            if number.matches('.').count() != 1 {
                continue;
            }
            let mut best: Option<(&str, f64)> = None;
            for guide in &guide_headings {
                let score = similarity(&heading.heading, guide);
                if best.is_none_or(|(_, current)| score > current) {
                    best = Some((guide, score));
                }
            }
            let (guide_heading, score) = best.unwrap_or(("", 0.0));
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
                note: "advisory classification; confirm with the reviewed manifest".to_string(),
            });
        }
    }
    hints
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
    fn hints_are_deterministic() {
        let first = coverage_hints(&inventory(), "## 3.1 Giriş\n");
        let second = coverage_hints(&inventory(), "## 3.1 Giriş\n");
        assert_eq!(first, second);
    }
}
