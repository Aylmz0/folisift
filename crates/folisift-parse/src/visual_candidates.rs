//! Ranked visual candidates for a source PDF.
//!
//! The candidate report is text-only: it never carries image bytes. The host
//! uses the bounded context and the score breakdown to decide whether to
//! embed, redraw, describe or skip a visual.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::Path;

use liteparse::config::OutputFormat;
use liteparse::{LiteParse, LiteParseConfig};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::error::{Error, Result};
use crate::layout::{LayoutBlock, LayoutBlockKind, Rect};
use crate::pdfium;
use crate::text::LOW_TEXT_THRESHOLD;
use crate::visual_classify::{ClassifyInput, classify, page_has_chart};
use crate::visual_score::{ScoreFeatures, score};

pub const SCORER_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CandidateContext {
    pub before: String,
    pub after: String,
    pub inside: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Relevance {
    pub label: String,
    pub confidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Reconstruction {
    pub kind: String,
    pub confidence: String,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VisualCandidate {
    pub id: String,
    pub source: String,
    pub page: usize,
    pub kind: String,
    pub bbox: Rect,
    pub page_width: f32,
    pub page_height: f32,
    pub area_ratio: f32,
    pub native: bool,
    pub image_id: Option<String>,
    pub pixel_width: Option<u32>,
    pub pixel_height: Option<u32>,
    pub format: Option<String>,
    pub vector_primitives: usize,
    pub text_inside_chars: usize,
    pub heading: Option<String>,
    pub caption_seed: Option<String>,
    pub context: CandidateContext,
    pub group_id: Option<String>,
    pub group_members: usize,
    pub series: Option<String>,
    pub series_index: Option<usize>,
    pub content_class: String,
    pub relevance: Relevance,
    pub reconstruction: Reconstruction,
    pub score: u32,
    pub score_parts: BTreeMap<String, f32>,
    pub signals: Vec<String>,
    pub penalties: Vec<String>,
    pub suggested: String,
    pub confidence: String,
    pub recommended_width_percent: u8,
    pub effective_dpi: Option<f32>,
    pub review_mode: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CandidateCounts {
    pub total: usize,
    pub returned: usize,
    pub embed: usize,
    pub describe: usize,
    pub redraw: usize,
    pub skip: usize,
    pub core: usize,
    pub supporting: usize,
    pub decorative: usize,
    pub seductive: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CandidateReport {
    pub guide: Option<String>,
    pub source: String,
    pub source_digest: String,
    pub scorer_version: u32,
    pub counts: CandidateCounts,
    pub candidates: Vec<VisualCandidate>,
    pub next: Option<usize>,
    pub policy: String,
}

pub const CANDIDATE_MANIFEST_SCHEMA_VERSION: u32 = 1;

/// Deterministic, text-only cache of ranked candidates for a guide.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CandidateManifest {
    pub schema_version: u32,
    pub scorer_version: u32,
    pub generated_from: String,
    #[serde(default)]
    pub sources: Vec<CandidateSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CandidateSource {
    pub source: String,
    pub source_digest: String,
    pub pages: usize,
    #[serde(default)]
    pub candidates: Vec<VisualCandidate>,
}

impl CandidateManifest {
    pub fn new() -> Self {
        Self {
            schema_version: CANDIDATE_MANIFEST_SCHEMA_VERSION,
            scorer_version: SCORER_VERSION,
            generated_from: "visual_candidates".to_string(),
            sources: Vec::new(),
        }
    }

    pub fn read(path: &Path) -> Result<Self> {
        let bytes = std::fs::read(path).map_err(|error| crate::error::io(path, error))?;
        serde_json::from_slice(&bytes).map_err(|error| crate::error::Error::Parse {
            path: path.to_path_buf(),
            message: format!("visual candidate manifest: {error}"),
        })
    }

    pub fn write(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| crate::error::io(parent, error))?;
        }
        let text =
            serde_json::to_string_pretty(self).map_err(|error| crate::error::Error::Parse {
                path: path.to_path_buf(),
                message: format!("serialize visual candidate manifest: {error}"),
            })?;
        let temporary = path.with_extension("json.tmp");
        std::fs::write(&temporary, text + "\n")
            .map_err(|error| crate::error::io(&temporary, error))?;
        std::fs::rename(&temporary, path).map_err(|error| crate::error::io(path, error))
    }

    /// Upsert one source report. Other sources are preserved.
    pub fn upsert_source(&mut self, report: &CandidateReport) {
        let pages = report
            .candidates
            .iter()
            .map(|candidate| candidate.page)
            .max()
            .unwrap_or(0);
        let entry = CandidateSource {
            source: report.source.clone(),
            source_digest: report.source_digest.clone(),
            pages,
            candidates: report.candidates.clone(),
        };
        if let Some(existing) = self
            .sources
            .iter_mut()
            .find(|existing| existing.source == entry.source)
        {
            *existing = entry;
        } else {
            self.sources.push(entry);
            self.sources.sort_by(|a, b| a.source.cmp(&b.source));
        }
    }

    /// Page-level gate set. Core candidates and real table candidates count;
    /// decorative/seductive rows never create a mandatory page.
    pub fn gate_pages(&self, min_score: u32) -> BTreeSet<(String, usize)> {
        let mut pages = BTreeSet::new();
        for source in &self.sources {
            for candidate in &source.candidates {
                if candidate.score < min_score {
                    continue;
                }
                let decorative = matches!(
                    candidate.relevance.label.as_str(),
                    "decorative" | "seductive"
                );
                let core = candidate.relevance.label == "core";
                let table = candidate.reconstruction.kind == "table" && !decorative;
                if core || table {
                    pages.insert((source.source.clone(), candidate.page));
                }
            }
        }
        pages
    }

    pub fn source_digest(&self, source: &str) -> Option<&str> {
        self.sources
            .iter()
            .find(|entry| entry.source == source)
            .map(|entry| entry.source_digest.as_str())
    }
}

impl Default for CandidateManifest {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
struct Primitive {
    bbox: Rect,
}

#[derive(Debug, Clone)]
struct Seed {
    kind: String,
    bbox: Rect,
    image_id: Option<String>,
    pixel_width: Option<u32>,
    pixel_height: Option<u32>,
    format: Option<String>,
    vector_primitives: usize,
    table: bool,
    tagged: bool,
    image_count: usize,
}

#[derive(Debug, Clone)]
struct TextBlock {
    bbox: Rect,
    text: String,
    heading: bool,
}

pub async fn collect_candidates(
    source: &Path,
    source_id: &str,
    guide: Option<String>,
    pages: Option<&str>,
    min_score: u32,
    limit: usize,
) -> Result<CandidateReport> {
    pdfium::init(None)?;
    let source_text = source.to_str().ok_or_else(|| Error::Parse {
        path: source.to_path_buf(),
        message: "source path is not valid UTF-8".to_string(),
    })?;
    let result = LiteParse::new(LiteParseConfig {
        ocr_enabled: false,
        output_format: OutputFormat::Json,
        extract_blocks: true,
        extract_vector_graphics: true,
        extract_structure_tree: true,
        extract_content_bounds: true,
        include_complexity: true,
        target_pages: pages.map(ToString::to_string),
        quiet: true,
        max_pages: 10_000,
        ..Default::default()
    })
    .parse(source_text)
    .await
    .map_err(|error| Error::Parse {
        path: source.to_path_buf(),
        message: error.to_string(),
    })?;

    let digest = std::fs::read(source)
        .ok()
        .map(|bytes| blake3::hash(&bytes).to_hex().to_string())
        .unwrap_or_default();

    // Repeat detection is metadata-only: identical small image dimensions and
    // shape repeated on several pages are almost always logos/header strips.
    let mut repeat_keys = HashMap::<(String, u32, u32, u32, u32), usize>::new();
    for page in &result.pages {
        for image in &page.image_refs {
            let key = (
                image.format.clone(),
                image.pixel_width,
                image.pixel_height,
                image.bbox.width.round() as u32,
                image.bbox.height.round() as u32,
            );
            *repeat_keys.entry(key).or_default() += 1;
        }
    }

    let mut all = Vec::new();
    for page in &result.pages {
        let page_width = page.page_width;
        let page_height = page.page_height;
        let page_area = (page_width * page_height).max(1.0);
        let blocks = page
            .blocks
            .as_deref()
            .unwrap_or_default()
            .iter()
            .enumerate()
            .map(|(index, block)| {
                LayoutBlock::from_liteparse(
                    format!("{source_id}:p{}:b{index}", page.page_number),
                    block,
                )
            })
            .collect::<Vec<_>>();
        let text_blocks = page_text_blocks(page, &blocks);
        let headings = text_blocks
            .iter()
            .filter(|block| block.heading)
            .cloned()
            .collect::<Vec<_>>();
        let table_boxes = blocks
            .iter()
            .filter(|block| block.kind.is_table())
            .filter_map(|block| block.bbox.map(|bbox| (bbox, block)))
            .collect::<Vec<_>>();

        let mut seeds = Vec::new();
        for block in &blocks {
            let Some(bbox) = block.bbox else {
                continue;
            };
            let area_ratio = (bbox.width * bbox.height) / page_area;
            if block.kind == LayoutBlockKind::Figure {
                if area_ratio > 0.5 {
                    continue; // page-spanning container: image/vector fallbacks below
                }
                let image = best_image_for(page, bbox);
                let tagged = tagged_for(page, bbox);
                seeds.push(Seed {
                    kind: "figure".to_string(),
                    bbox,
                    image_id: image.map(|image| image.id.clone()),
                    pixel_width: image.map(|image| image.pixel_width),
                    pixel_height: image.map(|image| image.pixel_height),
                    format: image.map(|image| image.format.clone()),
                    vector_primitives: vector_count(page, bbox),
                    table: false,
                    tagged,
                    image_count: usize::from(image.is_some()),
                });
            } else if block.kind.is_table() {
                let rows_chars = block
                    .header
                    .iter()
                    .chain(block.rows.iter().flatten())
                    .map(|cell| cell.text.trim().chars().count())
                    .sum::<usize>();
                let text_chars = block.text_or_empty().trim().chars().count().max(rows_chars);
                let has_rows = !block.header.is_empty() || !block.rows.is_empty();
                let vector_primitives = vector_count(page, bbox);
                let looks_table = if area_ratio > 0.5 {
                    false // page-spanning grid/slide container
                } else if has_rows {
                    text_chars >= 10 && vector_primitives < text_chars.max(20)
                } else {
                    text_chars >= 40 && vector_primitives < (text_chars.saturating_mul(2)).max(20)
                };
                if looks_table {
                    seeds.push(Seed {
                        kind: "table".to_string(),
                        bbox,
                        image_id: None,
                        pixel_width: None,
                        pixel_height: None,
                        format: None,
                        vector_primitives,
                        table: true,
                        tagged: tagged_for(page, bbox),
                        image_count: 0,
                    });
                }
            }
        }

        for image in &page.image_refs {
            let bbox = Rect::from(image.bbox.clone());
            if seeds
                .iter()
                .any(|seed| seed.kind == "figure" && covers(seed.bbox, bbox, 0.6))
            {
                continue;
            }
            let area_ratio = (bbox.width * bbox.height) / page_area;
            let key = (
                image.format.clone(),
                image.pixel_width,
                image.pixel_height,
                image.bbox.width.round() as u32,
                image.bbox.height.round() as u32,
            );
            let repeated = repeat_keys.get(&key).copied().unwrap_or(0) >= 3
                && (image.pixel_width < 120 || area_ratio < 0.01);
            seeds.push(Seed {
                kind: "image".to_string(),
                bbox,
                image_id: Some(image.id.clone()),
                pixel_width: Some(image.pixel_width),
                pixel_height: Some(image.pixel_height),
                format: Some(image.format.clone()),
                vector_primitives: vector_count(page, bbox),
                table: false,
                tagged: tagged_for(page, bbox),
                image_count: if repeated { 3 } else { 1 },
            });
        }

        for (cluster, count) in vector_clusters(page, page_area, page_width, page_height) {
            if seeds
                .iter()
                .any(|seed| covers(seed.bbox, cluster, 0.6) || covers(cluster, seed.bbox, 0.6))
            {
                continue;
            }
            seeds.push(Seed {
                kind: "vector".to_string(),
                bbox: cluster,
                image_id: None,
                pixel_width: None,
                pixel_height: None,
                format: None,
                vector_primitives: count,
                table: false,
                tagged: tagged_for(page, cluster),
                image_count: 0,
            });
        }

        // Merge near-duplicates; table > image > figure > vector.
        seeds.sort_by(|a, b| {
            priority(&b.kind)
                .cmp(&priority(&a.kind))
                .then_with(|| a.bbox.y.total_cmp(&b.bbox.y))
                .then_with(|| a.bbox.x.total_cmp(&b.bbox.x))
        });
        let mut kept: Vec<Seed> = Vec::new();
        for seed in seeds {
            if kept.iter().any(|other| iou(&other.bbox, &seed.bbox) >= 0.8) {
                continue;
            }
            kept.push(seed);
        }

        let page_low_text =
            page.text.chars().filter(|c| !c.is_whitespace()).count() < LOW_TEXT_THRESHOLD;
        let page_chart = page_has_chart(&page.text);
        let page_text_lower = page.text.to_lowercase();
        let cover_page = page.page_number == 1
            && (page_text_lower.contains("bölüm")
                || page_text_lower.contains("bolum")
                || page_text_lower.contains("chapter")
                || page_text_lower.contains("kapak")
                || page_text_lower.contains("cover"));
        let dominant_area = kept
            .iter()
            .map(|seed| (seed.bbox.width * seed.bbox.height) / page_area)
            .fold(0.0, f32::max);
        for mut seed in kept {
            let Some(clamped) = clamp_bbox(seed.bbox, page_width, page_height) else {
                continue;
            };
            seed.bbox = clamped;
            let area_ratio = (seed.bbox.width * seed.bbox.height) / page_area;
            let aspect = seed.bbox.width / seed.bbox.height.max(0.1);
            let repeat = seed.kind == "image" && seed.image_count >= 3;
            let heading = nearest_title(&text_blocks, &headings, seed.bbox, page_width, page_area);
            let (before, after) = context_text(&text_blocks, seed.bbox, page_width);
            let caption = caption_seed(page, &text_blocks, seed.bbox)
                .or_else(|| heading.as_ref().map(|heading| heading.text.clone()));
            let text_inside = text_inside(&text_blocks, seed.bbox);
            let inside_text = inside_text(&text_blocks, seed.bbox, 600);
            if seed.kind == "vector" && text_inside > 500 {
                continue; // whole-slide vector cluster, not a single visual
            }
            let table_overlap = table_boxes
                .iter()
                .any(|(bbox, _)| !seed.table && overlap_ratio(seed.bbox, *bbox) >= 0.6);
            let decorative_band = {
                let center = seed.bbox.y + seed.bbox.height / 2.0;
                center < page_height * 0.06 || center > page_height * 0.94
            };
            let frame_like =
                seed.kind == "vector" && text_inside == 0 && seed.vector_primitives <= 40;
            let secondary_decorative = matches!(seed.kind.as_str(), "figure" | "image")
                && area_ratio < 0.2
                && area_ratio < dominant_area * 0.5
                && !page_chart
                && !seed.table;
            let decorative = seed.image_count >= 3
                || seed.bbox.height <= 2.0
                || seed.bbox.width <= 24.0
                || aspect > 20.0
                || area_ratio < 0.01
                || decorative_band
                || frame_like
                || secondary_decorative
                || (cover_page && matches!(seed.kind.as_str(), "figure" | "image" | "vector"))
                || is_cover_text(caption.as_deref());
            let tagged_alt = tagged_alt(page, seed.bbox);
            let title_text = format!(
                "{} {}",
                heading.as_ref().map_or("", |heading| heading.text.as_str()),
                caption.as_deref().unwrap_or("")
            );
            // Large/whole-slide candidates use only text inside their own
            // region; before/after can leak a neighbouring slide in 6-up
            // pages. Small figures use the nearby context, which often carries
            // the chart title.
            let body_text = if area_ratio > 0.04 {
                inside_text.clone()
            } else {
                format!("{} {} {}", before, after, inside_text)
            };
            let classification = classify(ClassifyInput {
                title: &title_text,
                body: &body_text,
                page_text: &page.text,
                table: seed.table,
                native_image: seed.image_id.is_some(),
                vector_primitives: seed.vector_primitives,
                low_text: page_low_text,
                text_inside_chars: text_inside,
                decorative,
                large: area_ratio >= 0.3,
            });
            let mut scoring = score(&ScoreFeatures {
                area_ratio,
                heading_distance_pt: heading.as_ref().map(|heading| heading.distance),
                caption: tagged_alt.is_some()
                    || caption
                        .as_deref()
                        .is_some_and(|value| caption_regex().is_match(value)),
                tagged: seed.tagged,
                vector_primitives: seed.vector_primitives,
                native_pixel_width: seed.pixel_width,
                bbox_width_pt: seed.bbox.width,
                low_text: page_low_text,
                text_inside_chars: text_inside,
                table_overlap,
                repeat,
                duplicate: false,
                decorative_band,
                aspect_ratio: aspect,
                relevance: classification.relevance.clone(),
            });
            if classification.reconstruction == "text" {
                scoring.score = scoring.score.min(45);
            }
            let suggested = suggested(
                scoring.score,
                &classification.relevance,
                &classification.reconstruction,
                seed.image_id.is_some(),
                &scoring.quality_band,
                page_low_text,
            );
            let confidence = if scoring.score >= 70 || scoring.score <= 20 {
                "high"
            } else if scoring.score >= 40 {
                "medium"
            } else {
                "low"
            };
            let review_mode = if classification.relevance == "decorative"
                || classification.relevance == "seductive"
                || classification.reconstruction == "d2_concept"
                || classification.reconstruction == "table"
                || classification.reconstruction == "text"
            {
                "text-only"
            } else if scoring.score >= 50 {
                "visual-check-recommended"
            } else {
                "text-only"
            };
            let mut reason_parts = vec![format!("{} candidate", classification.content_class)];
            if let Some(heading) = heading.as_ref() {
                reason_parts.push(format!("near heading {:?}", heading.text));
            }
            if !scoring.penalties.is_empty() {
                reason_parts.push(scoring.penalties.join(", "));
            }
            if scoring.quality_band == "low" {
                reason_parts.push(format!(
                    "low effective DPI; recommended width {}%",
                    scoring.recommended_width_percent
                ));
            }
            all.push(VisualCandidate {
                id: String::new(),
                source: source_id.to_string(),
                page: page.page_number,
                kind: seed.kind.clone(),
                bbox: seed.bbox,
                page_width,
                page_height,
                area_ratio,
                native: seed.image_id.is_some(),
                image_id: seed.image_id.clone(),
                pixel_width: seed.pixel_width,
                pixel_height: seed.pixel_height,
                format: seed.format.clone(),
                vector_primitives: seed.vector_primitives,
                text_inside_chars: text_inside,
                heading: heading.as_ref().map(|heading| heading.text.clone()),
                caption_seed: caption.clone(),
                context: CandidateContext {
                    before: before.to_string(),
                    after: after.to_string(),
                    inside: inside_text,
                },
                group_id: heading.as_ref().map(|heading| {
                    format!(
                        "{}:p{}:g{}",
                        source_id,
                        page.page_number,
                        slug(&heading.text)
                    )
                }),
                group_members: 1,
                series: None,
                series_index: None,
                content_class: classification.content_class,
                relevance: Relevance {
                    label: classification.relevance,
                    confidence: classification.relevance_confidence,
                },
                reconstruction: Reconstruction {
                    kind: classification.reconstruction,
                    confidence: classification.reconstruction_confidence,
                    reasons: classification.reasons,
                },
                score: scoring.score,
                score_parts: scoring.score_parts,
                signals: scoring.signals,
                penalties: scoring.penalties,
                suggested,
                confidence: confidence.to_string(),
                recommended_width_percent: scoring.recommended_width_percent,
                effective_dpi: scoring.effective_dpi,
                review_mode: review_mode.to_string(),
                reason: reason_parts.join("; "),
            });
        }
    }

    // Stable IDs: page, y, x, kind, then kind ordinal.
    all.sort_by(|a, b| {
        a.page
            .cmp(&b.page)
            .then_with(|| a.bbox.y.total_cmp(&b.bbox.y))
            .then_with(|| a.bbox.x.total_cmp(&b.bbox.x))
            .then_with(|| a.kind.cmp(&b.kind))
    });
    let mut ordinals: HashMap<(usize, String), usize> = HashMap::new();
    for candidate in &mut all {
        let ordinal = ordinals
            .entry((candidate.page, candidate.kind.clone()))
            .and_modify(|value| *value += 1)
            .or_insert(1);
        let prefix = match candidate.kind.as_str() {
            "figure" => "fig",
            "image" => "img",
            "table" => "tbl",
            _ => "vec",
        };
        candidate.id = format!("{}:p{}:{}{}", source_id, candidate.page, prefix, ordinal);
    }

    merge_nearby(&mut all);
    merge_fragmented_pages(&mut all);

    // Series: adjacent pages with the same heading and content class.
    for index in 0..all.len() {
        let page = all[index].page;
        let heading = all[index].heading.clone();
        let content = all[index].content_class.clone();
        if heading.is_none() {
            continue;
        }
        let series_key = format!(
            "{}:{}:{}",
            source_id,
            slug(heading.as_deref().unwrap_or("")),
            content
        );
        let first = all.iter().position(|candidate| {
            candidate.heading == heading
                && candidate.content_class == content
                && candidate.page + 1 == page
        });
        if let Some(first) = first {
            all[index].series = all[first].series.clone().or(Some(series_key));
            all[index].series_index = Some(all[first].series_index.unwrap_or(1) + 1);
        }
    }

    let total = all.len();
    let mut counts = CandidateCounts {
        total,
        ..CandidateCounts::default()
    };
    for candidate in &all {
        match candidate.suggested.as_str() {
            "embed_image" | "embed_crop" => counts.embed += 1,
            "describe" => counts.describe += 1,
            "redraw" => counts.redraw += 1,
            _ => counts.skip += 1,
        }
        match candidate.relevance.label.as_str() {
            "core" => counts.core += 1,
            "supporting" => counts.supporting += 1,
            "decorative" => counts.decorative += 1,
            "seductive" => counts.seductive += 1,
            _ => {}
        }
    }

    all.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| a.page.cmp(&b.page))
            .then_with(|| a.bbox.y.total_cmp(&b.bbox.y))
            .then_with(|| a.bbox.x.total_cmp(&b.bbox.x))
    });
    let filtered = all
        .into_iter()
        .filter(|candidate| candidate.score >= min_score)
        .collect::<Vec<_>>();
    let bounded = filtered
        .iter()
        .take(limit.max(1))
        .cloned()
        .collect::<Vec<_>>();
    counts.returned = bounded.len();
    let next = (filtered.len() > bounded.len()).then_some(bounded.len());

    Ok(CandidateReport {
        guide,
        source: source_id.to_string(),
        source_digest: digest,
        scorer_version: SCORER_VERSION,
        counts,
        candidates: bounded,
        next,
        policy: "salience is advisory; the host decides".to_string(),
    })
}

fn merge_nearby(candidates: &mut Vec<VisualCandidate>) {
    let mut index = 0;
    while index < candidates.len() {
        let mut other = index + 1;
        while other < candidates.len() {
            let mergeable = {
                let a = &candidates[index];
                let b = &candidates[other];
                a.page == b.page
                    && matches!(a.kind.as_str(), "figure" | "image" | "vector")
                    && matches!(b.kind.as_str(), "figure" | "image" | "vector")
                    && same_group_or_heading(a, b)
                    && bbox_gap(a.bbox, b.bbox) <= 6.0
                    && {
                        let union = union_boxes(&[a.bbox, b.bbox]).unwrap_or(a.bbox);
                        (union.width * union.height) / (a.page_width * a.page_height) <= 0.6
                    }
            };
            if mergeable {
                let b = candidates.remove(other);
                let a = &mut candidates[index];
                let union = union_boxes(&[a.bbox, b.bbox]).unwrap_or(a.bbox);
                let keep_native =
                    a.native && b.native && a.image_id.is_some() && a.image_id == b.image_id;
                a.bbox = union;
                a.area_ratio =
                    (union.width * union.height) / (a.page_width * a.page_height).max(1.0);
                a.vector_primitives += b.vector_primitives;
                a.text_inside_chars += b.text_inside_chars;
                a.group_members += b.group_members;
                a.native = keep_native;
                if !keep_native {
                    a.image_id = None;
                    a.pixel_width = None;
                    a.pixel_height = None;
                    a.format = None;
                    a.effective_dpi = None;
                    a.recommended_width_percent = 90;
                }
                if b.score > a.score {
                    a.score = b.score;
                    a.score_parts = b.score_parts;
                    a.signals = b.signals;
                    a.penalties = b.penalties;
                    a.suggested = b.suggested;
                    a.confidence = b.confidence;
                    a.reconstruction = b.reconstruction;
                    a.content_class = b.content_class;
                }
                if b.relevance.label == "core" {
                    a.relevance = b.relevance;
                }
                if a.reconstruction.kind == "d2_concept" {
                    a.suggested = "redraw".to_string();
                } else if a.suggested == "embed_image" && !a.native {
                    a.suggested = if a.score >= 70 {
                        "embed_crop"
                    } else {
                        "describe"
                    }
                    .to_string();
                }
                a.reason = format!("{}; grouped {} panels", a.reason, a.group_members);
            } else {
                other += 1;
            }
        }
        index += 1;
    }
}

fn merge_fragmented_pages(candidates: &mut Vec<VisualCandidate>) {
    let pages = candidates
        .iter()
        .map(|candidate| candidate.page)
        .collect::<HashSet<_>>();
    for page in pages {
        let indices = candidates
            .iter()
            .enumerate()
            .filter(|(_, candidate)| {
                candidate.page == page
                    && candidate.score >= 30
                    && candidate.relevance.label != "decorative"
                    && matches!(candidate.kind.as_str(), "figure" | "image" | "vector")
            })
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        if indices.len() < 8 {
            continue;
        }
        let Some(base_index) = indices
            .iter()
            .copied()
            .max_by_key(|index| candidates[*index].score)
        else {
            continue;
        };
        let page_area = candidates[base_index].page_width * candidates[base_index].page_height;
        let union = indices
            .iter()
            .map(|index| candidates[*index].bbox)
            .reduce(|left, right| union_boxes(&[left, right]).unwrap_or(left))
            .unwrap_or(candidates[base_index].bbox);
        if (union.width * union.height) / page_area.max(1.0) > 1.001 {
            continue;
        }
        let contains_concept = indices
            .iter()
            .any(|index| candidates[*index].reconstruction.kind == "d2_concept");
        let base_id = candidates[base_index].id.clone();
        let members = indices
            .iter()
            .map(|index| candidates[*index].group_members)
            .sum::<usize>();
        let vector = indices
            .iter()
            .map(|index| candidates[*index].vector_primitives)
            .sum::<usize>();
        let text = indices
            .iter()
            .map(|index| candidates[*index].text_inside_chars)
            .sum::<usize>();
        for index in indices.iter().rev() {
            if *index != base_index {
                candidates.remove(*index);
            }
        }
        let base_index = candidates
            .iter()
            .position(|candidate| candidate.id == base_id)
            .unwrap_or(0);
        let base = &mut candidates[base_index];
        base.bbox = union;
        base.area_ratio = (union.width * union.height) / page_area.max(1.0);
        base.vector_primitives = vector;
        base.text_inside_chars = text;
        base.group_members = members;
        base.native = false;
        base.image_id = None;
        base.pixel_width = None;
        base.pixel_height = None;
        base.format = None;
        base.effective_dpi = None;
        base.recommended_width_percent = 90;
        if base.reconstruction.kind == "d2_concept" || contains_concept {
            base.suggested = "redraw".to_string();
        } else if base.score >= 70 {
            base.suggested = "embed_crop".to_string();
        } else {
            base.suggested = "describe".to_string();
        }
        base.reason = format!("{}; fragmented visual group", base.reason);
    }
}

fn same_group_or_heading(a: &VisualCandidate, b: &VisualCandidate) -> bool {
    if let (Some(left), Some(right)) = (a.group_id.as_deref(), b.group_id.as_deref())
        && left == right
    {
        return true;
    }
    if let (Some(left), Some(right)) = (a.heading.as_deref(), b.heading.as_deref())
        && left == right
    {
        return true;
    }
    a.heading.is_none()
        && b.heading.is_none()
        && a.area_ratio.min(b.area_ratio) < 0.03
        && bbox_gap(a.bbox, b.bbox) <= 4.0
}

fn bbox_gap(a: Rect, b: Rect) -> f32 {
    let gap_x = (a.x - (b.x + b.width)).max(b.x - (a.x + a.width)).max(0.0);
    let gap_y = (a.y - (b.y + b.height))
        .max(b.y - (a.y + a.height))
        .max(0.0);
    gap_x.max(gap_y)
}

fn clamp_bbox(bbox: Rect, page_width: f32, page_height: f32) -> Option<Rect> {
    let left = bbox.x.max(0.0);
    let top = bbox.y.max(0.0);
    let right = (bbox.x + bbox.width).min(page_width);
    let bottom = (bbox.y + bbox.height).min(page_height);
    if right <= left || bottom <= top {
        return None;
    }
    Some(Rect {
        x: left,
        y: top,
        width: right - left,
        height: bottom - top,
    })
}

fn priority(kind: &str) -> u8 {
    match kind {
        "table" => 4,
        "image" => 3,
        "figure" => 2,
        _ => 1,
    }
}

fn suggested(
    score: u32,
    relevance: &str,
    reconstruction: &str,
    native: bool,
    quality: &str,
    low_text: bool,
) -> String {
    if relevance == "decorative" || relevance == "seductive" {
        return "skip".to_string();
    }
    if reconstruction == "table" {
        return "native_table".to_string();
    }
    if reconstruction == "text" {
        return "skip".to_string();
    }
    if reconstruction == "d2_concept" || reconstruction == "d2_schematic" {
        return if score >= 50 { "redraw" } else { "describe" }.to_string();
    }
    if reconstruction == "unique" {
        if score >= 70 && quality != "low" {
            return if native { "embed_image" } else { "embed_crop" }.to_string();
        }
        if score >= 50 || low_text {
            return "describe".to_string();
        }
    }
    if score >= 70 {
        return if native { "embed_image" } else { "embed_crop" }.to_string();
    }
    if score >= 30 {
        return "describe".to_string();
    }
    "skip".to_string()
}

fn best_image_for(
    page: &liteparse::types::ParsedPage,
    bbox: Rect,
) -> Option<&liteparse::types::ImageRef> {
    page.image_refs
        .iter()
        .filter(|image| {
            let image_bbox = Rect::from(image.bbox.clone());
            overlap_ratio(bbox, image_bbox) >= 0.5
        })
        .max_by(|a, b| {
            let aa = a.bbox.width * a.bbox.height;
            let bb = b.bbox.width * b.bbox.height;
            aa.total_cmp(&bb)
        })
}

fn tagged_for(page: &liteparse::types::ParsedPage, bbox: Rect) -> bool {
    page.struct_nodes.iter().any(|node| {
        let role = node.role.to_lowercase();
        (role.contains("figure") || role.contains("formula") || role.contains("table"))
            && node
                .bbox
                .as_ref()
                .is_some_and(|node_bbox| overlap_ratio(bbox, Rect::from(node_bbox.clone())) >= 0.5)
    })
}

fn tagged_alt(page: &liteparse::types::ParsedPage, bbox: Rect) -> Option<String> {
    page.struct_nodes.iter().find_map(|node| {
        let role = node.role.to_lowercase();
        if !(role.contains("figure") || role.contains("formula") || role.contains("table")) {
            return None;
        }
        let node_bbox = node.bbox.as_ref().map(|value| Rect::from(value.clone()))?;
        if overlap_ratio(bbox, node_bbox) < 0.5 {
            return None;
        }
        node.alt_text
            .clone()
            .filter(|value| !value.trim().is_empty())
    })
}

fn vector_count(page: &liteparse::types::ParsedPage, bbox: Rect) -> usize {
    let Some(graphics) = page.vector_graphics.as_ref() else {
        return 0;
    };
    let shapes = graphics
        .shapes
        .iter()
        .filter(|shape| overlap_ratio(bbox, Rect::from(shape.bbox.clone())) > 0.05)
        .count();
    let lines = graphics
        .lines
        .iter()
        .filter(|line| {
            let line_bbox = Rect {
                x: line.x1.min(line.x2),
                y: line.y1.min(line.y2),
                width: (line.x1 - line.x2).abs().max(1.0),
                height: (line.y1 - line.y2).abs().max(1.0),
            };
            overlap_ratio(bbox, line_bbox) > 0.05
        })
        .count();
    shapes + lines
}

fn vector_clusters(
    page: &liteparse::types::ParsedPage,
    page_area: f32,
    page_width: f32,
    page_height: f32,
) -> Vec<(Rect, usize)> {
    let Some(graphics) = page.vector_graphics.as_ref() else {
        return Vec::new();
    };
    let mut primitives = graphics
        .shapes
        .iter()
        .map(|shape| Primitive {
            bbox: Rect::from(shape.bbox.clone()),
        })
        .collect::<Vec<_>>();
    primitives.extend(graphics.lines.iter().map(|line| Primitive {
        bbox: Rect {
            x: line.x1.min(line.x2),
            y: line.y1.min(line.y2),
            width: (line.x1 - line.x2).abs().max(1.0),
            height: (line.y1 - line.y2).abs().max(1.0),
        },
    }));
    // Slide borders, title rules and other page-spanning chrome connect every
    // region; they are not part of a chart/diagram cluster.
    primitives.retain(|primitive| {
        primitive.bbox.width <= page_width * 0.6 && primitive.bbox.height <= page_height * 0.6
    });
    if primitives.len() < 8 {
        return Vec::new();
    }

    // Union-find over spatially adjacent primitives. Expand by 6 pt so a
    // chart's axes, ticks, labels and marks join one cluster.
    let mut parent = (0..primitives.len()).collect::<Vec<_>>();
    fn find(parent: &mut [usize], mut index: usize) -> usize {
        while parent[index] != index {
            parent[index] = parent[parent[index]];
            index = parent[index];
        }
        index
    }
    for i in 0..primitives.len() {
        for j in (i + 1)..primitives.len() {
            if near(&primitives[i].bbox, &primitives[j].bbox, 6.0) {
                let left = find(&mut parent, i);
                let right = find(&mut parent, j);
                if left != right {
                    parent[right] = left;
                }
            }
        }
    }
    let mut groups: HashMap<usize, Vec<Rect>> = HashMap::new();
    for (index, primitive) in primitives.iter().enumerate() {
        let root = find(&mut parent, index);
        groups.entry(root).or_default().push(primitive.bbox);
    }
    groups
        .into_values()
        .filter_map(|boxes| {
            let bbox = union_boxes(&boxes)?;
            let area_ratio = (bbox.width * bbox.height) / page_area;
            (boxes.len() >= 8 && (0.005..=0.85).contains(&area_ratio) && bbox.width > 8.0)
                .then_some((bbox, boxes.len()))
        })
        .collect()
}

fn near(a: &Rect, b: &Rect, padding: f32) -> bool {
    let left = a.x.max(b.x) - padding;
    let right = (a.x + a.width).min(b.x + b.width) + padding;
    let top = a.y.max(b.y) - padding;
    let bottom = (a.y + a.height).min(b.y + b.height) + padding;
    right >= left && bottom >= top
}

fn union_boxes(boxes: &[Rect]) -> Option<Rect> {
    let first = boxes.first()?;
    let mut left = first.x;
    let mut top = first.y;
    let mut right = first.x + first.width;
    let mut bottom = first.y + first.height;
    for bbox in &boxes[1..] {
        left = left.min(bbox.x);
        top = top.min(bbox.y);
        right = right.max(bbox.x + bbox.width);
        bottom = bottom.max(bbox.y + bbox.height);
    }
    Some(Rect {
        x: left,
        y: top,
        width: (right - left).max(1.0),
        height: (bottom - top).max(1.0),
    })
}

fn covers(outer: Rect, inner: Rect, ratio: f32) -> bool {
    overlap_ratio(outer, inner) >= ratio
}

fn overlap_ratio(a: Rect, b: Rect) -> f32 {
    let left = a.x.max(b.x);
    let top = a.y.max(b.y);
    let right = (a.x + a.width).min(b.x + b.width);
    let bottom = (a.y + a.height).min(b.y + b.height);
    if right <= left || bottom <= top {
        return 0.0;
    }
    let intersection = (right - left) * (bottom - top);
    intersection / (b.width * b.height).max(1.0)
}

fn iou(a: &Rect, b: &Rect) -> f32 {
    let left = a.x.max(b.x);
    let top = a.y.max(b.y);
    let right = (a.x + a.width).min(b.x + b.width);
    let bottom = (a.y + a.height).min(b.y + b.height);
    if right <= left || bottom <= top {
        return 0.0;
    }
    let intersection = (right - left) * (bottom - top);
    let union = a.width * a.height + b.width * b.height - intersection;
    intersection / union.max(1.0)
}

fn page_text_blocks(page: &liteparse::types::ParsedPage, layout: &[LayoutBlock]) -> Vec<TextBlock> {
    let mut items = page
        .text_items
        .iter()
        .filter_map(|item| {
            let text = item.text.trim().to_string();
            if text.is_empty() {
                return None;
            }
            let font_size = item.font_size.or(item.font_height).unwrap_or(0.0);
            Some(TextBlock {
                bbox: Rect {
                    x: item.x,
                    y: item.y,
                    width: item.width.max(1.0),
                    height: item.height.max(1.0),
                },
                text,
                heading: font_size >= 14.0,
            })
        })
        .collect::<Vec<_>>();
    if items.is_empty() {
        return layout
            .iter()
            .filter_map(|block| {
                let bbox = block.bbox?;
                let text = block.text_or_empty().trim().to_string();
                if text.is_empty() {
                    return None;
                }
                Some(TextBlock {
                    bbox,
                    text,
                    heading: block.kind == LayoutBlockKind::Heading,
                })
            })
            .collect();
    }
    items.sort_by(|a, b| {
        a.bbox
            .y
            .total_cmp(&b.bbox.y)
            .then_with(|| a.bbox.x.total_cmp(&b.bbox.x))
    });
    let mut lines: Vec<TextBlock> = Vec::new();
    for item in items {
        let merge = lines.last_mut().is_some_and(|last| {
            let same_line = (last.bbox.y - item.bbox.y).abs() <= 2.0;
            let gap = item.bbox.x - (last.bbox.x + last.bbox.width);
            same_line && (-4.0..=12.0).contains(&gap)
        });
        if merge {
            let last = lines.last_mut().expect("line");
            last.text.push(' ');
            last.text.push_str(&item.text);
            let right = (last.bbox.x + last.bbox.width).max(item.bbox.x + item.bbox.width);
            let bottom = (last.bbox.y + last.bbox.height).max(item.bbox.y + item.bbox.height);
            last.bbox.width = right - last.bbox.x;
            last.bbox.height = bottom - last.bbox.y;
            last.heading |= item.heading;
        } else {
            lines.push(item);
        }
    }
    lines
}

fn nearest_title(
    blocks: &[TextBlock],
    headings: &[TextBlock],
    bbox: Rect,
    page_width: f32,
    page_area: f32,
) -> Option<HeadingHit> {
    let inside_top = |block: &TextBlock| {
        block.bbox.y >= bbox.y
            && block.bbox.y + block.bbox.height <= bbox.y + bbox.height * 0.35
            && horizontally_related(block.bbox, bbox, page_width)
    };
    let above = |block: &TextBlock| {
        block.bbox.y + block.bbox.height <= bbox.y + 8.0
            && horizontally_related(block.bbox, bbox, page_width)
    };
    let short = |block: &TextBlock| {
        let text = block.text.trim();
        !text.starts_with(['•', '-', '–', '*'])
            && !text.contains("://")
            && !text.contains("www.")
            && !text.to_lowercase().contains("aspx")
            && !text.chars().all(|character| character.is_ascii_digit())
            && (4..=100).contains(&text.chars().count())
    };
    let inside_large = blocks
        .iter()
        .filter(|block| block.heading && short(block) && inside_top(block))
        .map(|block| {
            let distance = (bbox.y - (block.bbox.y + block.bbox.height)).abs();
            HeadingHit {
                text: block.text.clone(),
                distance,
            }
        })
        .min_by(|a, b| a.distance.total_cmp(&b.distance));
    // A whole-slide vector cluster needs the title inside its own region; the
    // nearest block above may belong to the previous slide. The topmost short
    // line inside the region is the slide title when font metadata is absent.
    if bbox.width * bbox.height > page_area * 0.04 {
        if let Some(title) = blocks
            .iter()
            .filter(|block| {
                block.bbox.y >= bbox.y
                    && block.bbox.y + block.bbox.height <= bbox.y + bbox.height
                    && horizontally_related(block.bbox, bbox, page_width)
                    && short(block)
            })
            .min_by(|a, b| a.bbox.y.total_cmp(&b.bbox.y))
        {
            return Some(HeadingHit {
                text: title.text.clone(),
                distance: 0.0,
            });
        }
        if let Some(title) = inside_large {
            return Some(title);
        }
    }
    if let Some(title) = headings
        .iter()
        .filter(|heading| above(heading) && short(heading))
        .map(|heading| {
            let distance = (bbox.y - (heading.bbox.y + heading.bbox.height)).max(0.0);
            HeadingHit {
                text: heading.text.clone(),
                distance,
            }
        })
        .min_by(|a, b| a.distance.total_cmp(&b.distance))
    {
        return Some(title);
    }
    blocks
        .iter()
        .filter(|block| above(block) && short(block))
        .map(|block| {
            let distance = (bbox.y - (block.bbox.y + block.bbox.height)).max(0.0);
            HeadingHit {
                text: block.text.clone(),
                distance,
            }
        })
        .min_by(|a, b| a.distance.total_cmp(&b.distance))
        .or(inside_large)
}

#[derive(Debug, Clone)]
struct HeadingHit {
    text: String,
    distance: f32,
}

fn context_text(blocks: &[TextBlock], bbox: Rect, page_width: f32) -> (String, String) {
    let mut before = blocks
        .iter()
        .filter(|block| {
            block.bbox.y + block.bbox.height <= bbox.y + 8.0
                && horizontally_related(block.bbox, bbox, page_width)
        })
        .collect::<Vec<_>>();
    before.sort_by(|a, b| b.bbox.y.total_cmp(&a.bbox.y));
    let before_text = before
        .into_iter()
        .take(3)
        .map(|block| block.text.as_str())
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join(" ");
    let mut after = blocks
        .iter()
        .filter(|block| {
            block.bbox.y >= bbox.y + bbox.height - 8.0
                && horizontally_related(block.bbox, bbox, page_width)
        })
        .collect::<Vec<_>>();
    after.sort_by(|a, b| a.bbox.y.total_cmp(&b.bbox.y));
    let after_text = after
        .into_iter()
        .take(2)
        .map(|block| block.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    (truncate(&before_text, 420), truncate(&after_text, 380))
}

fn horizontally_related(text: Rect, bbox: Rect, page_width: f32) -> bool {
    let left = text.x.max(bbox.x);
    let right = (text.x + text.width).min(bbox.x + bbox.width);
    if right > left {
        return true;
    }
    // Fallback for narrow labels: same page half/column.
    (text.x + text.width / 2.0 < page_width / 2.0) == (bbox.x + bbox.width / 2.0 < page_width / 2.0)
}

fn inside_text(blocks: &[TextBlock], bbox: Rect, max: usize) -> String {
    let text = blocks
        .iter()
        .filter(|block| {
            let center_x = block.bbox.x + block.bbox.width / 2.0;
            let center_y = block.bbox.y + block.bbox.height / 2.0;
            (center_x >= bbox.x
                && center_x <= bbox.x + bbox.width
                && center_y >= bbox.y
                && center_y <= bbox.y + bbox.height)
                || overlap_ratio(bbox, block.bbox) >= 0.3
        })
        .map(|block| block.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    truncate(&text, max)
}

fn text_inside(blocks: &[TextBlock], bbox: Rect) -> usize {
    blocks
        .iter()
        .filter(|block| {
            let center_x = block.bbox.x + block.bbox.width / 2.0;
            let center_y = block.bbox.y + block.bbox.height / 2.0;
            (center_x >= bbox.x
                && center_x <= bbox.x + bbox.width
                && center_y >= bbox.y
                && center_y <= bbox.y + bbox.height)
                || overlap_ratio(bbox, block.bbox) >= 0.3
        })
        .map(|block| block.text.chars().count())
        .sum()
}

fn caption_seed(
    page: &liteparse::types::ParsedPage,
    blocks: &[TextBlock],
    bbox: Rect,
) -> Option<String> {
    if let Some(alt) = tagged_alt(page, bbox) {
        return Some(alt);
    }
    let regex = caption_regex();
    blocks
        .iter()
        .filter(|block| {
            block.bbox.y >= bbox.y + bbox.height - 8.0
                && block.bbox.y <= bbox.y + bbox.height + 40.0
        })
        .find(|block| regex.is_match(&block.text))
        .map(|block| block.text.clone())
}

fn caption_regex() -> &'static Regex {
    static CAPTION: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    CAPTION.get_or_init(|| {
        Regex::new(r"(?i)^(şekil|sekil|figure|tablo|table|grafik|chart|diyagram|diagram|algoritma|algorithm|görsel|gorsel)\b")
            .expect("caption regex")
    })
}

fn is_cover_text(text: Option<&str>) -> bool {
    text.is_some_and(|value| {
        let value = value.to_lowercase();
        value.contains("kapak") || value.contains("cover") || value.contains("karikatür")
    })
}

fn truncate(value: &str, max: usize) -> String {
    let mut out = value.chars().take(max).collect::<String>();
    if value.chars().count() > max {
        out.push('…');
    }
    out
}

fn slug(value: &str) -> String {
    value
        .to_lowercase()
        .chars()
        .map(|character| {
            if character.is_alphanumeric() {
                character
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

pub fn candidate_json(report: &CandidateReport) -> serde_json::Value {
    json!(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bbox_clamping_keeps_candidates_on_the_page() {
        let bbox = Rect {
            x: -10.0,
            y: 20.0,
            width: 100.0,
            height: 100.0,
        };
        let clamped = clamp_bbox(bbox, 80.0, 100.0).expect("clamped");
        assert_eq!(clamped.x, 0.0);
        assert_eq!(clamped.width, 80.0);
        assert_eq!(clamped.y + clamped.height, 100.0);
    }

    #[test]
    fn suggested_actions_follow_the_reconstruction() {
        assert_eq!(
            suggested(80, "core", "d2_concept", false, "high", false),
            "redraw"
        );
        assert_eq!(
            suggested(80, "core", "unique", true, "high", false),
            "embed_image"
        );
        assert_eq!(
            suggested(80, "core", "unique", false, "high", false),
            "embed_crop"
        );
        assert_eq!(
            suggested(80, "decorative", "unique", true, "high", false),
            "skip"
        );
        assert_eq!(
            suggested(55, "core", "unique", true, "low", false),
            "describe"
        );
    }

    #[test]
    fn gap_measurement_handles_overlap_and_separation() {
        let a = Rect {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
        };
        let b = Rect {
            x: 12.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
        };
        assert_eq!(bbox_gap(a, b), 2.0);
        assert_eq!(bbox_gap(b, a), 2.0);
        assert_eq!(bbox_gap(a, a), 0.0);
    }

    #[test]
    fn score_sort_puts_the_most_salient_first() {
        let mut values = [(1, 50), (2, 80), (3, 20)];
        values.sort_by_key(|value| std::cmp::Reverse(value.1));
        assert_eq!(values[0].1, 80);
    }
}
