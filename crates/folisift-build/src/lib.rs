//! Shared guide-build orchestration.
//!
//! Both hosts call this service so structure, genre, layout, coverage, visual
//! and revision gates cannot drift. Hosts still own their JSON presentation.

use std::path::{Path, PathBuf};

use folisift_core::{Guide, StyleProfile, Workspace, gates};
use folisift_parse::{
    AuditOptions, GuideVisualReport, Inventory, LayoutAudit, VisualAudit, analyze_sources,
    check_coverage,
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
    pub incremental: Option<serde_json::Value>,
}

fn fail_with_draft(message: String, staged: &mut Option<StagedPdf>) -> Error {
    let mut message = message;
    if let Some(staged) = staged.take() {
        match staged.persist_draft() {
            Ok(path) => message.push_str(&format!("\ndraft_path: {}", path.display())),
            Err(error) => message.push_str(&format!("\ndraft could not be kept: {error}")),
        }
    }
    Error::Message(message)
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
    let incremental = incremental_report(&guide, &markdown)?;
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

    let mut staged = if request.check {
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
            return Err(fail_with_draft(
                format!(
                    "{} grammar failed:\n- {}",
                    style.structure,
                    issues.join("\n- ")
                ),
                &mut staged,
            ));
        }
        let audit_options = audit_options_for(workspace, &style)?;
        let report = folisift_parse::audit_pdf_with(audit_path, None, audit_options)?;
        if !report.underfilled_pages.is_empty() {
            let pages = report
                .underfilled_pages
                .iter()
                .map(|page| page.page)
                .collect::<Vec<_>>();
            return Err(fail_with_draft(
                format!("underfilled body pages: {pages:?}"),
                &mut staged,
            ));
        }
        let mut page_issues = gates::page_count_issues(&style.structure, report.pages);
        page_issues.extend(gates::page_range_issues(
            guide.config.target_pages.as_deref(),
            report.pages,
        ));
        if !page_issues.is_empty() {
            return Err(fail_with_draft(page_issues.join("\n"), &mut staged));
        }
        layout = Some(report);

        if gates::uses_course_coverage(&style.structure)
            || gates::uses_revision_coverage(&style.structure)
        {
            let inventory = load_inventory(&guide).await?;
            if gates::uses_revision_coverage(&style.structure) {
                let report = folisift_parse::revision_coverage_report(&inventory, &markdown);
                if let Some(path) = &guide.coverage {
                    report.write_markdown(path)?;
                }
                if !report.passed {
                    return Err(fail_with_draft(
                        format!(
                            "revision source coverage failed:\n- {}",
                            report.issues.join("\n- ")
                        ),
                        &mut staged,
                    ));
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
                        return Err(fail_with_draft(
                            format!(
                                "visual audit failed: {} missing, {} unresolved",
                                report.missing.len(),
                                report.unresolved.len()
                            ),
                            &mut staged,
                        ));
                    }
                    visual = Some(report);
                }
                let manifest_path = guide.dir.join("data/coverage-manifest.json");
                let report = if manifest_path.is_file() {
                    let manifest = folisift_parse::CoverageManifest::read_json(&manifest_path)?;
                    manifest.report(&inventory, &markdown)
                } else {
                    check_coverage(&inventory, &markdown, folisift_parse::DEFAULT_THRESHOLD)
                };
                if let Some(path) = &guide.coverage {
                    report.write_markdown(path)?;
                }
                if !report.passed {
                    return Err(fail_with_draft(
                        format!(
                            "coverage gate failed: {}/{} headings missing, {} empty sections",
                            report.missing,
                            report.total,
                            report.empty_sections.len()
                        ),
                        &mut staged,
                    ));
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
        incremental: Some(incremental),
    })
}

fn incremental_report(guide: &Guide, markdown: &str) -> Result<serde_json::Value> {
    let mut current = std::collections::BTreeMap::new();
    let mut current_heading = String::from("(front)");
    let mut current_body = String::new();
    for line in markdown.lines() {
        if let Some(heading) = line.strip_prefix("## ") {
            current.insert(
                current_heading.clone(),
                folisift_render::content_hash(current_body.trim()),
            );
            current_heading = heading.trim().to_string();
            current_body.clear();
        } else {
            current_body.push_str(line);
            current_body.push('\n');
        }
    }
    current.insert(
        current_heading,
        folisift_render::content_hash(current_body.trim()),
    );
    let path = guide.dir.join("data/incremental-sections.json");
    let previous = std::fs::read_to_string(&path)
        .ok()
        .and_then(|text| {
            serde_json::from_str::<std::collections::BTreeMap<String, String>>(&text).ok()
        })
        .unwrap_or_default();
    let reused = current
        .iter()
        .filter(|(heading, hash)| previous.get(*heading) == Some(*hash))
        .count();
    let changed = current
        .iter()
        .filter(|(heading, hash)| previous.get(*heading) != Some(*hash))
        .map(|(heading, _)| heading.clone())
        .collect::<Vec<_>>();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(&current)
        .map_err(|error| Error::Message(format!("incremental report: {error}")))?;
    std::fs::write(&path, text + "\n")?;
    Ok(serde_json::json!({
        "reused_sections": reused,
        "rebuilt_sections": current.len().saturating_sub(reused),
        "layout_recomputed": true,
        "changed_headings": changed,
    }))
}

/// Audit a guide PDF with the guide's own margins and typography.
pub fn audit_guide(
    workspace: &Workspace,
    slug: &str,
    pdf: Option<&Path>,
    expected_pages: Option<usize>,
) -> Result<LayoutAudit> {
    let guide = Guide::load(workspace, slug)?;
    let options = audit_options_for(workspace, &guide.style())?;
    let pdf = pdf.map_or_else(|| guide.output.clone(), Path::to_path_buf);
    folisift_parse::audit_pdf_with(&pdf, expected_pages, options).map_err(Error::from)
}

/// One compact release report: build, coverage, audit, visual, cards, claims
/// and diagram counts. The report never hides a failed gate.
pub async fn qa(workspace: &Workspace, slug: &str) -> Result<serde_json::Value> {
    let outcome = build(
        workspace,
        BuildRequest {
            slug: slug.to_string(),
            content: None,
            output: None,
            check: true,
        },
    )
    .await?;
    let guide = Guide::load(workspace, slug)?;
    let markdown = outcome.markdown.clone();
    let claims = folisift_core::claims::extract_claims(&markdown);
    let cards =
        folisift_core::cards::cards_from_markdown(&markdown, &guide.slug, &[1, 3, 7, 14, 30]);
    let diagram_specs = std::fs::read_dir(guide.dir.join("diagrams"))
        .map(|entries| {
            entries
                .filter_map(std::result::Result::ok)
                .filter(|entry| {
                    entry.path().extension().and_then(|value| value.to_str()) == Some("json")
                })
                .count()
        })
        .unwrap_or(0);
    let diagram_assets = count_files(&guide.dir.join("assets/generated"), "svg");
    let audit = outcome.layout.clone();
    let audit_passed = audit.as_ref().is_some_and(|report| report.passed);
    let underfilled = audit
        .as_ref()
        .map_or(0, |report| report.underfilled_pages.len());
    let passed = outcome.passed.unwrap_or(false) && audit_passed && underfilled == 0;
    Ok(serde_json::json!({
        "guide": guide.slug,
        "passed": passed,
        "build": {
            "passed": outcome.passed,
            "pages": outcome.rendered.pages,
            "content_hash": outcome.rendered.content_hash,
            "cache_hit": outcome.rendered.cache_hit,
            "output": outcome.output.display().to_string(),
            "warnings": outcome.rendered.warnings,
        },
        "coverage": outcome.coverage,
        "audit": audit,
        "visual": outcome.visual,
        "cards": {
            "count": cards.len(),
            "warning": if cards.is_empty() { Some("no recall/answer pairs") } else { None },
        },
        "claims": {
            "total": claims.total,
            "referenced": claims.referenced,
            "unreferenced": claims.unreferenced,
            "noise_filtered": claims.noise_filtered,
        },
        "diagrams": {
            "specs": diagram_specs,
            "assets": diagram_assets,
        },
    }))
}

fn count_files(path: &Path, extension: &str) -> usize {
    let Ok(entries) = std::fs::read_dir(path) else {
        return 0;
    };
    entries
        .filter_map(std::result::Result::ok)
        .map(|entry| {
            let path = entry.path();
            if path.is_dir() {
                count_files(&path, extension)
            } else if path.extension().and_then(|value| value.to_str()) == Some(extension) {
                1
            } else {
                0
            }
        })
        .sum()
}

/// Resolve the audit margins/typography options for a guide style.
pub fn audit_options_for(workspace: &Workspace, style: &StyleProfile) -> Result<AuditOptions> {
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

    /// Keep the staged PDF as `<name>_draft.pdf` when a check fails.
    fn persist_draft(mut self) -> std::io::Result<PathBuf> {
        let parent = self.output.parent().unwrap_or_else(|| Path::new("."));
        let stem = self
            .output
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("output");
        let draft = parent.join(format!("{stem}_draft.pdf"));
        std::fs::rename(&self.path, &draft)?;
        self.committed = true;
        Ok(draft)
    }
}

impl Drop for StagedPdf {
    fn drop(&mut self) {
        if !self.committed {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::count_files;

    #[test]
    fn counts_nested_assets_by_extension() {
        let root = std::env::temp_dir().join(format!("folisift-assets-{}", std::process::id()));
        std::fs::create_dir_all(root.join("nested")).expect("dir");
        std::fs::write(root.join("a.svg"), "x").expect("a");
        std::fs::write(root.join("b.txt"), "x").expect("b");
        std::fs::write(root.join("nested/c.svg"), "x").expect("c");
        assert_eq!(count_files(&root, "svg"), 2);
        let _ = std::fs::remove_dir_all(root);
    }
}
