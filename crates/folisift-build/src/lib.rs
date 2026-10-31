//! Shared guide-build orchestration.
//!
//! Both hosts call this service so structure, genre, layout, coverage, visual
//! and revision gates cannot drift. Hosts still own their JSON presentation.

use std::path::{Path, PathBuf};

use folisift_core::{Guide, StyleProfile, Workspace, gates};
use folisift_parse::{
    AuditOptions, GuideVisualReport, Inventory, LayoutAudit, VisualAudit, analyze_sources,
    check_coverage, revision_coverage_issues,
};
use folisift_render::{RenderOptions, Rendered, render, styles};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Core(#[from] folisift_core::Error),
    #[error(transparent)]
    Parse(#[from] folisift_parse::Error),
    #[error(transparent)]
    Render(#[from] folisift_render::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Message(String),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub struct BuildRequest {
    pub slug: String,
    pub content: Option<PathBuf>,
    pub output: Option<PathBuf>,
    pub check: bool,
}

#[derive(Debug)]
pub struct BuildOutcome {
    pub output: PathBuf,
    pub rendered: Rendered,
    pub style: StyleProfile,
    pub style_rationale: Option<String>,
    pub markdown: String,
    pub layout: Option<LayoutAudit>,
    pub coverage: Option<folisift_parse::CoverageReport>,
    pub visual: Option<GuideVisualReport>,
    pub revision_coverage: bool,
    pub genre_passed: bool,
    pub passed: Option<bool>,
}

pub async fn build(workspace: &Workspace, request: BuildRequest) -> Result<BuildOutcome> {
    let guide = Guide::load(workspace, &request.slug)?;
    let content_path = match &request.content {
        Some(path) => workspace.resolve_read(path)?,
        None => guide.content.clone(),
    };
    if content_path.extension().and_then(|value| value.to_str()) != Some("md") {
        return Err(Error::Message(format!(
            "{} is not Markdown; pass content=guides/<slug>/guide.md",
            content_path.display()
        )));
    }
    let output_path = match &request.output {
        Some(path) => workspace.resolve_write(path)?,
        None => guide.output.clone(),
    };
    let markdown = std::fs::read_to_string(&content_path)?;
    let style = guide.style();
    let source_names = guide
        .config
        .sources
        .iter()
        .filter_map(|source| {
            Path::new(source)
                .file_stem()
                .and_then(|stem| stem.to_str())
                .map(ToString::to_string)
        })
        .collect::<Vec<_>>();
    let rendered = render(
        &markdown,
        &RenderOptions {
            guide_root: &guide.dir,
            title: &guide.config.title,
            subtitle: "",
            course: "",
            language: &guide.config.language,
            sources: &source_names,
            style: Some(&style),
            styles_root: Some(workspace.root()),
            cache_dir: Some(&workspace.root().join("data/cache/build")),
        },
    )?;

    let staged = if request.check {
        Some(StagedPdf::create(&output_path, &rendered.pdf)?)
    } else {
        write_pdf(&output_path, &rendered.pdf)?;
        None
    };
    let audit_path = staged
        .as_ref()
        .map_or(output_path.as_path(), StagedPdf::path);

    let mut layout = None;
    let mut coverage = None;
    let mut visual = None;
    let mut revision_coverage = false;
    let genre_ok = true;
    let mut visual_passed = None;
    if request.check {
        let issues = gates::genre_issues(&style.structure, &markdown);
        if !issues.is_empty() {
            return Err(Error::Message(format!(
                "revision grammar failed:\n- {}",
                issues.join("\n- ")
            )));
        }
        let audit_options = audit_options_for(workspace, &style)?;
        let report = folisift_parse::audit_pdf_with(audit_path, None, audit_options)?;
        if !report.underfilled_pages.is_empty() {
            let pages = report
                .underfilled_pages
                .iter()
                .map(|page| page.page)
                .collect::<Vec<_>>();
            return Err(Error::Message(format!("underfilled body pages: {pages:?}")));
        }
        let page_issues = gates::page_count_issues(&style.structure, report.pages);
        if !page_issues.is_empty() {
            return Err(Error::Message(page_issues.join("\n")));
        }
        layout = Some(report);

        if gates::uses_course_coverage(&style.structure)
            || gates::uses_revision_coverage(&style.structure)
        {
            let inventory = load_inventory(&guide).await?;
            if gates::uses_revision_coverage(&style.structure) {
                let issues = revision_coverage_issues(&inventory, &markdown);
                if !issues.is_empty() {
                    return Err(Error::Message(format!(
                        "revision source coverage failed:\n- {}",
                        issues.join("\n- ")
                    )));
                }
                revision_coverage = true;
            } else {
                if inventory.totals.low_text_pages > 0 {
                    let report = VisualAudit::load_and_validate(
                        &guide.visual_audit(),
                        &inventory,
                        &guide.dir,
                    )?;
                    visual_passed = Some(report.passed);
                    if !report.passed {
                        return Err(Error::Message(format!(
                            "visual audit failed: {} missing, {} unresolved",
                            report.missing.len(),
                            report.unresolved.len()
                        )));
                    }
                    visual = Some(report);
                }
                let report =
                    check_coverage(&inventory, &markdown, folisift_parse::DEFAULT_THRESHOLD);
                if let Some(path) = &guide.coverage {
                    report.write_markdown(path)?;
                }
                if !report.passed {
                    return Err(Error::Message(format!(
                        "coverage gate failed: {}/{} headings missing, {} empty sections",
                        report.missing,
                        report.total,
                        report.empty_sections.len()
                    )));
                }
                coverage = Some(report);
            }
        }
    }
    let coverage_passed = coverage
        .as_ref()
        .map(|report| report.passed)
        .or(revision_coverage.then_some(true));
    let passed = request.check.then(|| {
        gates::top_level_passed(
            layout.as_ref().is_some_and(|report| report.passed),
            genre_ok,
            coverage_passed,
            visual_passed,
        )
    });
    if let Some(staged) = staged {
        staged.commit()?;
    }
    Ok(BuildOutcome {
        output: output_path,
        rendered,
        style,
        style_rationale: guide.config.style_rationale,
        markdown,
        layout,
        coverage,
        visual,
        revision_coverage,
        genre_passed: genre_ok,
        passed,
    })
}

fn audit_options_for(workspace: &Workspace, style: &StyleProfile) -> Result<AuditOptions> {
    let loaded = styles::load_with(
        workspace.root(),
        &style.aesthetic,
        Some(style.palette.as_str()),
        Some(style.fonts.as_str()),
    )?;
    Ok(AuditOptions::from_tokens(&loaded.tokens))
}

async fn load_inventory(guide: &Guide) -> Result<Inventory> {
    if let Some(path) = &guide.inventory
        && path.exists()
    {
        let bytes = std::fs::read(path)?;
        let inventory = serde_json::from_slice::<Inventory>(&bytes)
            .map_err(|error| Error::Message(format!("inventory JSON: {error}")))?;
        if inventory.matches_sources(&guide.sources) {
            return Ok(inventory);
        }
    }
    analyze_sources(&guide.sources, folisift_parse::LOW_TEXT_THRESHOLD)
        .await
        .map_err(Error::from)
}

fn write_pdf(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(parent)?;
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("output.pdf");
    let temporary = parent.join(format!(".{name}.{}.tmp", std::process::id()));
    std::fs::write(&temporary, bytes)?;
    std::fs::rename(&temporary, path)
}

struct StagedPdf {
    path: PathBuf,
    output: PathBuf,
    committed: bool,
}

impl StagedPdf {
    fn create(output: &Path, bytes: &[u8]) -> std::io::Result<Self> {
        let parent = output.parent().unwrap_or_else(|| Path::new("."));
        std::fs::create_dir_all(parent)?;
        let name = output
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("output.pdf");
        let path = parent.join(format!(".{name}.{}.staging.pdf", std::process::id()));
        std::fs::write(&path, bytes)?;
        Ok(Self {
            path,
            output: output.to_path_buf(),
            committed: false,
        })
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn commit(mut self) -> std::io::Result<()> {
        std::fs::rename(&self.path, &self.output)?;
        self.committed = true;
        Ok(())
    }
}

impl Drop for StagedPdf {
    fn drop(&mut self) {
        if !self.committed {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}
