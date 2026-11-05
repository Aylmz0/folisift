//! Deterministic salience scoring for visual candidates.
//!
//! The score is an advisory ranking, not a truth claim. Every part is returned
//! so the host can see why a candidate was ranked.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::visual_quality::{effective_dpi, recommended_width_percent};

#[derive(Debug, Clone)]
pub struct ScoreFeatures {
    pub area_ratio: f32,
    pub heading_distance_pt: Option<f32>,
    pub caption: bool,
    pub tagged: bool,
    pub vector_primitives: usize,
    pub native_pixel_width: Option<u32>,
    pub bbox_width_pt: f32,
    pub low_text: bool,
    pub text_inside_chars: usize,
    pub table_overlap: bool,
    pub repeat: bool,
    pub duplicate: bool,
    pub decorative_band: bool,
    pub aspect_ratio: f32,
    pub relevance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScoreResult {
    pub score: u32,
    pub score_parts: BTreeMap<String, f32>,
    pub signals: Vec<String>,
    pub penalties: Vec<String>,
    pub effective_dpi: Option<f32>,
    pub recommended_width_percent: u8,
    pub quality_band: String,
}

fn area_score(area: f32) -> f32 {
    if area < 0.01 {
        0.05
    } else if area < 0.04 {
        0.45
    } else if area <= 0.50 {
        1.0
    } else if area <= 0.75 {
        0.55
    } else {
        0.25
    }
}

fn heading_score(distance: Option<f32>) -> f32 {
    match distance {
        Some(value) if value <= 72.0 => 1.0,
        Some(value) if value <= 180.0 => 0.75,
        Some(value) if value <= 360.0 => 0.45,
        Some(_) => 0.2,
        None => 0.1,
    }
}

fn dpi_score(dpi: Option<f32>) -> f32 {
    match dpi {
        Some(value) if value >= 200.0 => 1.0,
        Some(value) if value >= 150.0 => 0.85,
        Some(value) if value >= 120.0 => 0.65,
        Some(value) if value >= 96.0 => 0.4,
        Some(_) => 0.2,
        None => 0.65,
    }
}

pub fn score(features: &ScoreFeatures) -> ScoreResult {
    let mut parts = BTreeMap::new();
    let mut signals = Vec::new();
    let mut penalties = Vec::new();

    let area = area_score(features.area_ratio) * 20.0;
    parts.insert("area".to_string(), round1(area));
    if features.area_ratio >= 0.04 && features.area_ratio <= 0.5 {
        signals.push("area".to_string());
    }

    let heading = heading_score(features.heading_distance_pt) * 12.0;
    parts.insert("heading".to_string(), round1(heading));
    if features
        .heading_distance_pt
        .is_some_and(|value| value <= 180.0)
    {
        signals.push("near-heading".to_string());
    }

    let caption = if features.caption { 1.0 } else { 0.1 } * 18.0;
    parts.insert("caption".to_string(), round1(caption));
    if features.caption {
        signals.push("caption".to_string());
    }

    let complexity = if features.native_pixel_width.is_some() {
        let base = 0.6_f32;
        let vector = (features.vector_primitives as f32 / 40.0).min(0.4);
        (base + vector).min(1.0)
    } else if features.vector_primitives > 0 {
        (0.4 + (features.vector_primitives as f32 / 40.0).min(0.4)).min(1.0)
    } else {
        0.0
    } * 18.0;
    parts.insert("complexity".to_string(), round1(complexity));
    if features.native_pixel_width.is_some() {
        signals.push("native-image".to_string());
    }
    if features.vector_primitives >= 8 {
        signals.push("vector-content".to_string());
    }

    let dpi = features
        .native_pixel_width
        .and_then(|width| effective_dpi(width, features.bbox_width_pt));
    let quality = dpi_score(dpi) * 14.0;
    parts.insert("quality".to_string(), round1(quality));

    let low_text = if features.low_text { 8.0 } else { 0.0 };
    parts.insert("low_text".to_string(), low_text);
    if features.low_text {
        signals.push("low-text-page".to_string());
    }

    let structure = if features.tagged { 10.0 } else { 0.0 };
    parts.insert("structure".to_string(), structure);
    if features.tagged {
        signals.push("tagged-figure".to_string());
    }

    let relevance_bonus = match features.relevance.as_str() {
        "core" => 15.0,
        "supporting" => 5.0,
        _ => 0.0,
    };
    parts.insert("relevance_bonus".to_string(), relevance_bonus);
    if relevance_bonus > 0.0 {
        signals.push(format!("relevance:{}", features.relevance));
    }

    let mut total: f32 = parts.values().sum();

    if features.text_inside_chars > 200 {
        let value = 15.0 * (features.text_inside_chars as f32 / 1000.0).min(1.0);
        total -= value;
        penalties.push(format!("text-inside:-{value:.1}"));
    }
    if features.table_overlap {
        total -= 18.0;
        penalties.push("table-overlap:-18.0".to_string());
    }
    if features.repeat {
        total -= 25.0;
        penalties.push("repeated-decoration:-25.0".to_string());
    }
    if features.duplicate {
        total -= 25.0;
        penalties.push("duplicate-image:-25.0".to_string());
    }
    if features.decorative_band {
        total -= 20.0;
        penalties.push("header-footer-band:-20.0".to_string());
    }
    if features.area_ratio < 0.01 {
        total -= 20.0;
        penalties.push("tiny:-20.0".to_string());
    }
    if features.aspect_ratio > 8.0 || features.aspect_ratio < 1.0 / 8.0 {
        total -= 10.0;
        penalties.push("extreme-aspect:-10.0".to_string());
    }
    if features.aspect_ratio > 20.0 {
        total -= 20.0;
        penalties.push("strip-aspect:-20.0".to_string());
    }

    if features.relevance == "decorative" || features.relevance == "seductive" {
        total = total.min(20.0);
        penalties.push("relevance-cap:20.0".to_string());
    }

    let score = total.clamp(0.0, 100.0).round() as u32;
    let quality_band = match dpi {
        Some(value) if value >= 150.0 => "high",
        Some(value) if value >= 96.0 => "medium",
        Some(_) => "low",
        None => {
            if features.vector_primitives > 0 {
                "high"
            } else {
                "medium"
            }
        }
    }
    .to_string();

    let recommended_width_percent = features
        .native_pixel_width
        .map(recommended_width_percent)
        .unwrap_or(90);

    ScoreResult {
        score,
        score_parts: parts,
        signals,
        penalties,
        effective_dpi: dpi,
        recommended_width_percent,
        quality_band,
    }
}

fn round1(value: f32) -> f32 {
    (value * 10.0).round() / 10.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn features() -> ScoreFeatures {
        ScoreFeatures {
            area_ratio: 0.2,
            heading_distance_pt: Some(40.0),
            caption: true,
            tagged: false,
            vector_primitives: 20,
            native_pixel_width: Some(1200),
            bbox_width_pt: 300.0,
            low_text: false,
            text_inside_chars: 0,
            table_overlap: false,
            repeat: false,
            duplicate: false,
            decorative_band: false,
            aspect_ratio: 1.4,
            relevance: "core".to_string(),
        }
    }

    #[test]
    fn a_core_captioned_chart_scores_as_embed_candidate() {
        let result = score(&features());
        assert!(result.score >= 70, "{result:?}");
        assert_eq!(result.quality_band, "high");
    }

    #[test]
    fn decorative_visuals_are_capped_below_skip_threshold() {
        let mut input = features();
        input.relevance = "decorative".to_string();
        let result = score(&input);
        assert!(result.score <= 20, "{result:?}");
        assert!(
            result
                .penalties
                .iter()
                .any(|item| item.contains("relevance-cap"))
        );
    }

    #[test]
    fn low_resolution_native_image_loses_quality_points() {
        let mut input = features();
        input.native_pixel_width = Some(200);
        input.bbox_width_pt = 180.0;
        let result = score(&input);
        assert!(result.effective_dpi.unwrap() < 120.0);
        assert_eq!(result.quality_band, "low");
        assert!(result.recommended_width_percent <= 45);
    }

    #[test]
    fn text_inside_and_repeat_are_explicit_penalties() {
        let mut input = features();
        input.text_inside_chars = 900;
        input.repeat = true;
        let result = score(&input);
        assert!(
            result
                .penalties
                .iter()
                .any(|item| item.contains("text-inside"))
        );
        assert!(
            result
                .penalties
                .iter()
                .any(|item| item.contains("repeated"))
        );
    }
}
