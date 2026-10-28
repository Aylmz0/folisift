use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use folisift_core::{Guide, ScaffoldOptions, Workspace, list_guides, scaffold, version};
use folisift_parse::{
    Inventory, SourceCache, analyze_sources, check_coverage, parse_page_list, read_source,
    render_pages, search_source,
};
use folisift_render::{RenderOptions, render};
use serde::Serialize;
use serde_json::json;

#[derive(Debug, Parser)]
#[command(
    name = "folisift",
    version = version::VERSION,
    about = "Deterministic study-guide pipeline"
)]
struct Cli {
    /// Workspace root. Defaults to the current directory.
    #[arg(long, global = true, env = "FOLISIFT_ROOT", default_value = ".")]
    root: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Start the MCP stdio server.
    Mcp,
    /// Show workspace root, version and guide count.
    WorkspaceInfo,
    /// List guides and whether each config is valid.
    ListGuides,
    /// Show a validated guide config and resolved paths.
    GuideStatus { slug: String },
    /// Diagnose PDFium, guides, cache and output paths before a build.
    Doctor,
    /// Audit a rendered PDF for empty pages, overflow and missing glyphs.
    Audit {
        /// PDF path, relative to the workspace root.
        #[arg(long)]
        pdf: PathBuf,
        /// Expected page count, e.g. 22.
        #[arg(long)]
        expected_pages: Option<usize>,
    },
    /// Build a source inventory with liteparse.
    Analyze {
        /// Source PDFs, relative to the workspace root.
        #[arg(long = "source", required = true, num_args = 1..)]
        sources: Vec<PathBuf>,
        /// Inventory JSON path.
        #[arg(long, default_value = "data/source.json")]
        out: PathBuf,
        /// Inventory Markdown path.
        #[arg(long, default_value = "data/source.md")]
        md: PathBuf,
        /// Low-text threshold in non-whitespace characters.
        #[arg(long, default_value_t = folisift_parse::LOW_TEXT_THRESHOLD)]
        low_threshold: usize,
    },
    /// Create a new guide bundle from source PDFs.
    Scaffold {
        #[arg(long)]
        slug: String,
        #[arg(long)]
        title: String,
        #[arg(long, default_value = "")]
        course: String,
        #[arg(long = "source", required = true, num_args = 1..)]
        sources: Vec<PathBuf>,
    },
    /// Render a guide and optionally run the coverage gate.
    Build {
        /// Guide slug under guides/.
        #[arg(long)]
        guide: String,
        /// Markdown content override (relative to the workspace).
        #[arg(long)]
        content: Option<PathBuf>,
        /// PDF output override (relative to the workspace).
        #[arg(long)]
        output: Option<PathBuf>,
        /// Run the coverage gate and fail on a missing section.
        #[arg(long)]
        check: bool,
    },
    /// Read a bounded page range from cached source text.
    ReadSource {
        #[arg(long)]
        source: PathBuf,
        #[arg(long, default_value = "1-20")]
        pages: String,
        #[arg(long)]
        max_chars: Option<usize>,
    },
    /// Search cached source text for a term.
    SearchSource {
        #[arg(long)]
        source: PathBuf,
        #[arg(long)]
        query: String,
        #[arg(long)]
        limit: Option<usize>,
        #[arg(long)]
        context_chars: Option<usize>,
    },
    /// Render PDF pages to PNG files for visual inspection.
    RenderPages {
        /// Source PDF, relative to the workspace root.
        #[arg(long)]
        source: PathBuf,
        /// Page list such as `1,3-5`. Omit for the first 8 pages.
        #[arg(long)]
        pages: Option<String>,
        /// Output directory.
        #[arg(long, default_value = "data/pages")]
        out: PathBuf,
        /// Compose a 2-column contact sheet.
        #[arg(long)]
        contact_sheet: bool,
        /// Render DPI (72–200).
        #[arg(long)]
        dpi: Option<f32>,
    },
}

#[derive(Debug, Serialize)]
struct GuideStatus {
    slug: String,
    title: String,
    schema_version: u32,
    language: String,
    theme: String,
    content: String,
    output: String,
    sources: Vec<String>,
    inventory: Option<String>,
    coverage: Option<String>,
    target_pages: Option<String>,
    content_words: usize,
    section_count: usize,
    empty_sections: Vec<String>,
}

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let workspace = Workspace::open(&cli.root)?;

    match cli.command {
        Command::Mcp => {
            folisift_mcp::serve(workspace.root()).await?;
        }
        Command::WorkspaceInfo => {
            let guides = list_guides(&workspace)?;
            let value = json!({
                "root": workspace.root().display().to_string(),
                "version": version::VERSION,
                "build": version::BUILD,
                "mcp_tool_schema_version": version::MCP_TOOL_SCHEMA_VERSION,
                "guides": guides.len(),
            });
            println!("{}", serde_json::to_string_pretty(&value)?);
        }
        Command::ListGuides => {
            let guides = list_guides(&workspace)?;
            println!("{}", serde_json::to_string_pretty(&guides)?);
        }
        Command::Doctor => {
            let pdfium = folisift_parse::init_pdfium(None).map_err(|error| error.to_string());
            let cache = workspace.root().join("data/cache");
            let cache_ok = std::fs::create_dir_all(&cache).is_ok()
                && cache
                    .metadata()
                    .map(|m| !m.permissions().readonly())
                    .unwrap_or(false);
            let guides = list_guides(&workspace).unwrap_or_default();
            let invalid_guides = guides.iter().filter(|guide| !guide.valid).count();
            let output = workspace.root().join("output");
            let output_ok = output.is_dir()
                && output
                    .metadata()
                    .map(|m| !m.permissions().readonly())
                    .unwrap_or(false);
            let font_count = folisift_render::embedded_font_count();
            let checks = vec![
                json!({"name": "pdfium", "ok": pdfium.is_ok(), "detail": pdfium.as_ref().err().map(ToString::to_string).unwrap_or_else(|| folisift_parse::pdfium::PDFIUM_VERSION.to_string())}),
                json!({"name": "cache", "ok": cache_ok, "detail": cache.display().to_string()}),
                json!({"name": "output", "ok": output_ok, "detail": output.display().to_string()}),
                json!({"name": "guides", "ok": !guides.is_empty() && invalid_guides == 0, "detail": format!("{} guide(s), {invalid_guides} invalid", guides.len())}),
                json!({"name": "fonts", "ok": font_count > 0, "detail": format!("{font_count} embedded face(s)")}),
            ];
            let ok = checks.iter().all(|check| check["ok"] == true);
            let value = json!({
                "ok": ok,
                "root": workspace.root().display().to_string(),
                "version": version::VERSION,
                "checks": checks,
                "next": if ok { "build a guide with `folisift build --guide <slug> --check`" } else { "fix the failing checks, then rerun `folisift doctor`" },
            });
            println!("{}", serde_json::to_string_pretty(&value)?);
            if !ok {
                return Err("doctor found environment problems".into());
            }
        }
        Command::Audit {
            pdf,
            expected_pages,
        } => {
            let pdf = workspace.resolve_read(&pdf)?;
            let report = folisift_parse::audit_pdf(&pdf, expected_pages)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            if !report.passed {
                return Err("layout audit failed".into());
            }
        }
        Command::GuideStatus { slug } => {
            let guide = Guide::load(&workspace, &slug)?;
            let markdown = std::fs::read_to_string(&guide.content)?;
            let body_chars = folisift_parse::guide_body_chars(&markdown);
            let empty_sections = body_chars
                .iter()
                .filter(|(number, chars)| {
                    number.chars().next().is_some_and(|c| c.is_ascii_digit())
                        && **chars < folisift_parse::MIN_SECTION_CHARS
                })
                .map(|(number, _)| number.clone())
                .collect();
            let status = GuideStatus {
                slug: guide.slug,
                title: guide.config.title,
                schema_version: guide.config.schema_version,
                language: guide.config.language,
                theme: guide.config.theme,
                content: guide.content.display().to_string(),
                output: guide.output.display().to_string(),
                sources: guide
                    .sources
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect(),
                inventory: guide.inventory.map(|path| path.display().to_string()),
                coverage: guide.coverage.map(|path| path.display().to_string()),
                target_pages: guide.config.target_pages,
                content_words: markdown.split_whitespace().count(),
                section_count: body_chars.len(),
                empty_sections,
            };
            println!("{}", serde_json::to_string_pretty(&status)?);
        }
        Command::Analyze {
            sources,
            out,
            md,
            low_threshold,
        } => {
            let sources = sources
                .iter()
                .map(|path| workspace.resolve_read(path))
                .collect::<Result<Vec<_>, _>>()?;
            let out = workspace.resolve_write(out)?;
            let md = workspace.resolve_write(md)?;
            let inventory = analyze_sources(&sources, low_threshold).await?;
            inventory.write_files(&out, &md)?;
            let cache = SourceCache::new(workspace.root().join("data/cache"));
            let mut texts = Vec::new();
            for source in &sources {
                let text = cache.load(source, false).await?;
                texts.push(json!({
                    "source": source.display().to_string(),
                    "text": text.cache_path.display().to_string(),
                    "sha256": text.sha256,
                    "cache_hit": text.cache_hit,
                }));
            }
            let value = json!({
                "inventory": out.display().to_string(),
                "markdown": md.display().to_string(),
                "texts": texts,
                "pages": inventory.totals.pages,
                "words": inventory.totals.words,
                "low_text_pages": inventory.totals.low_text_pages,
            });
            println!("{}", serde_json::to_string_pretty(&value)?);
        }
        Command::Scaffold {
            slug,
            title,
            course,
            sources,
        } => {
            let sources = sources
                .iter()
                .map(|path| workspace.resolve_read(path))
                .collect::<Result<Vec<_>, _>>()?;
            let title_for_content = title.clone();
            let guide = scaffold(
                &workspace,
                &ScaffoldOptions {
                    slug,
                    title,
                    course,
                    sources: sources.clone(),
                },
            )?;
            let inventory = analyze_sources(&sources, folisift_parse::LOW_TEXT_THRESHOLD).await?;
            let mut body =
                format!("---\ntitle: {title_for_content:?}\nlanguage: tr\nsources: []\n---\n\n");
            for source in &inventory.sources {
                body.push_str(&format!("# {}\n\n", source.title_or_filename()));
                for heading in &source.headings {
                    let number = heading
                        .heading
                        .split_whitespace()
                        .next()
                        .unwrap_or_default();
                    if number.matches('.').count() == 1
                        && number.chars().next().is_some_and(|c| c.is_ascii_digit())
                    {
                        body.push_str(&format!(
                            "## {}\n\nKaynak: [{}]\n\n",
                            heading.heading, source.filename
                        ));
                    }
                }
            }
            std::fs::write(&guide.content, body)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "slug": guide.slug,
                    "content": guide.content.display().to_string(),
                    "output": guide.output.display().to_string(),
                }))?
            );
        }
        Command::Build {
            guide,
            content,
            output,
            check,
        } => {
            let guide = Guide::load(&workspace, &guide)?;
            let content_path = match content {
                Some(path) => workspace.resolve_read(path)?,
                None => guide.content.clone(),
            };
            if content_path.extension().and_then(|value| value.to_str()) != Some("md") {
                return Err(format!(
                    "{} is not Markdown; pass --content guides/<slug>/guide.md during the v2 migration",
                    content_path.display()
                )
                .into());
            }
            let output_path = match output {
                Some(path) => workspace.resolve_write(path)?,
                None => guide.output.clone(),
            };
            let markdown = std::fs::read_to_string(&content_path)?;
            let sources = guide
                .config
                .sources
                .iter()
                .filter_map(|source| {
                    std::path::Path::new(source)
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
                    sources: &sources,
                },
            )?;
            std::fs::write(&output_path, &rendered.pdf)?;

            let mut coverage = None;
            if check {
                let inventory = match &guide.inventory {
                    Some(path) if path.exists() => {
                        let bytes = std::fs::read(path)?;
                        serde_json::from_slice::<Inventory>(&bytes)?
                    }
                    _ => analyze_sources(&guide.sources, 120).await?,
                };
                let report = check_coverage(&inventory, &markdown, 0.55);
                if let Some(path) = &guide.coverage {
                    report.write_markdown(path)?;
                }
                coverage = Some(report);
            }
            if let Some(report) = &coverage
                && !report.passed
            {
                return Err(format!(
                    "coverage gate failed: {}/{} headings missing, {} empty sections",
                    report.missing,
                    report.total,
                    report.empty_sections.len()
                )
                .into());
            }

            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "output": output_path.display().to_string(),
                    "pages": rendered.pages,
                    "warnings": rendered.warnings.len(),
                    "coverage": coverage.as_ref().map(|report| json!({
                        "passed": report.passed,
                        "total": report.total,
                        "missing": report.missing,
                        "empty_sections": report.empty_sections.len(),
                        "weak_sections": report.weak_sections.len(),
                    })),
                }))?
            );
        }
        Command::ReadSource {
            source,
            pages,
            max_chars,
        } => {
            let source = workspace.resolve_read(source)?;
            let cache = SourceCache::new(workspace.root().join("data/cache"));
            let result = read_source(&cache, &source, Some(&pages), max_chars).await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        Command::SearchSource {
            source,
            query,
            limit,
            context_chars,
        } => {
            let source = workspace.resolve_read(source)?;
            let cache = SourceCache::new(workspace.root().join("data/cache"));
            let result = search_source(&cache, &source, &query, limit, context_chars).await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        Command::RenderPages {
            source,
            pages,
            out,
            contact_sheet,
            dpi,
        } => {
            let source = workspace.resolve_read(source)?;
            let out = workspace.resolve_write(out)?;
            let pages = pages.as_deref().map(parse_page_list).transpose()?;
            let result = render_pages(&source, pages, &out, contact_sheet, dpi).await?;
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "source": source.display().to_string(),
                    "out": out.display().to_string(),
                    "result": result,
                }))?
            );
        }
    }
    Ok(())
}
