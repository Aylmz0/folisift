//! Deterministic topic clustering from local source headings.
//!
//! Model2Vec is optional: when no embedder is supplied the report is a
//! heading-term fallback and says so explicitly.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::link::{Embedder, Error, Result};
use crate::model::Model2VecEmbedder;
use folisift_parse::Inventory;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TopicHeading {
    pub source: String,
    pub page: usize,
    pub heading: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TopicCluster {
    pub term: String,
    pub headings: Vec<TopicHeading>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TopicReport {
    pub method: String,
    pub fallback: Option<String>,
    pub warning: Option<String>,
    pub model_signature: Option<String>,
    pub inventory_digest: String,
    pub topics: Vec<TopicCluster>,
}

pub fn topic_report(
    inventory: &Inventory,
    k: usize,
    embedder: Option<&Model2VecEmbedder>,
) -> Result<TopicReport> {
    let digest = inventory_digest(inventory);
    let rows = topic_rows(inventory);
    if rows.is_empty() {
        return Ok(TopicReport {
            method: "empty".to_string(),
            fallback: None,
            warning: None,
            model_signature: None,
            inventory_digest: digest,
            topics: Vec::new(),
        });
    }
    let Some(embedder) = embedder else {
        return Ok(TopicReport {
            method: "heading-terms".to_string(),
            fallback: None,
            warning: None,
            model_signature: None,
            inventory_digest: digest,
            topics: heading_terms(&rows, k),
        });
    };
    let texts = rows
        .iter()
        .map(|row| row.heading.clone())
        .collect::<Vec<_>>();
    let vectors = embedder.embed(&texts)?;
    let mut clusters: Vec<(Vec<usize>, Vec<f32>)> = Vec::new();
    for (index, vector) in vectors.iter().enumerate() {
        let mut best: Option<(usize, f32)> = None;
        for (cluster_index, (_, centroid)) in clusters.iter().enumerate() {
            let score = cosine(vector, centroid);
            if best.is_none_or(|(_, value)| score > value) {
                best = Some((cluster_index, score));
            }
        }
        match best {
            Some((cluster_index, score)) if score >= 0.78 => {
                clusters[cluster_index].0.push(index);
                clusters[cluster_index].1 = centroid(&vectors, &clusters[cluster_index].0);
            }
            _ if clusters.len() < k.max(1) => clusters.push((vec![index], vector.clone())),
            Some((cluster_index, _)) => {
                clusters[cluster_index].0.push(index);
                clusters[cluster_index].1 = centroid(&vectors, &clusters[cluster_index].0);
            }
            None => clusters.push((vec![index], vector.clone())),
        }
    }
    let topics = clusters
        .into_iter()
        .map(|(members, _)| {
            let first = members.first().map(|index| &rows[*index]);
            TopicCluster {
                term: first.map_or_else(String::new, |row| row.heading.clone()),
                headings: members
                    .iter()
                    .take(8)
                    .map(|index| rows[*index].clone())
                    .collect(),
            }
        })
        .collect();
    Ok(TopicReport {
        method: "model2vec".to_string(),
        fallback: None,
        warning: None,
        model_signature: Some(embedder.signature()?),
        inventory_digest: digest,
        topics,
    })
}

pub fn heading_term_report(
    inventory: &Inventory,
    k: usize,
    warning: Option<String>,
) -> TopicReport {
    let rows = topic_rows(inventory);
    TopicReport {
        method: "heading-terms".to_string(),
        fallback: Some("model2vec-unavailable".to_string()),
        warning,
        model_signature: None,
        inventory_digest: inventory_digest(inventory),
        topics: if rows.is_empty() {
            Vec::new()
        } else {
            heading_terms(&rows, k)
        },
    }
}

fn topic_rows(inventory: &Inventory) -> Vec<TopicHeading> {
    let mut rows = Vec::new();
    for source in &inventory.sources {
        let id = source.filename.trim_end_matches(".pdf").to_string();
        for heading in &source.headings {
            rows.push(TopicHeading {
                source: id.clone(),
                page: heading.page,
                heading: heading.heading.clone(),
            });
        }
    }
    rows.truncate(300);
    rows
}

fn heading_terms(rows: &[TopicHeading], k: usize) -> Vec<TopicCluster> {
    let stop = [
        "the", "and", "for", "with", "from", "this", "that", "bir", "ile", "için", "olarak",
        "daha", "gibi", "olan", "ve", "bu",
    ];
    let mut terms: std::collections::BTreeMap<String, Vec<TopicHeading>> =
        std::collections::BTreeMap::new();
    for row in rows {
        for word in row.heading.to_lowercase().split_whitespace() {
            let word = word.trim_matches(|character: char| !character.is_alphanumeric());
            if word.chars().count() < 5 || stop.contains(&word) {
                continue;
            }
            terms.entry(word.to_string()).or_default().push(row.clone());
        }
    }
    let mut ranked = terms.into_iter().collect::<Vec<_>>();
    ranked.sort_by_key(|(term, rows)| (std::cmp::Reverse(rows.len()), term.clone()));
    ranked.truncate(k);
    ranked
        .into_iter()
        .map(|(term, headings)| TopicCluster {
            term,
            headings: headings.into_iter().take(8).collect(),
        })
        .collect()
}

fn cosine(left: &[f32], right: &[f32]) -> f32 {
    let dot = left.iter().zip(right).map(|(a, b)| a * b).sum::<f32>();
    let ln = left.iter().map(|value| value * value).sum::<f32>().sqrt();
    let rn = right.iter().map(|value| value * value).sum::<f32>().sqrt();
    if ln == 0.0 || rn == 0.0 {
        0.0
    } else {
        dot / (ln * rn)
    }
}

fn centroid(vectors: &[Vec<f32>], members: &[usize]) -> Vec<f32> {
    let len = vectors[members[0]].len();
    let mut out = vec![0.0; len];
    for index in members {
        for (slot, value) in out.iter_mut().zip(&vectors[*index]) {
            *slot += value;
        }
    }
    let count = members.len().max(1) as f32;
    for value in &mut out {
        *value /= count;
    }
    out
}

pub fn inventory_digest(inventory: &Inventory) -> String {
    let bytes = serde_json::to_vec(inventory).unwrap_or_default();
    format!("{:x}", Sha256::digest(bytes))
}

pub fn cache_matches(
    value: &serde_json::Value,
    inventory_digest: &str,
    model_signature: Option<&str>,
    k: usize,
) -> bool {
    value.get("inventory_digest").and_then(|item| item.as_str()) == Some(inventory_digest)
        && value.get("model_signature").and_then(|item| item.as_str()) == model_signature
        && value
            .get("k")
            .and_then(|item| item.as_u64())
            .is_some_and(|value| value == k as u64)
}

pub fn model_signature(path: &std::path::Path) -> Result<String> {
    let manifest = std::fs::read(path.join("manifest.json"))
        .map_err(|error| Error::Model(format!("model manifest: {error}")))?;
    let parsed: serde_json::Value = serde_json::from_slice(&manifest)
        .map_err(|error| Error::Model(format!("model manifest JSON: {error}")))?;
    let mut hasher = Sha256::new();
    hasher.update(&manifest);
    if let Some(files) = parsed.get("files").and_then(serde_json::Value::as_object) {
        let mut names = files.keys().collect::<Vec<_>>();
        names.sort();
        for name in names {
            hasher.update(name.as_bytes());
            if let Some(value) = files[name]
                .get("sha256")
                .and_then(serde_json::Value::as_str)
            {
                hasher.update(value.as_bytes());
            }
        }
    }
    Ok(format!("{:x}", hasher.finalize()))
}
