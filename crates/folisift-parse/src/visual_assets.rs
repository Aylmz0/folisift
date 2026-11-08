//! Materialize native images and page crops for the visual pipeline.
//!
//! Assets are local-only and written under a guide's `data/crops` or
//! `data/figures` directory. The hash is the content identity used by the
//! audit and build gate.

use std::path::{Path, PathBuf};

use image::ImageFormat;
use liteparse::config::OutputFormat;
use liteparse::{LiteParse, LiteParseConfig};
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result, io};
use crate::layout::Rect;
use crate::pdfium;
use crate::render::CropBox;
use crate::visual_quality::{QualityReport, effective_dpi, inspect};

pub const MAX_CROP_DPI: f32 = 500.0;
pub const MIN_CROP_DPI: f32 = 72.0;
pub const MAX_RENDER_PIXELS: u64 = 40_000_000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NativeAsset {
    pub page: u32,
    pub image_id: String,
    pub asset: String,
    pub hash: String,
    /// Format of the embedded source image.
    pub native_format: String,
    /// Format actually written to `asset`.
    pub format: String,
    pub width: u32,
    pub height: u32,
    pub bbox: Rect,
    pub effective_dpi: Option<f32>,
    pub quality: QualityReport,
    pub duplicate_of: Option<String>,
    pub local_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CropAsset {
    pub page: u32,
    pub asset: String,
    pub hash: String,
    pub crop: String,
    pub bbox: Rect,
    pub dpi: f32,
    pub width: u32,
    pub height: u32,
    pub effective_dpi: Option<f32>,
    pub quality: QualityReport,
    pub local_only: bool,
}

/// Compute a normalized crop from a viewport bbox with a small padding.
pub fn crop_from_bbox(
    bbox: Rect,
    page_width: f32,
    page_height: f32,
    padding: f32,
) -> Result<CropBox> {
    let left = ((bbox.x - padding) / page_width).clamp(0.0, 1.0);
    let top = ((bbox.y - padding) / page_height).clamp(0.0, 1.0);
    let right = ((bbox.x + bbox.width + padding) / page_width).clamp(0.0, 1.0);
    let bottom = ((bbox.y + bbox.height + padding) / page_height).clamp(0.0, 1.0);
    if right <= left || bottom <= top {
        return Err(Error::PageSelection {
            value: "candidate bbox produces an empty crop".to_string(),
        });
    }
    CropBox::parse(&format!(
        "{},{},{},{}",
        left,
        top,
        right - left,
        bottom - top
    ))
}

/// Canonical extension for a native image format.
fn native_extension(format: &str) -> Result<&'static str> {
    if format.eq_ignore_ascii_case("jpg") || format.eq_ignore_ascii_case("jpeg") {
        Ok("jpg")
    } else if format.eq_ignore_ascii_case("png") {
        Ok("png")
    } else {
        Err(Error::PageSelection {
            value: format!("unsupported native image format {format:?}; expected png or jpeg"),
        })
    }
}

/// Resolve the requested output path and the format actually written.
///
/// An explicit `.png` or `.jpg` extension decides the encoding. A path with no
/// extension gets the native extension; any other extension is rejected.
fn requested_format(out: Option<&str>, native: &str) -> Result<(Option<String>, String)> {
    let Some(out) = out else {
        return Ok((None, native.to_string()));
    };
    let extension = Path::new(out)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    match extension.as_str() {
        "" => Ok((
            Some(format!("{}.{native}", out.trim_end_matches('.'))),
            native.to_string(),
        )),
        "png" => Ok((Some(out.to_string()), "png".to_string())),
        "jpg" | "jpeg" => Ok((Some(out.to_string()), "jpg".to_string())),
        other => Err(Error::PageSelection {
            value: format!("unsupported image extension .{other}; use .png, .jpg or .jpeg"),
        }),
    }
}

/// Encode decoded source bytes in the requested output format. Native bytes
/// are preserved unchanged when no conversion is needed.
fn encode_for_format(bytes: &[u8], native: &str, format: &str) -> Result<Vec<u8>> {
    if native == format {
        return Ok(bytes.to_vec());
    }
    let decoded = image::load_from_memory(bytes).map_err(|error| Error::Parse {
        path: PathBuf::from("<embedded image>"),
        message: format!("decode embedded {native} image: {error}"),
    })?;
    let mut encoded = Vec::new();
    match format {
        "png" => decoded
            .write_to(&mut std::io::Cursor::new(&mut encoded), ImageFormat::Png)
            .map_err(|error| Error::Parse {
                path: PathBuf::from("<embedded image>"),
                message: format!("encode png asset: {error}"),
            })?,
        "jpg" => {
            let rgb = decoded.to_rgb8();
            let mut cursor = std::io::Cursor::new(&mut encoded);
            let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, 92);
            encoder.encode_image(&rgb).map_err(|error| Error::Parse {
                path: PathBuf::from("<embedded image>"),
                message: format!("encode jpeg asset: {error}"),
            })?;
        }
        other => {
            return Err(Error::PageSelection {
                value: format!("unsupported output image format {other:?}; use png or jpg"),
            });
        }
    }
    Ok(encoded)
}

/// Extract one embedded raster image at its original pixel data.
pub async fn extract_native(
    source: &Path,
    guide_dir: &Path,
    page: u32,
    image_id: &str,
    out: Option<&str>,
) -> Result<NativeAsset> {
    pdfium::init(None)?;
    let source_text = source.to_str().ok_or_else(|| Error::Parse {
        path: source.to_path_buf(),
        message: "source path is not valid UTF-8".to_string(),
    })?;
    let result = LiteParse::new(LiteParseConfig {
        ocr_enabled: false,
        output_format: OutputFormat::Json,
        extract_images: true,
        target_pages: Some(page.to_string()),
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
    let image = result
        .images
        .iter()
        .find(|image| image.id == image_id)
        .ok_or_else(|| Error::Parse {
            path: source.to_path_buf(),
            message: format!("image {image_id:?} not found on page {page}"),
        })?;
    let bytes = image.bytes.as_slice();
    let native = native_extension(&image.format)?;
    let (out_path, format) = requested_format(out, native)?;
    let bytes = encode_for_format(bytes, native, &format)?;
    let hash = blake3::hash(&bytes).to_hex().to_string();
    let relative = out_path.unwrap_or_else(|| format!("data/figures/{hash}.{format}"));
    let target = safe_join(guide_dir, &relative)?;
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).map_err(|error| io(parent, error))?;
    }
    std::fs::write(&target, &bytes).map_err(|error| io(&target, error))?;
    let bbox = Rect::from(image.bbox.clone());
    let effective = effective_dpi(image.width, bbox.width);
    let quality = inspect(&bytes, effective)?;
    Ok(NativeAsset {
        page,
        image_id: image.id.clone(),
        asset: relative,
        hash,
        native_format: native.to_string(),
        format,
        width: image.width,
        height: image.height,
        bbox,
        effective_dpi: effective,
        quality,
        duplicate_of: image.duplicate_of.clone(),
        local_only: true,
    })
}

/// Render one page and crop a normalized region at an explicit DPI.
pub async fn render_crop(
    source: &Path,
    guide_dir: &Path,
    page: u32,
    crop: CropBox,
    bbox: Rect,
    dpi: f32,
    out: Option<&str>,
) -> Result<CropAsset> {
    pdfium::init(None)?;
    if !(MIN_CROP_DPI..=MAX_CROP_DPI).contains(&dpi) {
        return Err(Error::PageSelection {
            value: format!("crop dpi must be between {MIN_CROP_DPI} and {MAX_CROP_DPI}"),
        });
    }
    let source_text = source.to_str().ok_or_else(|| Error::Parse {
        path: source.to_path_buf(),
        message: "source path is not valid UTF-8".to_string(),
    })?;
    let parser = LiteParse::new(LiteParseConfig {
        ocr_enabled: false,
        quiet: true,
        dpi,
        ..Default::default()
    });
    let shots = parser
        .screenshot(source_text, Some(vec![page]))
        .await
        .map_err(|error| Error::Parse {
            path: source.to_path_buf(),
            message: error.to_string(),
        })?;
    let shot = shots.into_iter().next().ok_or_else(|| Error::Parse {
        path: source.to_path_buf(),
        message: format!("page {page} was not rendered"),
    })?;
    if u64::from(shot.width) * u64::from(shot.height) > MAX_RENDER_PIXELS {
        return Err(Error::PageSelection {
            value: format!(
                "page {} at {dpi} DPI exceeds the {} MP render guard",
                page,
                MAX_RENDER_PIXELS / 1_000_000
            ),
        });
    }
    let image = image::load_from_memory(&shot.image_bytes).map_err(|error| Error::Parse {
        path: PathBuf::from("<rendered page>"),
        message: format!("decode rendered page: {error}"),
    })?;
    let x = (image.width() as f32 * crop.left).round() as u32;
    let y = (image.height() as f32 * crop.top).round() as u32;
    let width = ((image.width() as f32 * crop.width).round() as u32).max(1);
    let height = ((image.height() as f32 * crop.height).round() as u32).max(1);
    let width = width.min(image.width().saturating_sub(x));
    let height = height.min(image.height().saturating_sub(y));
    let cropped = image::imageops::crop_imm(&image, x, y, width, height).to_image();
    let mut bytes = Vec::new();
    cropped
        .write_to(&mut std::io::Cursor::new(&mut bytes), ImageFormat::Png)
        .map_err(|error| Error::Parse {
            path: PathBuf::from("<cropped page>"),
            message: format!("encode cropped page: {error}"),
        })?;
    let hash = blake3::hash(&bytes).to_hex().to_string();
    let relative = out
        .map(ToString::to_string)
        .unwrap_or_else(|| format!("data/crops/{hash}.png"));
    let target = safe_join(guide_dir, &relative)?;
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).map_err(|error| io(parent, error))?;
    }
    std::fs::write(&target, &bytes).map_err(|error| io(&target, error))?;
    let effective = effective_dpi(cropped.width(), bbox.width);
    let quality = inspect(&bytes, effective)?;
    Ok(CropAsset {
        page,
        asset: relative,
        hash,
        crop: format!("{},{},{},{}", crop.left, crop.top, crop.width, crop.height),
        bbox,
        dpi,
        width: cropped.width(),
        height: cropped.height(),
        effective_dpi: effective,
        quality,
        local_only: true,
    })
}

/// Resolve a guide-relative asset path, rejecting traversal and absolute paths.
pub fn safe_join(guide_dir: &Path, relative: &str) -> Result<PathBuf> {
    let path = PathBuf::from(relative);
    if path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(Error::PageSelection {
            value: format!("asset path must be guide-relative: {relative:?}"),
        });
    }
    let root = guide_dir
        .canonicalize()
        .map_err(|error| io(guide_dir, error))?;
    let target = guide_dir.join(&path);
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).map_err(|error| io(parent, error))?;
    }
    let resolved = target.canonicalize().unwrap_or(target);
    if !resolved.starts_with(&root) {
        return Err(Error::PageSelection {
            value: format!("asset path escapes the guide: {relative:?}"),
        });
    }
    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crop_from_bbox_clamps_to_the_page_and_keeps_padding() {
        let bbox = Rect {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 50.0,
        };
        let crop = crop_from_bbox(bbox, 200.0, 100.0, 4.0).expect("crop");
        assert!(crop.left >= 0.0 && crop.top >= 0.0);
        assert!(crop.left + crop.width <= 1.0);
        assert!(crop.top + crop.height <= 1.0);
        assert!(crop.width > 0.5);
    }

    #[test]
    fn crop_from_bbox_rejects_an_empty_box() {
        let bbox = Rect {
            x: 200.0,
            y: 100.0,
            width: 0.0,
            height: 0.0,
        };
        assert!(crop_from_bbox(bbox, 200.0, 100.0, 0.0).is_err());
    }

    fn jpeg_bytes() -> Vec<u8> {
        let image = image::RgbImage::from_fn(48, 32, |x, y| {
            image::Rgb([(x * 5) as u8, (y * 7) as u8, 128])
        });
        let mut bytes = Vec::new();
        {
            let mut cursor = std::io::Cursor::new(&mut bytes);
            let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, 92);
            encoder.encode_image(&image).expect("jpeg");
        }
        bytes
    }

    #[test]
    fn png_target_reencodes_a_native_jpeg() {
        let native = jpeg_bytes();
        let (path, format) =
            requested_format(Some("data/figures/exhibit.png"), "jpg").expect("requested format");
        assert_eq!(path.as_deref(), Some("data/figures/exhibit.png"));
        assert_eq!(format, "png");
        let written = encode_for_format(&native, "jpg", &format).expect("encode");
        assert_eq!(&written[..8], b"\x89PNG\r\n\x1a\n");
        assert_ne!(written, native);
        assert_eq!(
            blake3::hash(&written).to_hex().to_string(),
            blake3::hash(&written).to_hex().to_string()
        );
        inspect(&written, Some(200.0)).expect("quality");
    }

    #[test]
    fn jpg_target_keeps_native_jpeg_bytes() {
        let native = jpeg_bytes();
        let (path, format) =
            requested_format(Some("data/figures/exhibit.jpg"), "jpg").expect("requested format");
        assert_eq!(path.as_deref(), Some("data/figures/exhibit.jpg"));
        assert_eq!(format, "jpg");
        let written = encode_for_format(&native, "jpg", &format).expect("encode");
        assert_eq!(written, native);
        inspect(&written, Some(200.0)).expect("quality");
    }

    #[test]
    fn missing_extension_uses_the_native_extension() {
        let (path, format) =
            requested_format(Some("data/figures/exhibit"), "jpg").expect("requested format");
        assert_eq!(path.as_deref(), Some("data/figures/exhibit.jpg"));
        assert_eq!(format, "jpg");
        let (path, format) = requested_format(None, "png").expect("default requested format");
        assert!(path.is_none());
        assert_eq!(format, "png");
    }

    #[test]
    fn unsupported_extensions_are_named() {
        let error = requested_format(Some("data/figures/exhibit.webp"), "png")
            .expect_err("webp should fail");
        assert!(error.to_string().contains(".webp"));
        assert!(error.to_string().contains(".png"));
    }
}
