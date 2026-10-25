use std::fs;
use std::path::PathBuf;

use crate::config::{Guide, GuideConfig};
use crate::error::{Error, Result, io};
use crate::paths::{Workspace, validate_slug};

#[derive(Debug, Clone)]
pub struct ScaffoldOptions {
    pub slug: String,
    pub title: String,
    pub course: String,
    pub sources: Vec<PathBuf>,
}

/// Create a new schema-2 guide bundle and return its validated config.
pub fn scaffold(workspace: &Workspace, options: &ScaffoldOptions) -> Result<Guide> {
    validate_slug(&options.slug)?;
    if options.title.trim().is_empty() {
        return Err(Error::Config {
            path: workspace.root().join("guides"),
            message: "title must not be empty".to_string(),
        });
    }
    if options.sources.is_empty() {
        return Err(Error::Config {
            path: workspace.root().join("guides"),
            message: "at least one source is required".to_string(),
        });
    }

    let guide_dir = workspace.root().join("guides").join(&options.slug);
    let guide_dir = workspace.resolve_write(&guide_dir)?;
    if guide_dir.exists() {
        return Err(Error::Config {
            path: guide_dir,
            message: format!("guide {:?} already exists", options.slug),
        });
    }

    let sources_root = workspace.root().join("sources");
    let mut relative_sources = Vec::with_capacity(options.sources.len());
    for source in &options.sources {
        let source = workspace.resolve_read(source)?;
        if !source.starts_with(&sources_root) {
            return Err(Error::Config {
                path: source,
                message: "sources must live under sources/".to_string(),
            });
        }
        let relative = pathdiff::diff_paths(&source, &guide_dir).ok_or_else(|| Error::Config {
            path: source.clone(),
            message: "could not make the source path relative to the guide".to_string(),
        })?;
        relative_sources.push(relative.display().to_string());
    }

    for directory in [guide_dir.join("assets"), guide_dir.join("data")] {
        fs::create_dir_all(&directory).map_err(|error| io(&directory, error))?;
    }

    let output = format!("../../output/{}.pdf", options.slug);
    let config = GuideConfig {
        schema_version: 2,
        slug: options.slug.clone(),
        title: options.title.clone(),
        language: "tr".to_string(),
        content: "guide.md".to_string(),
        stylesheet: None,
        output,
        sources: relative_sources,
        inventory: Some("data/inventory.json".to_string()),
        coverage: Some("data/coverage.md".to_string()),
        target_pages: None,
        theme: "paper".to_string(),
        extra: Default::default(),
    };
    let config_path = guide_dir.join("config.json");
    fs::write(
        &config_path,
        serde_json::to_string_pretty(&config).map_err(|error| Error::Config {
            path: config_path.clone(),
            message: error.to_string(),
        })? + "\n",
    )
    .map_err(|error| io(&config_path, error))?;

    let guide_content = format!(
        "---\ntitle: {title:?}\ncourse: {course:?}\nlanguage: tr\nsources: []\n---\n\n# Bölüm 1\n\n## 1.1 Giriş\n\nKaynak içeriği buraya gelecek. [PD0, s. 1]\n",
        title = options.title,
        course = options.course,
    );
    let content_path = guide_dir.join("guide.md");
    fs::write(&content_path, guide_content).map_err(|error| io(&content_path, error))?;

    Guide::load(workspace, &options.slug)
}

/// A compact, pure helper used by tests and callers that only need the paths.
pub fn guide_paths(workspace: &Workspace, slug: &str) -> Result<(PathBuf, PathBuf)> {
    validate_slug(slug)?;
    let dir = workspace.guide_dir(slug)?;
    Ok((dir.join("config.json"), dir.join("guide.md")))
}
