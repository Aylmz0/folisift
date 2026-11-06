//! Folisift-owned page layout model.
//!
//! LiteParse exposes a flat, serializable block shape. This module maps it to
//! a stable Folisift schema so a parser upgrade cannot silently change the
//! inventory or the document-link artifact.

use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result, io};

pub const LAYOUT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl From<liteparse::types::Rect> for Rect {
    fn from(value: liteparse::types::Rect) -> Self {
        Self {
            x: value.x,
            y: value.y,
            width: value.width,
            height: value.height,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayoutBlockKind {
    Heading,
    Paragraph,
    ListItem,
    Code,
    Table,
    MergedTable,
    GridFallback,
    Rule,
    Figure,
    Unknown,
}

impl LayoutBlockKind {
    pub fn from_liteparse(kind: &str) -> Self {
        match kind {
            "heading" => Self::Heading,
            "paragraph" => Self::Paragraph,
            "list_item" => Self::ListItem,
            "code" => Self::Code,
            "table" => Self::Table,
            "merged_table" => Self::MergedTable,
            "grid_fallback" => Self::GridFallback,
            "rule" => Self::Rule,
            "figure" => Self::Figure,
            _ => Self::Unknown,
        }
    }

    pub fn is_table(self) -> bool {
        matches!(self, Self::Table | Self::MergedTable)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutCell {
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bbox: Option<Rect>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub colspan: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rowspan: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutBlock {
    pub id: String,
    pub kind: LayoutBlockKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bbox: Option<Rect>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub level: Option<u8>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub bold: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub italic: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ordered: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lines: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub header: Vec<LayoutCell>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rows: Vec<Vec<LayoutCell>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub header_rows: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub figure_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub figure_format: Option<String>,
}

impl LayoutBlock {
    pub fn from_liteparse(id: String, block: &liteparse::layout::LayoutBlock) -> Self {
        let cells = |cells: &[liteparse::layout::LayoutCell]| {
            cells
                .iter()
                .map(|cell| LayoutCell {
                    text: cell.text.clone(),
                    bbox: cell.bbox.clone().map(Rect::from),
                    colspan: cell.colspan,
                    rowspan: cell.rowspan,
                })
                .collect::<Vec<_>>()
        };
        Self {
            id,
            kind: LayoutBlockKind::from_liteparse(&block.kind),
            bbox: block.bbox.clone().map(Rect::from),
            text: block.text.clone(),
            level: block.level,
            bold: block.bold,
            italic: block.italic,
            ordered: block.ordered,
            marker: block.marker.clone(),
            lines: block.lines.clone().unwrap_or_default(),
            lang: block.lang.clone(),
            header: block.header.as_deref().map(cells).unwrap_or_default(),
            rows: block
                .rows
                .as_deref()
                .map(|rows| rows.iter().map(|row| cells(row)).collect())
                .unwrap_or_default(),
            header_rows: block.header_rows,
            figure_id: block.id.clone(),
            figure_format: block.format.clone(),
        }
    }

    pub fn text_or_empty(&self) -> &str {
        self.text.as_deref().unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageLayout {
    pub page: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_label: Option<String>,
    pub width: f32,
    pub height: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_bounds: Option<Rect>,
    #[serde(default)]
    pub blocks: Vec<LayoutBlock>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceLayout {
    pub schema_version: u32,
    pub source: String,
    pub page_count: usize,
    pub pages: Vec<PageLayout>,
}

impl SourceLayout {
    pub fn new(source: impl Into<String>, page_count: usize, pages: Vec<PageLayout>) -> Self {
        Self {
            schema_version: LAYOUT_SCHEMA_VERSION,
            source: source.into(),
            page_count,
            pages,
        }
    }

    pub fn summary(&self) -> LayoutSummary {
        let pages = self
            .pages
            .iter()
            .map(|page| PageLayoutSummary {
                page: page.page,
                blocks: page.blocks.len(),
                headings: page
                    .blocks
                    .iter()
                    .filter(|block| block.kind == LayoutBlockKind::Heading)
                    .count(),
                tables: page
                    .blocks
                    .iter()
                    .filter(|block| block.kind.is_table())
                    .count(),
                figures: page
                    .blocks
                    .iter()
                    .filter(|block| block.kind == LayoutBlockKind::Figure)
                    .count(),
                content_bounds: page.content_bounds,
            })
            .collect::<Vec<_>>();
        LayoutSummary {
            schema_version: self.schema_version,
            pages_with_blocks: pages.iter().filter(|page| page.blocks > 0).count(),
            blocks: pages.iter().map(|page| page.blocks).sum(),
            headings: pages.iter().map(|page| page.headings).sum(),
            tables: pages.iter().map(|page| page.tables).sum(),
            figures: pages.iter().map(|page| page.figures).sum(),
            pages,
        }
    }

    pub fn write_json(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| io(parent, error))?;
        }
        let json = serde_json::to_string_pretty(self).map_err(|error| Error::Parse {
            path: path.to_path_buf(),
            message: error.to_string(),
        })?;
        std::fs::write(path, format!("{json}\n")).map_err(|error| io(path, error))
    }

    pub fn read_json(path: &Path) -> Result<Self> {
        let bytes = std::fs::read(path).map_err(|error| io(path, error))?;
        serde_json::from_slice(&bytes).map_err(|error| Error::Parse {
            path: path.to_path_buf(),
            message: error.to_string(),
        })
    }
}

/// Write one layout artifact per source under `dir`.
pub fn write_layouts(layouts: &[SourceLayout], dir: &Path) -> Result<Vec<std::path::PathBuf>> {
    std::fs::create_dir_all(dir).map_err(|error| io(dir, error))?;
    let mut paths = Vec::with_capacity(layouts.len());
    for layout in layouts {
        let path = dir.join(format!("{}.json", layout.source));
        layout.write_json(&path)?;
        paths.push(path);
    }
    Ok(paths)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockView {
    pub schema_version: u32,
    pub source: String,
    pub page_count: usize,
    pub pages: Vec<PageLayout>,
    pub total_blocks: usize,
    pub returned_blocks: usize,
    pub truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutSummary {
    pub schema_version: u32,
    pub pages_with_blocks: usize,
    pub blocks: usize,
    pub headings: usize,
    pub tables: usize,
    pub figures: usize,
    pub pages: Vec<PageLayoutSummary>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageLayoutSummary {
    pub page: usize,
    pub blocks: usize,
    pub headings: usize,
    pub tables: usize,
    pub figures: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_bounds: Option<Rect>,
}

/// Read a bounded page/block view from a stored layout artifact.
/// Normalize a source argument to the block-file stem.
///
/// Accepted forms: `PD0`, `PD0.pdf`, `sources/PD0.pdf` and
/// `data/blocks/PD0.json`. Paths that escape the workspace are rejected.
pub fn source_id(value: &str) -> Result<String> {
    let raw = value.trim();
    let path = Path::new(raw);
    if raw.is_empty()
        || path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(Error::Parse {
            path: PathBuf::from(raw),
            message: "invalid source id; use PD0, PD0.pdf, sources/PD0.pdf or data/blocks/PD0.json"
                .to_string(),
        });
    }
    let file = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| Error::Parse {
            path: PathBuf::from(raw),
            message: "source id has no file name".to_string(),
        })?;
    let mut stem = file;
    for extension in [".pdf.json", ".pdf", ".json"] {
        if let Some(value) = stem.strip_suffix(extension) {
            stem = value;
            break;
        }
    }
    if stem.is_empty() || stem.contains('/') || stem.contains('\\') || stem == "." || stem == ".." {
        return Err(Error::Parse {
            path: PathBuf::from(raw),
            message: "invalid source id; use the PDF file stem".to_string(),
        });
    }
    Ok(stem.to_string())
}

/// Resolve one source argument to `<guide>/data/blocks/<id>.json`.
pub fn block_path(guide_dir: &Path, source: &str) -> Result<PathBuf> {
    let id = source_id(source)?;
    Ok(guide_dir.join("data/blocks").join(format!("{id}.json")))
}

pub fn read_block_view(
    blocks_dir: &Path,
    source: &str,
    pages: Option<&[u32]>,
    max_blocks: usize,
) -> Result<BlockView> {
    let source = source_id(source)?;
    let layout = SourceLayout::read_json(&blocks_dir.join(format!("{source}.json")))?;
    let total_blocks = layout.pages.iter().map(|page| page.blocks.len()).sum();
    let mut returned_blocks = 0usize;
    let mut truncated = false;
    let mut selected = Vec::new();
    for page in &layout.pages {
        if pages.is_some_and(|selected_pages| !selected_pages.contains(&(page.page as u32))) {
            continue;
        }
        let mut page = page.clone();
        if returned_blocks + page.blocks.len() > max_blocks {
            let remaining = max_blocks.saturating_sub(returned_blocks);
            page.blocks.truncate(remaining);
            truncated = true;
        }
        returned_blocks += page.blocks.len();
        selected.push(page);
        if truncated {
            break;
        }
    }
    Ok(BlockView {
        schema_version: layout.schema_version,
        source: layout.source,
        page_count: layout.page_count,
        pages: selected,
        total_blocks,
        returned_blocks,
        truncated,
    })
}

fn is_false(value: &bool) -> bool {
    !value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_source_arguments() {
        for value in ["PD0", "PD0.pdf", "sources/PD0.pdf", "data/blocks/PD0.json"] {
            assert_eq!(source_id(value).expect(value), "PD0");
        }
        assert!(source_id("").is_err());
        assert!(source_id("../PD0.pdf").is_err());
        assert!(source_id("/tmp/PD0.pdf").is_err());
        assert!(source_id("sources/..").is_err());
    }

    #[test]
    fn maps_layout_cells_and_counts() {
        let page = PageLayout {
            page: 1,
            page_label: None,
            width: 600.0,
            height: 800.0,
            content_bounds: Some(Rect {
                x: 10.0,
                y: 20.0,
                width: 300.0,
                height: 400.0,
            }),
            blocks: vec![
                LayoutBlock {
                    id: "SRC:p1:b0".to_string(),
                    kind: LayoutBlockKind::Heading,
                    bbox: None,
                    text: Some("Title".to_string()),
                    level: Some(1),
                    bold: false,
                    italic: false,
                    ordered: None,
                    marker: None,
                    lines: vec![],
                    lang: None,
                    header: vec![],
                    rows: vec![],
                    header_rows: None,
                    figure_id: None,
                    figure_format: None,
                },
                LayoutBlock {
                    id: "SRC:p1:b1".to_string(),
                    kind: LayoutBlockKind::Table,
                    bbox: None,
                    text: None,
                    level: None,
                    bold: false,
                    italic: false,
                    ordered: None,
                    marker: None,
                    lines: vec![],
                    lang: None,
                    header: vec![LayoutCell {
                        text: "A".to_string(),
                        bbox: None,
                        colspan: None,
                        rowspan: None,
                    }],
                    rows: vec![],
                    header_rows: None,
                    figure_id: None,
                    figure_format: None,
                },
            ],
        };
        let layout = SourceLayout::new("SRC", 1, vec![page]);
        let summary = layout.summary();
        assert_eq!(summary.blocks, 2);
        assert_eq!(summary.headings, 1);
        assert_eq!(summary.tables, 1);
        assert_eq!(summary.pages_with_blocks, 1);
    }

    #[test]
    fn reads_a_bounded_block_view() {
        let page = PageLayout {
            page: 1,
            page_label: None,
            width: 600.0,
            height: 800.0,
            content_bounds: None,
            blocks: (0..3)
                .map(|index| LayoutBlock {
                    id: format!("SRC:p1:b{index}"),
                    kind: LayoutBlockKind::Paragraph,
                    bbox: None,
                    text: Some(format!("block {index}")),
                    level: None,
                    bold: false,
                    italic: false,
                    ordered: None,
                    marker: None,
                    lines: vec![],
                    lang: None,
                    header: vec![],
                    rows: vec![],
                    header_rows: None,
                    figure_id: None,
                    figure_format: None,
                })
                .collect(),
        };
        let layout = SourceLayout::new("SRC", 1, vec![page]);
        let dir = std::env::temp_dir().join(format!("folisift-block-view-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        write_layouts(std::slice::from_ref(&layout), &dir).expect("write");
        let view = read_block_view(&dir, "SRC", Some(&[1]), 2).expect("view");
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(view.returned_blocks, 2);
        assert_eq!(view.total_blocks, 3);
        assert!(view.truncated);
        assert_eq!(view.pages[0].blocks.len(), 2);
    }

    #[test]
    fn round_trips_json() {
        let layout = SourceLayout::new("SRC", 1, vec![]);
        let path =
            std::env::temp_dir().join(format!("folisift-layout-{}.json", std::process::id()));
        layout.write_json(&path).expect("write");
        let read = SourceLayout::read_json(&path).expect("read");
        let _ = std::fs::remove_file(path);
        assert_eq!(read, layout);
    }
}
