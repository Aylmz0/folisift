use std::fs;
use std::path::PathBuf;

use crate::config::{Guide, GuideConfig, StyleProfile};
use crate::error::{Error, Result, io};
use crate::paths::{Workspace, validate_slug};

#[derive(Debug, Clone)]
pub struct ScaffoldOptions {
    pub slug: String,
    pub title: String,
    pub course: String,
    pub sources: Vec<PathBuf>,
    pub style: StyleProfile,
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

    options.style.validate().map_err(|message| Error::Config {
        path: guide_dir.join("config.json"),
        message,
    })?;
    let output = format!("../../output/{}.pdf", options.slug);
    let config = GuideConfig {
        schema_version: 3,
        slug: options.slug.clone(),
        title: options.title.clone(),
        language: options.style.language.clone(),
        content: "guide.md".to_string(),
        stylesheet: None,
        output,
        sources: relative_sources,
        inventory: Some("data/inventory.json".to_string()),
        coverage: Some("data/coverage.md".to_string()),
        target_pages: None,
        theme: options.style.aesthetic.clone(),
        style: Some(options.style.clone()),
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

    let course = options.course.replace('"', "'");
    let title = options.title.replace('"', "'");
    let structured_sections =
        crate::structures::scaffold_sections(&options.style.structure, &options.style.language);
    let sections_markdown = if structured_sections.is_empty() {
        if options.style.language == "en" {
            "## 1.1 Introduction\n\nSource content goes here. [PD0, p. 1]\n".to_string()
        } else {
            "## 1.1 Giriş\n\nKaynak içeriği buraya gelecek. [PD0, s. 1]\n".to_string()
        }
    } else {
        let mut body = String::new();
        for (index, (section, hint)) in structured_sections.iter().enumerate() {
            let hint = if options.style.language == "en" {
                "Source content goes here."
            } else {
                hint.as_str()
            };
            let source = if options.style.language == "en" {
                "p. 1"
            } else {
                "s. 1"
            };
            body.push_str(&format!(
                "## 1.{} {}\n\n{} [PD0, {source}]\n\n",
                index + 1,
                section,
                hint
            ));
        }
        body
    };
    let (front, toc) = if options.style.language == "en" {
        (
            "::: front kicker=\"Front matter · Reading note\" title=\"About this guide\" lead=\"This guide turns source slides into an exam-oriented study text.\"\nThis section will be written by the host LLM.\n:::",
            "::: toc title=\"Guide map\" note=\"Page numbers refer to this PDF.\"\n:::",
        )
    } else {
        (
            "::: front kicker=\"Ön kısım · Okuma notu\" title=\"Bu kitapçık hakkında\" lead=\"Bu kılavuz, kaynak slaytlardan sınav odaklı bir çalışma metni üretir.\"\nBu bölüm host LLM tarafından yazılacaktır.\n:::",
            "::: toc title=\"Kılavuzun haritası\" note=\"Sayfa numaraları bu PDF’e aittir.\"\n:::",
        )
    };
    let chapter_labels = if options.style.language == "en" {
        (
            "B · DETAILED REVIEW",
            "SOURCE",
            "CHAPTER 1",
            "Introduction",
            "Chapter 1 — Introduction",
            "Chapter 1 · Introduction",
        )
    } else {
        (
            "B · AYRINTILI İNCELEME",
            "KAYNAK",
            "BÖLÜM 1",
            "Giriş",
            "Bölüm 1 — Giriş",
            "Bölüm 1 · Giriş",
        )
    };
    let guide_content = format!(
        "---\ntitle: {title:?}\ncourse: {course:?}\nrunning: {running:?}\nlanguage: {language:?}\nsources: []\n---\n\n\
         {front}\n\n\
         {toc}\n\n\
         ::: chapter part=\"{part}\" side=\"{side}\" kicker=\"{kicker}\" title=\"{chapter_title}\" toc=\"{toc_label}\" running=\"{running_label}\"\n\
         :::\n\n\
         {sections_markdown}\n",
        front = front,
        toc = toc,
        part = chapter_labels.0,
        side = chapter_labels.1,
        kicker = chapter_labels.2,
        chapter_title = chapter_labels.3,
        toc_label = chapter_labels.4,
        running_label = chapter_labels.5,
        sections_markdown = sections_markdown,
        title = title,
        course = course,
        running = if options.course.trim().is_empty() {
            &options.title
        } else {
            &options.course
        },
        language = options.style.language,
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
