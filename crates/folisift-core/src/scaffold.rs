use std::fs;
use std::path::{Path, PathBuf};

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
    pub output_intent: Option<String>,
    pub target_pages: Option<String>,
}

/// Create a new schema-3 guide bundle and return its validated config.
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
        sources: relative_sources.clone(),
        inventory: Some("data/inventory.json".to_string()),
        coverage: Some("data/coverage.md".to_string()),
        target_pages: options.target_pages.clone(),
        output_intent: options.output_intent.clone(),
        review_intervals: None,
        theme: options.style.aesthetic.clone(),
        style: Some(options.style.clone()),
        style_rationale: Some(format!(
            "Scaffold default for {}; the host must refine or confirm this rationale.",
            options.style.aesthetic
        )),
        style_source: Some("scaffold".to_string()),
        audit: None,
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
    let source_ids = relative_sources
        .iter()
        .filter_map(|path| {
            Path::new(path)
                .file_stem()
                .and_then(|value| value.to_str())
                .map(ToString::to_string)
        })
        .collect::<Vec<_>>();
    let structured_sections =
        crate::structures::scaffold_sections(&options.style.structure, &options.style.language);
    if options.style.structure == "irac" {
        let mut headings = String::from("# Source headings\n\n");
        for source in &relative_sources {
            let id = Path::new(source)
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or(source);
            headings.push_str(&format!("## {id}\n\n"));
            headings
                .push_str("Run `folisift analyze` or `prepare_guide` to fill source headings.\n\n");
        }
        let path = guide_dir.join("source-headings.md");
        fs::write(&path, headings).map_err(|error| io(&path, error))?;
    }
    let sections_markdown = if options.style.structure == "irac" {
        let mut body = String::new();
        for case in 1..=10 {
            if options.style.language == "en" {
                body.push_str(&format!(
                    "## Case {case}\n\n### Issue\n\nDescribe the legal issue. [PD0, p. 1]\n\n### Rule\n\nState the applicable rule. [PD0, p. 1]\n\n### Application\n\nApply the rule to the facts. [PD0, p. 1]\n\n### Conclusion\n\nGive the reasoned conclusion. [PD0, p. 1]\n\n"
                ));
            } else {
                body.push_str(&format!(
                    "## Dava {case}\n\n### Sorun\n\nÇözülecek hukuki soruyu yaz. [PD0, s. 1]\n\n### Kural\n\nUygulanacak norm ve içtihadı yaz. [PD0, s. 1]\n\n### Uygulama\n\nKuralı olaya uygula. [PD0, s. 1]\n\n### Sonuç\n\nGerekçeli sonucu yaz. [PD0, s. 1]\n\n"
                ));
            }
        }
        body
    } else if structured_sections.is_empty() {
        if options.style.language == "en" {
            "## 1.1 Introduction\n\nSource content goes here. [PD0, p. 1]\n".to_string()
        } else {
            "## 1.1 Giriş\n\nKaynak içeriği buraya gelecek. [PD0, s. 1]\n".to_string()
        }
    } else {
        let mut body = String::new();
        for (index, (section, hint, level)) in structured_sections.iter().enumerate() {
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
                "{} 1.{} {}\n\n{} [PD0, {source}]\n\n",
                "#".repeat(usize::from(*level)),
                index + 1,
                section,
                hint
            ));
        }
        body
    };
    let toc = if options.style.language == "en" {
        "::: toc title=\"Guide map\" note=\"Page numbers refer to this PDF.\"\n:::"
    } else {
        "::: toc title=\"Kılavuzun haritası\" note=\"Sayfa numaraları bu PDF’e aittir.\"\n:::"
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
    let sources = format!("[{}]", source_ids.join(", "));
    let guide_content = format!(
        "---\ntitle: {title:?}\ncourse: {course:?}\nrunning: {running:?}\nlanguage: {language:?}\nsources: {sources}\n---\n\n\
         {toc}\n\n\
         ::: chapter part=\"{part}\" side=\"{side}\" kicker=\"{kicker}\" title=\"{chapter_title}\" toc=\"{toc_label}\" running=\"{running_label}\"\n\
         :::\n\n\
         {sections_markdown}\n",
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
