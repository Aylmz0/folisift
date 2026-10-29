use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{Error, Result, io};
use crate::paths::{Workspace, validate_slug};

pub const SUPPORTED_SCHEMA_VERSIONS: [u32; 3] = [1, 2, 3];

pub const AESTHETICS: &[&str] = &[
    "paper",
    "reader",
    "clean",
    "journal",
    "brief",
    "clinical",
    "casebook",
    "accessible",
];
pub const STRUCTURES: &[&str] = &[
    "a-g",
    "imrad",
    "irac",
    "soap",
    "formula-sheet",
    "literature-matrix",
    "lecture-notes",
    "executive-brief",
];
pub const LANGUAGES: &[&str] = &["tr", "en"];
pub const VOICES: &[&str] = &[
    "academic",
    "technical",
    "plain",
    "legal",
    "clinical",
    "business",
];
pub const STUDY_MODES: &[&str] = &["exam", "deep", "reference", "teach"];
pub const DOMAINS: &[&str] = &[
    "general",
    "cs",
    "math",
    "medicine",
    "law",
    "business",
    "humanities",
    "language",
];
pub const CITATIONS: &[&str] = &["page-ref", "apa", "ieee", "chicago", "oscola"];

/// The independent style axes for a guide.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StyleProfile {
    #[serde(default = "default_aesthetic")]
    pub aesthetic: String,
    #[serde(default = "default_structure")]
    pub structure: String,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default = "default_source_language")]
    pub source_language: String,
    #[serde(default = "default_voice")]
    pub voice: String,
    #[serde(default = "default_study_mode")]
    pub study_mode: String,
    #[serde(default = "default_domain")]
    pub domain: String,
    #[serde(default = "default_citation")]
    pub citation: String,
}

impl Default for StyleProfile {
    fn default() -> Self {
        Self {
            aesthetic: default_aesthetic(),
            structure: default_structure(),
            language: default_language(),
            source_language: default_source_language(),
            voice: default_voice(),
            study_mode: default_study_mode(),
            domain: default_domain(),
            citation: default_citation(),
        }
    }
}

impl StyleProfile {
    pub fn validate(&self) -> std::result::Result<(), String> {
        for (name, value, allowed) in [
            ("aesthetic", self.aesthetic.as_str(), AESTHETICS),
            ("structure", self.structure.as_str(), STRUCTURES),
            ("language", self.language.as_str(), LANGUAGES),
            ("source_language", self.source_language.as_str(), LANGUAGES),
            ("voice", self.voice.as_str(), VOICES),
            ("study_mode", self.study_mode.as_str(), STUDY_MODES),
            ("domain", self.domain.as_str(), DOMAINS),
            ("citation", self.citation.as_str(), CITATIONS),
        ] {
            if !allowed.contains(&value) {
                return Err(format!(
                    "style.{name} = {value:?} is not supported; expected one of {allowed:?}"
                ));
            }
        }
        let structure = self.structure.as_str();
        let domain = self.domain.as_str();
        if structure == "soap" && !matches!(domain, "medicine" | "general") {
            return Err(format!("soap is not compatible with domain {domain:?}"));
        }
        if structure == "irac" && !matches!(domain, "law" | "business" | "general") {
            return Err(format!("irac is not compatible with domain {domain:?}"));
        }
        if structure == "executive-brief" && !matches!(domain, "business" | "general") {
            return Err(format!(
                "executive-brief is not compatible with domain {domain:?}"
            ));
        }
        if self.aesthetic == "clinical" && structure != "soap" {
            return Err("clinical aesthetic requires soap structure".to_string());
        }
        if self.aesthetic == "casebook" && structure != "irac" {
            return Err("casebook aesthetic requires irac structure".to_string());
        }
        Ok(())
    }
}

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
    #[serde(default)]
    pub style: Option<StyleProfile>,
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
    pub style: String,
    pub language: String,
    pub output: String,
    pub sources: usize,
    pub content_words: usize,
    pub section_count: usize,
    pub valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Guide {
    /// Host-authored visual disposition manifest for low-text source pages.
    pub fn visual_audit(&self) -> PathBuf {
        self.dir.join("visual-audit.json")
    }

    /// Effective style: schema-3 `style`, or the legacy language/theme fields.
    pub fn style(&self) -> StyleProfile {
        self.config.style.clone().unwrap_or_else(|| StyleProfile {
            aesthetic: self.config.theme.clone(),
            language: self.config.language.clone(),
            ..StyleProfile::default()
        })
    }

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
        let style = config.style.clone().unwrap_or_else(|| StyleProfile {
            aesthetic: config.theme.clone(),
            language: config.language.clone(),
            ..StyleProfile::default()
        });
        style
            .validate()
            .map_err(|message| config_error(&config_path, message))?;
        if config.schema_version >= 3 && config.style.is_some() && style.language != config.language
        {
            return Err(config_error(
                &config_path,
                "style.language must match the flat language field",
            ));
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
                let style = guide.style();
                summaries.push(GuideSummary {
                    slug: guide.slug,
                    title: guide.config.title,
                    schema_version: guide.config.schema_version,
                    style: style.aesthetic,
                    language: style.language,
                    output: guide.config.output,
                    sources: guide.sources.len(),
                    content_words: prose_words(&content),
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
                    style: "unknown".to_string(),
                    language: String::new(),
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
        2 | 3 => "md",
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

fn prose_words(markdown: &str) -> usize {
    markdown
        .lines()
        .map(str::trim)
        .filter(|line| {
            !line.is_empty()
                && !line.starts_with('#')
                && !line.starts_with('|')
                && !line.starts_with(":::")
                && !line.starts_with("```")
                && !line.starts_with("---")
        })
        .map(|line| line.split_whitespace().count())
        .sum()
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

fn default_aesthetic() -> String {
    "paper".to_string()
}

fn default_structure() -> String {
    "a-g".to_string()
}

fn default_source_language() -> String {
    "tr".to_string()
}

fn default_voice() -> String {
    "academic".to_string()
}

fn default_study_mode() -> String {
    "exam".to_string()
}

fn default_domain() -> String {
    "general".to_string()
}

fn default_citation() -> String {
    "page-ref".to_string()
}

fn default_theme() -> String {
    "paper".to_string()
}
