use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{Error, Result, io};
use crate::paths::{Workspace, validate_slug};

pub const SUPPORTED_SCHEMA_VERSIONS: [u32; 2] = [1, 2];

/// One guide's `config.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuideConfig {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    pub slug: String,
    pub title: String,
    #[serde(default = "default_language")]
    pub language: String,
    pub content: String,
    #[serde(default)]
    pub stylesheet: Option<String>,
    pub output: String,
    #[serde(default)]
    pub sources: Vec<String>,
    #[serde(default)]
    pub inventory: Option<String>,
    #[serde(default)]
    pub coverage: Option<String>,
    #[serde(default)]
    pub target_pages: Option<String>,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// A validated guide bundle.
#[derive(Debug, Clone)]
pub struct Guide {
    pub slug: String,
    pub dir: PathBuf,
    pub config: GuideConfig,
    pub content: PathBuf,
    pub output: PathBuf,
    pub sources: Vec<PathBuf>,
    pub inventory: Option<PathBuf>,
    pub coverage: Option<PathBuf>,
}

/// A lightweight guide row for listings and status output.
#[derive(Debug, Clone, Serialize)]
pub struct GuideSummary {
    pub slug: String,
    pub title: String,
    pub schema_version: u32,
    pub output: String,
    pub sources: usize,
    pub content_words: usize,
    pub section_count: usize,
    pub valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Guide {
    /// Load and validate `guides/<slug>/config.json`.
    pub fn load(workspace: &Workspace, slug: &str) -> Result<Self> {
        validate_slug(slug)?;
        let dir = workspace.guide_dir(slug)?;
        let config_path = dir.join("config.json");
        let bytes = fs::read(&config_path).map_err(|error| io(&config_path, error))?;
        let config: GuideConfig = serde_json::from_slice(&bytes).map_err(|source| Error::Json {
            path: config_path.clone(),
            source,
        })?;

        let guide = Self::validate(workspace, dir, config)?;
        if guide.slug != slug {
            return Err(Error::Config {
                path: config_path,
                message: format!(
                    "config slug {:?} does not match directory {:?}",
                    guide.slug, slug
                ),
            });
        }
        Ok(guide)
    }

    fn validate(workspace: &Workspace, dir: PathBuf, config: GuideConfig) -> Result<Self> {
        let config_path = dir.join("config.json");

        if !SUPPORTED_SCHEMA_VERSIONS.contains(&config.schema_version) {
            return Err(Error::UnsupportedSchema {
                version: config.schema_version,
            });
        }
        validate_slug(&config.slug)?;
        if config.title.trim().is_empty() {
            return Err(config_error(&config_path, "title must not be empty"));
        }
        if config.language.trim().is_empty() {
            return Err(config_error(&config_path, "language must not be empty"));
        }
        if config.sources.is_empty() {
            return Err(config_error(
                &config_path,
                "at least one source is required",
            ));
        }
        if let Some(range) = &config.target_pages {
            validate_target_pages(range)?;
        }

        let content = resolve_guide_path(workspace, &dir, &config.content, PathKind::Read)?;
        if !content.starts_with(&dir) {
            return Err(config_error(
                &config_path,
                "content must stay inside the guide bundle",
            ));
        }
        require_extension(
            &config_path,
            &content,
            expected_content_extension(config.schema_version),
        )?;

        let output = resolve_guide_path(workspace, &dir, &config.output, PathKind::Write)?;
        require_extension(&config_path, &output, "pdf")?;

        let mut sources = Vec::with_capacity(config.sources.len());
        for source in &config.sources {
            let path = resolve_guide_path(workspace, &dir, source, PathKind::Read)?;
            if !path.starts_with(workspace.root().join("sources")) {
                return Err(config_error(
                    &config_path,
                    format!("source {source:?} must live under sources/"),
                ));
            }
            sources.push(path);
        }

        let inventory = resolve_optional(
            workspace,
            &dir,
            config.inventory.as_deref(),
            PathKind::Write,
        )?;
        let coverage =
            resolve_optional(workspace, &dir, config.coverage.as_deref(), PathKind::Write)?;

        if let Some(stylesheet) = &config.stylesheet {
            // Legacy schema-1 compatibility: a stylesheet may live outside the bundle.
            let _ = resolve_guide_path(workspace, &dir, stylesheet, PathKind::Read)?;
        }

        Ok(Self {
            slug: config.slug.clone(),
            dir,
            config,
            content,
            output,
            sources,
            inventory,
            coverage,
        })
    }
}

/// List every directory under `guides/` that has a config, including invalid ones.
pub fn list_guides(workspace: &Workspace) -> Result<Vec<GuideSummary>> {
    let guides_dir = workspace.root().join("guides");
    if !guides_dir.exists() {
        return Ok(Vec::new());
    }
    let entries = fs::read_dir(&guides_dir).map_err(|error| io(&guides_dir, error))?;
    let mut summaries = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| io(&guides_dir, error))?;
        let path = entry.path();
        if !path.is_dir() || !path.join("config.json").is_file() {
            continue;
        }
        let Some(slug) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        match Guide::load(workspace, slug) {
            Ok(guide) => {
                let content = fs::read_to_string(&guide.content).unwrap_or_default();
                summaries.push(GuideSummary {
                    slug: guide.slug,
                    title: guide.config.title,
                    schema_version: guide.config.schema_version,
                    output: guide.config.output,
                    sources: guide.sources.len(),
                    content_words: content.split_whitespace().count(),
                    section_count: content
                        .lines()
                        .filter(|line| line.trim_start().starts_with("## "))
                        .count(),
                    valid: true,
                    error: None,
                });
            }
            Err(error) => {
                let title = read_title(&path.join("config.json")).unwrap_or_default();
                summaries.push(GuideSummary {
                    slug: slug.to_string(),
                    title,
                    schema_version: 0,
                    output: String::new(),
                    sources: 0,
                    content_words: 0,
                    section_count: 0,
                    valid: false,
                    error: Some(error.to_string()),
                });
            }
        }
    }
    summaries.sort_by(|left, right| left.slug.cmp(&right.slug));
    Ok(summaries)
}

#[derive(Debug, Clone, Copy)]
enum PathKind {
    Read,
    Write,
}

fn resolve_guide_path(
    workspace: &Workspace,
    dir: &Path,
    raw: &str,
    kind: PathKind,
) -> Result<PathBuf> {
    let path = Path::new(raw);
    if path.is_absolute() {
        return Err(Error::Config {
            path: dir.join("config.json"),
            message: format!("path must be relative to the guide: {raw:?}"),
        });
    }
    let joined = dir.join(path);
    match kind {
        PathKind::Read => workspace.resolve_read(joined),
        PathKind::Write => workspace.resolve_write(joined),
    }
}

fn resolve_optional(
    workspace: &Workspace,
    dir: &Path,
    raw: Option<&str>,
    kind: PathKind,
) -> Result<Option<PathBuf>> {
    let Some(raw) = raw else {
        return Ok(None);
    };
    resolve_guide_path(workspace, dir, raw, kind).map(Some)
}

fn require_extension(config: &Path, path: &Path, expected: &str) -> Result<()> {
    let actual = path.extension().and_then(|value| value.to_str());
    if actual.is_some_and(|value| value.eq_ignore_ascii_case(expected)) {
        Ok(())
    } else {
        Err(config_error(
            config,
            format!("expected a .{expected} file, got {}", path.display()),
        ))
    }
}

fn expected_content_extension(schema_version: u32) -> &'static str {
    match schema_version {
        2 => "md",
        _ => "html",
    }
}

fn validate_target_pages(range: &str) -> Result<()> {
    let invalid = || Error::InvalidTargetPages {
        range: range.to_string(),
    };
    let range = range.trim();
    if range.is_empty() {
        return Err(invalid());
    }
    if let Some((start, end)) = range.split_once('-') {
        let start: u32 = start.trim().parse().map_err(|_| invalid())?;
        let end: u32 = end.trim().parse().map_err(|_| invalid())?;
        if start == 0 || end < start {
            return Err(invalid());
        }
    } else {
        let page: u32 = range.parse().map_err(|_| invalid())?;
        if page == 0 {
            return Err(invalid());
        }
    }
    Ok(())
}

fn read_title(path: &Path) -> Option<String> {
    let bytes = fs::read(path).ok()?;
    let value: Value = serde_json::from_slice(&bytes).ok()?;
    value.get("title")?.as_str().map(ToString::to_string)
}

fn config_error(path: &Path, message: impl Into<String>) -> Error {
    Error::Config {
        path: path.to_path_buf(),
        message: message.into(),
    }
}

fn default_schema_version() -> u32 {
    1
}

fn default_language() -> String {
    "tr".to_string()
}

fn default_theme() -> String {
    "paper".to_string()
}
