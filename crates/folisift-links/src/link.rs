use std::collections::{BTreeMap, BTreeSet};

use bm25::{DefaultTokenizer, Document, Language, SearchEngineBuilder};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::section::Section;

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("model error: {0}")]
    Model(String),
    #[error("invalid link input: {0}")]
    Invalid(String),
    #[error("layout parse error: {0}")]
    Parse(String),
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait Embedder {
    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LinkOptions {
    pub top_k: usize,
    pub min_cosine: f32,
    pub min_score: f32,
    pub lexical_candidates: usize,
    pub max_section_chars: usize,
}

impl Default for LinkOptions {
    fn default() -> Self {
        Self {
            top_k: 5,
            min_cosine: 0.75,
            min_score: 0.55,
            lexical_candidates: 20,
            max_section_chars: 1200,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Link {
    pub from: String,
    pub to: String,
    pub relation: String,
    pub direction: String,
    pub confidence: String,
    pub cosine: f32,
    pub lexical: f32,
    pub score: f32,
    pub from_source: String,
    pub from_page: usize,
    pub to_source: String,
    pub to_page: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LinkReport {
    pub schema_version: u32,
    pub model: Option<String>,
    pub options: LinkOptions,
    pub sections: Vec<Section>,
    pub links: Vec<Link>,
}

pub struct LinkIndex;

impl LinkIndex {
    pub fn build(
        sections: &[Section],
        embedder: Option<&dyn Embedder>,
        model_name: Option<String>,
        options: &LinkOptions,
    ) -> Result<LinkReport> {
        if sections.len() < 2 {
            return Ok(LinkReport {
                schema_version: crate::store::LINKS_SCHEMA_VERSION,
                model: model_name,
                options: options.clone(),
                sections: sections.to_vec(),
                links: Vec::new(),
            });
        }

        let tokenizer = DefaultTokenizer::new(Language::Turkish);
        let documents = sections
            .iter()
            .enumerate()
            .map(|(index, section)| Document::new(index as u32, section.text.clone()));
        let engine =
            SearchEngineBuilder::<u32>::with_tokenizer_and_documents(tokenizer, documents).build();

        let mut candidates: BTreeMap<(usize, usize), f32> = BTreeMap::new();
        for (index, section) in sections.iter().enumerate() {
            let results = engine.search(&section.text, Some(options.lexical_candidates + 1));
            let mut scores = Vec::new();
            for result in &results {
                let other = result.document.id as usize;
                if other != index {
                    scores.push((other, result.score));
                }
            }
            let min = scores
                .iter()
                .map(|(_, score)| *score)
                .fold(f32::INFINITY, f32::min);
            let max = scores
                .iter()
                .map(|(_, score)| *score)
                .fold(f32::NEG_INFINITY, f32::max);
            for (other, score) in scores {
                let normalized = if max > min {
                    (score - min) / (max - min)
                } else {
                    1.0
                };
                let key = if index < other {
                    (index, other)
                } else {
                    (other, index)
                };
                candidates
                    .entry(key)
                    .and_modify(|value| *value = value.max(normalized))
                    .or_insert(normalized);
            }
        }

        let embeddings = match embedder {
            Some(embedder) => {
                let texts = sections
                    .iter()
                    .map(|section| section.text.clone())
                    .collect::<Vec<_>>();
                let embeddings = embedder.embed(&texts)?;
                if embeddings.len() != sections.len() {
                    return Err(Error::Invalid(
                        "embedding count does not match section count".to_string(),
                    ));
                }
                Some(embeddings)
            }
            None => None,
        };

        let mut best: BTreeMap<(usize, usize), Link> = BTreeMap::new();
        let mut degrees = vec![0usize; sections.len()];
        let mut accepted_pairs: Vec<(usize, usize)> = Vec::new();
        let mut incident: Vec<Vec<usize>> = vec![Vec::new(); sections.len()];
        let mut page_pairs: BTreeSet<((String, usize), (String, usize))> = BTreeSet::new();
        let mut ranked = Vec::new();
        for (&(left, right), lexical) in &candidates {
            if sections[left].source == sections[right].source
                && (same_heading(&sections[left], &sections[right])
                    || sections[left]
                        .page_start
                        .abs_diff(sections[right].page_start)
                        <= 2
                    || (sections[left]
                        .page_start
                        .abs_diff(sections[right].page_start)
                        <= 10
                        && same_heading_tokens(&sections[left], &sections[right])))
            {
                // Repeated slide titles and immediate neighbours are serial
                // navigation, not a cross-topic connection.
                continue;
            }
            if sections[left].source != sections[right].source
                && same_heading(&sections[left], &sections[right])
                && token_overlap(&sections[left].text, &sections[right].text) >= 0.80
            {
                // A shared bibliography or boilerplate block is duplicated
                // material, not a new connection.
                continue;
            }
            let cosine = match &embeddings {
                Some(embeddings) => cosine(&embeddings[left], &embeddings[right]),
                None => 0.0,
            };
            if embeddings.is_some() && cosine < options.min_cosine {
                continue;
            }
            let score = if embeddings.is_some() {
                0.4 * lexical + 0.6 * cosine
            } else {
                *lexical
            };
            if score < options.min_score {
                continue;
            }
            ranked.push((score, *lexical, cosine, left, right));
        }
        ranked.sort_by(|left, right| {
            right
                .0
                .partial_cmp(&left.0)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left.3.cmp(&right.3))
                .then_with(|| left.4.cmp(&right.4))
        });

        for (score, lexical, cosine, left, right) in ranked {
            if degrees[left] >= options.top_k || degrees[right] >= options.top_k {
                continue;
            }
            let duplicate = incident[left].iter().any(|&index| {
                let (first, second) = accepted_pairs[index];
                let other = if first == left { second } else { first };
                near_duplicate_target(&sections[right], &sections[other])
            }) || incident[right].iter().any(|&index| {
                let (first, second) = accepted_pairs[index];
                let other = if first == right { second } else { first };
                near_duplicate_target(&sections[left], &sections[other])
            });
            if duplicate {
                // One section should not collect several near-identical
                // targets; keep the highest-scoring edge only.
                continue;
            }
            let from_page = (sections[left].source.clone(), sections[left].page_start);
            let to_page = (sections[right].source.clone(), sections[right].page_start);
            let page_pair = if from_page <= to_page {
                (from_page, to_page)
            } else {
                (to_page, from_page)
            };
            if page_pairs.contains(&page_pair) {
                // Several sections can share a page; one page-to-page edge
                // is enough for a review proposal.
                continue;
            }
            let (from, to) = (left, right);
            let key = (from, to);
            let entry = Link {
                from: sections[from].id.clone(),
                to: sections[to].id.clone(),
                relation: "semantically_similar_to".to_string(),
                direction: "bidirectional".to_string(),
                confidence: "SIMILARITY".to_string(),
                cosine,
                lexical,
                score,
                from_source: sections[from].source.clone(),
                from_page: sections[from].page_start,
                to_source: sections[to].source.clone(),
                to_page: sections[to].page_start,
            };
            if best.insert(key, entry).is_none() {
                degrees[from] += 1;
                degrees[to] += 1;
                incident[from].push(accepted_pairs.len());
                incident[to].push(accepted_pairs.len());
                accepted_pairs.push(key);
                page_pairs.insert(page_pair);
            }
        }

        let mut links = best.into_values().collect::<Vec<_>>();
        links.sort_by(|left, right| {
            right
                .score
                .partial_cmp(&left.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left.from.cmp(&right.from))
                .then_with(|| left.to.cmp(&right.to))
        });
        Ok(LinkReport {
            schema_version: crate::store::LINKS_SCHEMA_VERSION,
            model: model_name,
            options: options.clone(),
            sections: sections.to_vec(),
            links,
        })
    }
}

fn same_heading(left: &Section, right: &Section) -> bool {
    let normalize = |value: Option<&str>| {
        value
            .unwrap_or_default()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase()
    };
    let left = normalize(left.heading.as_deref());
    let right = normalize(right.heading.as_deref());
    !left.is_empty() && left == right
}

/// Ignore slide numbers and punctuation so "Örnek 2: X (9)" and "Örnek: X
/// (10)" compare as the same series title.
fn same_heading_tokens(left: &Section, right: &Section) -> bool {
    let heading_tokens = |section: &Section| {
        tokens(section.heading.as_deref().unwrap_or_default())
            .into_iter()
            .filter(|token| !token.chars().all(|character| character.is_ascii_digit()))
            .collect::<std::collections::BTreeSet<_>>()
    };
    let left = heading_tokens(left);
    let right = heading_tokens(right);
    !left.is_empty() && left == right
}

/// Two adjacent sections that repeat a heading or nearly all of their text are
/// the same slide variant, not two independent targets.
fn near_duplicate_target(left: &Section, right: &Section) -> bool {
    if left.source != right.source || left.page_start.abs_diff(right.page_start) > 2 {
        return false;
    }
    same_heading(left, right) || token_overlap(&left.text, &right.text) >= 0.85
}

fn token_overlap(left: &str, right: &str) -> f32 {
    let left = tokens(left);
    let right = tokens(right);
    if left.is_empty() || right.is_empty() {
        return 0.0;
    }
    let intersection = left.intersection(&right).count() as f32;
    let union = left.union(&right).count() as f32;
    intersection / union
}

fn tokens(text: &str) -> std::collections::BTreeSet<String> {
    text.split(|character: char| !character.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(str::to_lowercase)
        .collect()
}

fn cosine(left: &[f32], right: &[f32]) -> f32 {
    if left.len() != right.len() || left.is_empty() {
        return 0.0;
    }
    let mut dot = 0.0;
    let mut left_norm = 0.0;
    let mut right_norm = 0.0;
    for (a, b) in left.iter().zip(right) {
        dot += a * b;
        left_norm += a * a;
        right_norm += b * b;
    }
    if left_norm == 0.0 || right_norm == 0.0 {
        0.0
    } else {
        dot / (left_norm.sqrt() * right_norm.sqrt())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::section::Section;

    struct FakeEmbedder;

    impl Embedder for FakeEmbedder {
        fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
            Ok(texts
                .iter()
                .map(|text| {
                    let mut vector = vec![0.0; 4];
                    if text.contains("atom") {
                        vector[0] = 1.0;
                    }
                    if text.contains("list") {
                        vector[1] = 1.0;
                    }
                    if text.contains("compiler") {
                        vector[2] = 1.0;
                    }
                    if text.contains("table") {
                        vector[3] = 1.0;
                    }
                    vector
                })
                .collect())
        }
    }

    fn section(id: &str, text: &str, page: usize) -> Section {
        Section {
            id: id.to_string(),
            source: "PD0".to_string(),
            page_start: page,
            page_end: page,
            block_ids: vec![],
            heading: None,
            text: text.to_string(),
            chars: text.chars().count(),
        }
    }

    fn headed(source: &str, id: &str, text: &str, page: usize, heading: &str) -> Section {
        Section {
            id: id.to_string(),
            source: source.to_string(),
            page_start: page,
            page_end: page,
            block_ids: vec![],
            heading: Some(heading.to_string()),
            text: text.to_string(),
            chars: text.chars().count(),
        }
    }

    #[test]
    fn links_similar_sections() {
        let sections = vec![
            section("a", "atom and list", 1),
            section("b", "atom and list representation", 5),
            section("c", "compiler table", 9),
        ];
        let report = LinkIndex::build(
            &sections,
            Some(&FakeEmbedder),
            Some("fake".to_string()),
            &LinkOptions::default(),
        )
        .expect("links");
        assert!(
            report
                .links
                .iter()
                .any(|link| link.from == "a" && link.to == "b")
        );
        assert!(report.links.iter().all(|link| link.from != link.to));
    }

    #[test]
    fn duplicate_targets_keep_one_edge() {
        let mut sections = vec![
            section("a", "compiler pipeline stage", 1),
            section("b", "compiler pipeline stage example one", 10),
            section("c", "compiler pipeline stage example two", 11),
        ];
        sections[1].heading = Some("Ara Kod".to_string());
        sections[2].heading = Some("Ara Kod".to_string());
        let report = LinkIndex::build(
            &sections,
            Some(&FakeEmbedder),
            Some("fake".to_string()),
            &LinkOptions::default(),
        )
        .expect("links");
        let to_b = report
            .links
            .iter()
            .any(|link| link.from == "a" && link.to == "b");
        let to_c = report
            .links
            .iter()
            .any(|link| link.from == "a" && link.to == "c");
        assert!(to_b ^ to_c, "expected exactly one duplicate target edge");
    }

    #[test]
    fn repeated_shared_material_is_not_a_link() {
        let text = "Roberto Sebesta Concepts of Programming Languages David Watt \
                    Programming Language Design Concepts Michael Scott Pragmatics";
        let sections = vec![
            headed("PD0", "a", text, 123, "Kaynaklar"),
            headed("PD1", "b", text, 137, "Kaynaklar"),
        ];
        let report = LinkIndex::build(
            &sections,
            Some(&FakeEmbedder),
            Some("fake".to_string()),
            &LinkOptions::default(),
        )
        .expect("links");
        assert!(
            report.links.is_empty(),
            "duplicated bibliography should not produce a link"
        );
    }

    #[test]
    fn repeated_slide_series_is_not_a_link() {
        let sections = vec![
            headed(
                "PD0",
                "a",
                "shift reduce parse example int plus int",
                77,
                "Örnek 2: Shift-Reduce Ayrıştırma (1)",
            ),
            headed(
                "PD0",
                "b",
                "shift reduce parse example int plus int",
                81,
                "Örnek: Shift-Reduce Ayrıştırma (10)",
            ),
        ];
        let report = LinkIndex::build(
            &sections,
            Some(&FakeEmbedder),
            Some("fake".to_string()),
            &LinkOptions::default(),
        )
        .expect("links");
        assert!(
            report.links.is_empty(),
            "a repeated slide series should not produce a link"
        );
    }

    #[test]
    fn one_page_pair_keeps_one_edge() {
        let sections = vec![
            section("a", "compiler pipeline stage", 1),
            headed(
                "PD0",
                "b",
                "compiler pipeline stage example one",
                10,
                "Örnek A",
            ),
            headed(
                "PD0",
                "c",
                "compiler pipeline stage example two",
                10,
                "Örnek B",
            ),
        ];
        let report = LinkIndex::build(
            &sections,
            Some(&FakeEmbedder),
            Some("fake".to_string()),
            &LinkOptions::default(),
        )
        .expect("links");
        let incident = report
            .links
            .iter()
            .filter(|link| link.from == "a" || link.to == "a")
            .count();
        assert_eq!(incident, 1, "expected one edge for the page pair");
    }

    #[test]
    fn bm25_only_mode_has_no_cosine_requirement() {
        let sections = vec![
            section("a", "atom and list", 1),
            section("b", "atom and list representation", 5),
            section("c", "compiler table", 9),
        ];
        let options = LinkOptions {
            min_score: 0.0,
            min_cosine: 1.0,
            ..LinkOptions::default()
        };
        let report = LinkIndex::build(&sections, None, None, &options).expect("links");
        assert!(!report.links.is_empty());
        assert!(report.links.iter().all(|link| link.cosine == 0.0));
    }
}
