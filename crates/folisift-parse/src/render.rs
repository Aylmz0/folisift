use std::path::{Path, PathBuf};

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

/// Render selected pages (or all pages when `pages` is `None`) to PNG files.
pub async fn render_pages(
    source: &Path,
    pages: Option<Vec<u32>>,
    out_dir: &Path,
) -> Result<Vec<PageImage>> {
    pdfium::init(None)?;
    let parser = LiteParse::new(LiteParseConfig {
        ocr_enabled: false,
        quiet: true,
        dpi: 150.0,
        ..Default::default()
    });
    let source_text = source.to_str().ok_or_else(|| Error::Parse {
        path: source.to_path_buf(),
        message: "source path is not valid UTF-8".to_string(),
    })?;
    let shots = parser
        .screenshot(source_text, pages)
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
    Ok(images)
}
