//! Local, review-only semantic links between layout-block sections.

mod link;
mod model;
mod section;
mod store;

use std::path::{Path, PathBuf};

use link::Result;

pub use link::{Embedder, Link, LinkIndex, LinkOptions, LinkReport};

pub const DEFAULT_MODEL: &str = "potion-multilingual-128M-int8";

/// Read every stored layout artifact in a directory, sorted by source.
pub fn read_layouts(dir: &Path) -> Result<Vec<folisift_parse::SourceLayout>> {
    let mut paths = Vec::new();
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) == Some("json") {
            paths.push(path);
        }
    }
    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            folisift_parse::SourceLayout::read_json(&path)
                .map_err(|error| link::Error::Parse(error.to_string()))
        })
        .collect()
}

/// Resolve the bundled model directory without any network access.
pub fn find_model_dir(workspace_root: &Path) -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("FOLISIFT_MODEL_DIR") {
        let path = PathBuf::from(path);
        if path.is_dir() {
            return Some(path);
        }
    }
    for candidate in [
        workspace_root.join("data/models").join(DEFAULT_MODEL),
        workspace_root.join("models").join(DEFAULT_MODEL),
    ] {
        if candidate.is_dir() {
            return Some(candidate);
        }
    }
    let sidecar = std::env::current_exe().ok().and_then(|path| {
        path.parent()
            .map(|parent| parent.join("models").join(DEFAULT_MODEL))
    });
    sidecar.filter(|path| path.is_dir())
}
pub use model::Model2VecEmbedder;
pub use section::{Section, sections_from_layout};
pub use store::{LINKS_SCHEMA_VERSION, read_report, write_markdown_report, write_report};
