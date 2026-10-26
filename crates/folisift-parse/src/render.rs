use std::path::{Path, PathBuf};

use image::{ImageBuffer, Rgba, RgbaImage};
use liteparse::{LiteParse, LiteParseConfig};
use serde::Serialize;

use crate::error::{Error, Result, io};
use crate::pdfium;

/// A rendered page image on disk.
#[derive(Debug, Clone, Serialize)]
pub struct PageImage {
    pub page: u32,
    pub width: u32,
    pub height: u32,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
pub struct RenderPagesResult {
    pub pages: Vec<PageImage>,
    pub returned: usize,
    pub requested: usize,
    pub truncated: bool,
    pub next_pages: Option<String>,
    pub contact_sheet: Option<PathBuf>,
}

pub const MAX_PAGES_PER_CALL: usize = 8;

/// Render selected pages to PNG files.
///
/// At most [`MAX_PAGES_PER_CALL`] pages are rendered per call. A wider request
/// returns a prefix plus `next_pages`; it never silently drops pages.
pub async fn render_pages(
    source: &Path,
    pages: Option<Vec<u32>>,
    out_dir: &Path,
    contact_sheet: bool,
    dpi: Option<f32>,
) -> Result<RenderPagesResult> {
    pdfium::init(None)?;
    let requested = pages.unwrap_or_else(|| (1..=MAX_PAGES_PER_CALL as u32).collect());
    if requested.is_empty() {
        return Err(Error::PageSelection {
            value: "empty page selection".to_string(),
        });
    }
    let truncated = requested.len() > MAX_PAGES_PER_CALL;
    let selected = requested
        .iter()
        .take(MAX_PAGES_PER_CALL)
        .copied()
        .collect::<Vec<_>>();
    let next_pages = if truncated {
        Some(
            requested[MAX_PAGES_PER_CALL..]
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(","),
        )
    } else {
        None
    };

    let parser = LiteParse::new(LiteParseConfig {
        ocr_enabled: false,
        quiet: true,
        dpi: dpi.unwrap_or(150.0).clamp(72.0, 200.0),
        ..Default::default()
    });
    let source_text = source.to_str().ok_or_else(|| Error::Parse {
        path: source.to_path_buf(),
        message: "source path is not valid UTF-8".to_string(),
    })?;
    let shots = parser
        .screenshot(source_text, Some(selected))
        .await
        .map_err(|error| Error::Parse {
            path: source.to_path_buf(),
            message: error.to_string(),
        })?;

    std::fs::create_dir_all(out_dir).map_err(|error| io(out_dir, error))?;
    let mut images = Vec::with_capacity(shots.len());
    for shot in shots {
        let path = out_dir.join(format!("page-{:03}.png", shot.page_num));
        std::fs::write(&path, &shot.image_bytes).map_err(|error| io(&path, error))?;
        images.push(PageImage {
            page: shot.page_num,
            width: shot.width,
            height: shot.height,
            path,
        });
    }

    let contact_sheet = if contact_sheet && !images.is_empty() {
        Some(write_contact_sheet(out_dir, &images)?)
    } else {
        None
    };

    Ok(RenderPagesResult {
        returned: images.len(),
        requested: requested.len(),
        pages: images,
        truncated,
        next_pages,
        contact_sheet,
    })
}

fn write_contact_sheet(out_dir: &Path, images: &[PageImage]) -> Result<PathBuf> {
    const THUMB_WIDTH: u32 = 600;
    const GAP: u32 = 8;
    let columns = 2u32;
    let rows = (images.len() as u32).div_ceil(columns);
    let mut thumbs = Vec::with_capacity(images.len());
    for image in images {
        let source = image::open(&image.path).map_err(|error| Error::Parse {
            path: image.path.clone(),
            message: format!("contact sheet source: {error}"),
        })?;
        let ratio = THUMB_WIDTH as f32 / source.width().max(1) as f32;
        let height = (source.height() as f32 * ratio).round().max(1.0) as u32;
        thumbs.push(source.resize(THUMB_WIDTH, height, image::imageops::FilterType::Lanczos3));
    }
    let thumb_height = thumbs.iter().map(|thumb| thumb.height()).max().unwrap_or(1);
    let width = columns * THUMB_WIDTH + (columns + 1) * GAP;
    let height = rows * thumb_height + (rows + 1) * GAP;
    let mut sheet: RgbaImage = ImageBuffer::from_pixel(width, height, Rgba([250, 246, 236, 255]));
    for (index, thumb) in thumbs.iter().enumerate() {
        let column = index as u32 % columns;
        let row = index as u32 / columns;
        let x = GAP + column * (THUMB_WIDTH + GAP);
        let y = GAP + row * (thumb_height + GAP);
        image::imageops::overlay(&mut sheet, thumb, x as i64, y as i64);
    }
    let path = out_dir.join("contact-sheet.png");
    sheet.save(&path).map_err(|error| Error::Parse {
        path: path.clone(),
        message: format!("contact sheet: {error}"),
    })?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_cap_is_consistent() {
        assert_eq!(MAX_PAGES_PER_CALL, 8);
    }
}
