//! Raster quality checks for embedded source visuals.
//!
//! These checks are deliberately conservative: a blank or unreadably small
//! asset is a build failure, while a merely low-resolution asset is a warning
//! unless the host records a `quality_exception`.

use std::collections::HashSet;

use image::GenericImageView;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// Quality evidence stored in the visual audit and returned by the extractor.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QualityReport {
    pub width: u32,
    pub height: u32,
    pub short_edge: u32,
    pub aspect_ratio: f32,
    pub blank: bool,
    pub unique_colors: usize,
    pub luminance_range: u8,
    pub effective_dpi: Option<f32>,
    pub file_bytes: usize,
}

impl QualityReport {
    /// Conditions that make the asset unsafe to embed, regardless of the
    /// visual gate. The caller may override the DPI check with an explicit
    /// `quality_exception` on the audit entry.
    pub fn hard_failures(&self, allow_low_dpi: bool) -> Vec<String> {
        let mut failures = Vec::new();
        if self.blank {
            failures.push("asset is blank or nearly blank".to_string());
        }
        if self.short_edge < 320 {
            failures.push(format!(
                "asset short edge is {} px; minimum is 320 px",
                self.short_edge
            ));
        }
        if !allow_low_dpi
            && let Some(dpi) = self.effective_dpi
            && dpi < 96.0
        {
            failures.push(format!(
                "effective DPI is {dpi:.0}; minimum is 96 without a quality_exception"
            ));
        }
        failures
    }

    pub fn warnings(&self) -> Vec<String> {
        let mut warnings = Vec::new();
        if let Some(dpi) = self.effective_dpi
            && (96.0..150.0).contains(&dpi)
        {
            warnings.push(format!("effective DPI is {dpi:.0}; 150+ is recommended"));
        }
        if self.aspect_ratio > 8.0 || self.aspect_ratio < 1.0 / 8.0 {
            warnings.push(format!(
                "unusual aspect ratio {:.2}; check the crop",
                self.aspect_ratio
            ));
        }
        if self.file_bytes > 8 * 1024 * 1024 {
            warnings.push(format!(
                "asset is {:.1} MB; check output size",
                self.file_bytes as f32 / (1024.0 * 1024.0)
            ));
        }
        warnings
    }
}

/// Inspect PNG/JPEG bytes. Alpha is composited on white before the blank
/// check so a transparent logo is not mistaken for an empty image.
pub fn inspect(bytes: &[u8], effective_dpi: Option<f32>) -> Result<QualityReport> {
    let decoded = image::load_from_memory(bytes).map_err(|error| Error::Parse {
        path: std::path::PathBuf::from("<visual asset>"),
        message: format!("decode visual asset: {error}"),
    })?;
    let (width, height) = decoded.dimensions();
    if width == 0 || height == 0 {
        return Err(Error::Parse {
            path: std::path::PathBuf::from("<visual asset>"),
            message: "visual asset has zero dimensions".to_string(),
        });
    }
    let rgba = decoded.to_rgba8();
    let step_x = (width / 447).max(1);
    let step_y = (height / 447).max(1);
    let mut colors = HashSet::new();
    let mut min_luma = 255u8;
    let mut max_luma = 0u8;

    for y in (0..height).step_by(step_y as usize) {
        for x in (0..width).step_by(step_x as usize) {
            let pixel = rgba.get_pixel(x, y);
            let alpha = u32::from(pixel[3]);
            let composite = |channel: u8| -> u8 {
                ((u32::from(channel) * alpha + 255 * (255 - alpha)) / 255) as u8
            };
            let r = composite(pixel[0]);
            let g = composite(pixel[1]);
            let b = composite(pixel[2]);
            let luma =
                ((u32::from(r) * 299 + u32::from(g) * 587 + u32::from(b) * 114) / 1000) as u8;
            min_luma = min_luma.min(luma);
            max_luma = max_luma.max(luma);
            colors.insert((u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b));
            if colors.len() > 4096 {
                // The exact count no longer matters; keep the set bounded.
                colors.insert(0xFFFF_FFFF);
            }
        }
    }

    let unique_colors = colors.len();
    let luminance_range = max_luma.saturating_sub(min_luma);
    Ok(QualityReport {
        width,
        height,
        short_edge: width.min(height),
        aspect_ratio: width as f32 / height as f32,
        blank: luminance_range < 8 || unique_colors < 2,
        unique_colors,
        luminance_range,
        effective_dpi,
        file_bytes: bytes.len(),
    })
}

/// Convert a pixel width and a viewport bbox width (points) into effective DPI.
pub fn effective_dpi(pixel_width: u32, bbox_width_pt: f32) -> Option<f32> {
    if bbox_width_pt <= 0.0 {
        return None;
    }
    let inches = bbox_width_pt / 72.0;
    Some(pixel_width as f32 / inches)
}

/// Largest print width (35–90% of a 16 cm content column) that keeps at least
/// 150 DPI for the given pixel width. A value below 35 is returned as 35 so
/// the caller can still see that quality is marginal.
pub fn recommended_width_percent(pixel_width: u32) -> u8 {
    let content_inches = 6.3_f32;
    let raw = (pixel_width as f32 / (150.0 * content_inches) * 100.0).round();
    (raw.clamp(35.0, 90.0)) as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    fn png(width: u32, height: u32, color: Rgba<u8>) -> Vec<u8> {
        let image = RgbaImage::from_pixel(width, height, color);
        let mut bytes = Vec::new();
        image::DynamicImage::ImageRgba8(image)
            .write_to(
                &mut std::io::Cursor::new(&mut bytes),
                image::ImageFormat::Png,
            )
            .expect("png");
        bytes
    }

    #[test]
    fn a_white_image_is_blank_and_fails() {
        let bytes = png(400, 300, Rgba([255, 255, 255, 255]));
        let report = inspect(&bytes, Some(200.0)).expect("quality");
        assert!(report.blank);
        assert!(!report.hard_failures(false).is_empty());
    }

    #[test]
    fn a_colored_image_passes_the_blank_check() {
        let mut image = RgbaImage::new(400, 400);
        for (x, y, pixel) in image.enumerate_pixels_mut() {
            *pixel = if (x / 20 + y / 20) % 2 == 0 {
                Rgba([30, 90, 150, 255])
            } else {
                Rgba([240, 220, 180, 255])
            };
        }
        let mut bytes = Vec::new();
        image::DynamicImage::ImageRgba8(image)
            .write_to(
                &mut std::io::Cursor::new(&mut bytes),
                image::ImageFormat::Png,
            )
            .expect("png");
        let report = inspect(&bytes, Some(200.0)).expect("quality");
        assert!(!report.blank);
        assert!(report.hard_failures(false).is_empty());
    }

    #[test]
    fn effective_dpi_uses_the_bbox_width() {
        let dpi = effective_dpi(1200, 288.0).expect("dpi");
        assert!((dpi - 300.0).abs() < 0.1);
        assert_eq!(recommended_width_percent(1200), 90);
        assert_eq!(recommended_width_percent(251), 35);
    }
}
