//! Visual comparison of two PDFs with PDFium-rendered pages.

use std::path::Path;

use image::{ImageBuffer, Rgba};
use serde::Serialize;

use crate::error::{Error, Result, io};
use crate::pdfium;

#[derive(Debug, Clone, Serialize)]
pub struct VisualPage {
    pub page: usize,
    pub mean_difference: f64,
    pub changed_ratio: f64,
    pub diff_image: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VisualReport {
    pub reference: String,
    pub current: String,
    pub pages: Vec<VisualPage>,
    pub passed: bool,
    pub threshold: f64,
    pub ratio_threshold: f64,
    pub issues: Vec<String>,
}

pub fn compare_pdfs(
    reference: &Path,
    current: &Path,
    pages: &[usize],
    dpi: f32,
    out_dir: &Path,
    threshold: f64,
    ratio_threshold: f64,
) -> Result<VisualReport> {
    pdfium::init(None)?;
    let library = liteparse_pdfium::Library::try_init().map_err(|error| Error::Pdfium {
        message: error.to_string(),
    })?;
    let reference_doc = library
        .load_document(reference.to_str().unwrap_or_default(), None)
        .map_err(|error| Error::Parse {
            path: reference.to_path_buf(),
            message: error.to_string(),
        })?;
    let current_doc = library
        .load_document(current.to_str().unwrap_or_default(), None)
        .map_err(|error| Error::Parse {
            path: current.to_path_buf(),
            message: error.to_string(),
        })?;
    std::fs::create_dir_all(out_dir).map_err(|error| io(out_dir, error))?;
    let mut reports = Vec::new();
    for page_number in pages {
        let index =
            i32::try_from(page_number.saturating_sub(1)).map_err(|_| Error::PageSelection {
                value: page_number.to_string(),
            })?;
        if index >= reference_doc.page_count() || index >= current_doc.page_count() {
            reports.push(VisualPage {
                page: *page_number,
                mean_difference: 100.0,
                changed_ratio: 1.0,
                diff_image: None,
            });
            continue;
        }
        let ref_page = reference_doc.page(index).map_err(|error| Error::Parse {
            path: reference.to_path_buf(),
            message: error.to_string(),
        })?;
        let cur_page = current_doc.page(index).map_err(|error| Error::Parse {
            path: current.to_path_buf(),
            message: error.to_string(),
        })?;
        let ref_bitmap = ref_page.render(dpi).map_err(|error| Error::Parse {
            path: reference.to_path_buf(),
            message: error.to_string(),
        })?;
        let cur_bitmap = cur_page.render(dpi).map_err(|error| Error::Parse {
            path: current.to_path_buf(),
            message: error.to_string(),
        })?;
        let ref_image = RgbaImageWrap::from_bitmap(&ref_bitmap);
        let cur_image = RgbaImageWrap::from_bitmap(&cur_bitmap);
        let mut total = 0f64;
        let mut changed = 0usize;
        let width = ref_image.width().min(cur_image.width());
        let height = ref_image.height().min(cur_image.height());
        let mut diff: ImageBuffer<Rgba<u8>, Vec<u8>> =
            ImageBuffer::from_pixel(width, height, Rgba([250u8, 246, 236, 255]));
        for y in 0..height {
            for x in 0..width {
                let a = ref_image.get_pixel(x, y).0;
                let b = cur_image.get_pixel(x, y).0;
                let delta = (0..3)
                    .map(|channel| (i32::from(a[channel]) - i32::from(b[channel])).abs())
                    .sum::<i32>() as f64
                    / 3.0;
                total += delta;
                if delta > 30.0 {
                    changed += 1;
                    diff.put_pixel(x, y, Rgba([185u8, 62, 47, 180]));
                }
            }
        }
        let count = (width as f64 * height as f64).max(1.0);
        let mean = total / count;
        let ratio = changed as f64 / count;
        let path = out_dir.join(format!("diff-{page_number:03}.png"));
        diff.save(&path).map_err(|error| Error::Parse {
            path: path.clone(),
            message: format!("visual diff: {error}"),
        })?;
        reports.push(VisualPage {
            page: *page_number,
            mean_difference: (mean * 100.0).round() / 100.0,
            changed_ratio: (ratio * 10_000.0).round() / 10_000.0,
            diff_image: Some(path.display().to_string()),
        });
    }
    let issues = reports
        .iter()
        .filter(|page| page.mean_difference > threshold || page.changed_ratio > ratio_threshold)
        .map(|page| {
            format!(
                "page {} differs: mean {:.2} (limit {:.2}), changed {:.2}% (limit {:.2}%)",
                page.page,
                page.mean_difference,
                threshold,
                page.changed_ratio * 100.0,
                ratio_threshold * 100.0
            )
        })
        .collect::<Vec<_>>();
    Ok(VisualReport {
        reference: reference.display().to_string(),
        current: current.display().to_string(),
        pages: reports,
        passed: issues.is_empty(),
        threshold,
        ratio_threshold,
        issues,
    })
}

struct RgbaImageWrap(image::RgbaImage);

impl RgbaImageWrap {
    fn from_bitmap(bitmap: &liteparse_pdfium::Bitmap<'_>) -> Self {
        let width = bitmap.width().max(1) as u32;
        let height = bitmap.height().max(1) as u32;
        let rgba = bitmap.to_rgba();
        Self(
            ImageBuffer::from_raw(width, height, rgba).unwrap_or_else(|| {
                ImageBuffer::from_pixel(width, height, Rgba([255, 255, 255, 255]))
            }),
        )
    }

    fn width(&self) -> u32 {
        self.0.width()
    }

    fn height(&self) -> u32 {
        self.0.height()
    }

    fn get_pixel(&self, x: u32, y: u32) -> &Rgba<u8> {
        self.0.get_pixel(x, y)
    }
}
