use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use folisift_build::build as build_guide;
use folisift_core::{Guide, ScaffoldOptions, Workspace, list_guides, scaffold, version};
use folisift_links::{
    DEFAULT_MODEL, Embedder, LinkIndex, LinkOptions, Model2VecEmbedder, find_model_dir,
    read_layouts, read_report, sections_from_layout, verify_model_dir, write_markdown_report,
    write_report,
};
use folisift_parse::{
    DocumentGraph, SourceCache, analyze_sources_with_layout, parse_page_list, read_block_view,
    read_source_at, render_pages, search_source, write_layouts,
};
use folisift_render::{RenderOptions, render, styles};
use serde::Serialize;
use serde_json::{Value, json};

mod diagram_cmd;

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
    /// Start the MCP server over stdio, or HTTP with --http.
    Mcp {
        /// Bind address for streamable HTTP, e.g. 127.0.0.1:8000.
        #[arg(long)]
        http: Option<String>,
        /// Bearer token for HTTP; required for non-loopback binds.
        #[arg(long, env = "FOLISIFT_HTTP_TOKEN", hide_env_values = true)]
        http_token: Option<String>,
        /// Tool profile: core, source, authoring, review, fast, deep or full.
        #[arg(long, env = "FOLISIFT_MCP_PROFILE", default_value = "full")]
        profile: String,
    },
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
        /// Read margins/minimum font from this guide's resolved style.
        #[arg(long)]
        guide: Option<String>,
        /// Fail on underfilled body pages as well as layout defects.
        #[arg(long, default_value_t = false)]
        strict: bool,
    },
    /// Extract high-signal claims and citation gaps from a guide.
    Claims {
        #[arg(long)]
        guide: String,
        /// Markdown output path, relative to the workspace.
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Export recall/answer pairs as Anki-compatible CSV.
    Cards {
        #[arg(long)]
        guide: String,
        /// CSV output path, relative to the workspace.
        #[arg(long)]
        out: Option<PathBuf>,
        /// Comma-separated review intervals in days; overrides config.
        #[arg(long)]
        review_intervals: Option<String>,
    },
    /// Build or inspect local semantic links between source sections.
    Links {
        #[command(subcommand)]
        command: LinksCommand,
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
        /// Style/layout id (run `styles list`): paper, reader, clean,
        /// journal, brief, accessible, revision, notes, workbook, reference.
        #[arg(long = "style", default_value = "paper")]
        style: String,
        /// Structure profile: a-g, revision-sheet, knowledge-organiser,
        /// concept-summary, imrad, irac, soap, formula-sheet, lecture-notes,
        /// workbook, ...
        #[arg(long, default_value = "a-g")]
        structure: String,
        /// Output language: tr or en.
        #[arg(long, default_value = "tr")]
        language: String,
        /// Domain profile: general, cs, math, medicine, law, ...
        #[arg(long, default_value = "general")]
        domain: String,
        /// Optional palette id; empty uses the style default.
        #[arg(long)]
        palette: Option<String>,
        /// Optional font-pairing id; empty uses the style default.
        #[arg(long)]
        fonts: Option<String>,
        /// Reader task: exam-summary, concept-summary, study, deep, reference,
        /// practice or one-page.
        #[arg(long)]
        intent: Option<String>,
        /// Requested page budget, e.g. "3-5" or "12".
        #[arg(long)]
        target_pages: Option<String>,
    },
    /// List, inspect or preview engine-owned style packs.
    Styles {
        #[command(subcommand)]
        command: StylesCommand,
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
    /// Per-section page budget with concrete cut/move suggestions.
    Budget {
        #[arg(long)]
        guide: String,
        /// Optional target such as "22" or "18-22".
        #[arg(long)]
        target_pages: Option<String>,
    },
    /// One release report: build, coverage, audit, visual, cards, claims and diagrams.
    Qa {
        #[arg(long)]
        guide: String,
    },
    /// Return a bounded extractive brief for one source section.
    SectionBrief {
        #[arg(long)]
        guide: String,
        #[arg(long)]
        source: String,
        #[arg(long)]
        heading: String,
        #[arg(long, default_value_t = 600)]
        max_chars: usize,
    },
    /// Read one source section with an offset and truncation flag.
    ReadSection {
        #[arg(long)]
        guide: String,
        #[arg(long)]
        source: String,
        #[arg(long)]
        heading: String,
        #[arg(long, default_value_t = 4000)]
        max_chars: usize,
        #[arg(long, default_value_t = 0)]
        offset: usize,
    },
    /// Build, read or validate the reviewed coverage manifest.
    CoverageManifest {
        #[arg(long)]
        guide: String,
        /// build, read or validate.
        #[arg(long, default_value = "build")]
        action: String,
    },
    /// Build or inspect the deterministic document graph.
    Graph {
        #[command(subcommand)]
        command: GraphCommand,
    },
    /// Validate, build or inspect D2 diagram assets.
    Diagram {
        #[command(subcommand)]
        command: DiagramCommand,
    },
    /// Extract draft tables from a native-text PDF.
    Table {
        #[command(subcommand)]
        command: TableCommand,
    },
    /// Compare rendered pages of two PDFs and write diff images.
    Visual {
        #[command(subcommand)]
        command: VisualCommand,
    },
    /// Read a bounded page range from cached source text.
    ReadSource {
        #[arg(long)]
        source: PathBuf,
        #[arg(long, default_value = "1-20")]
        pages: String,
        #[arg(long)]
        max_chars: Option<usize>,
        /// Character offset to resume a page that exceeded max_chars.
        #[arg(long)]
        offset: Option<usize>,
    },
    /// Read a bounded view of stored layout blocks for one guide source.
    ReadBlocks {
        #[arg(long)]
        guide: String,
        #[arg(long)]
        source: String,
        #[arg(long)]
        pages: Option<String>,
        #[arg(long, default_value_t = 200)]
        max_blocks: usize,
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
        /// Crop as normalized fractions: left,top,width,height.
        #[arg(long)]
        crop: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
enum LinksCommand {
    /// Build document-links.json from stored layout blocks.
    Build {
        #[arg(long)]
        guide: String,
        #[arg(long, default_value_t = 5)]
        top_k: usize,
        #[arg(long, default_value_t = 0.75)]
        min_cosine: f32,
        #[arg(long, default_value_t = 0.55)]
        min_score: f32,
        /// Use the exploratory preset: top-k 8, min-cosine 0.65, min-score 0.45.
        #[arg(long, conflicts_with_all = ["top_k", "min_cosine", "min_score"])]
        explore: bool,
        /// Use BM25 only; no embedding model is loaded.
        #[arg(long)]
        no_model: bool,
        /// Optional model directory; otherwise FOLISIFT_MODEL_DIR / sidecar.
        #[arg(long)]
        model_dir: Option<PathBuf>,
    },
    /// Read links for one guide section or the whole guide.
    Read {
        #[arg(long)]
        guide: String,
        #[arg(long)]
        section: Option<String>,
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    /// Report model, blocks and links status without inference.
    Status {
        #[arg(long)]
        guide: String,
    },
}

#[derive(Debug, Subcommand)]
enum GraphCommand {
    /// Build document-graph.json from stored layout blocks and guide citations.
    Build {
        #[arg(long)]
        guide: String,
    },
    /// Report graph counts without rebuilding.
    Status {
        #[arg(long)]
        guide: String,
    },
    /// Search graph nodes by label.
    Query {
        #[arg(long)]
        guide: String,
        #[arg(long)]
        query: String,
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    /// Find the shortest graph path between two nodes or labels.
    Path {
        #[arg(long)]
        guide: String,
        #[arg(long)]
        from: String,
        #[arg(long)]
        to: String,
        #[arg(long, default_value_t = 8)]
        max_depth: usize,
    },
    /// Explain one node and its local edges.
    Explain {
        #[arg(long)]
        guide: String,
        #[arg(long)]
        node: String,
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
}

#[derive(Debug, Subcommand)]
enum DiagramCommand {
    /// Validate one or all diagram specs without running D2.
    Validate {
        #[arg(long)]
        guide: String,
        #[arg(long)]
        id: Option<String>,
        #[arg(long)]
        all: bool,
    },
    /// Render one or all diagram specs with D2.
    Build {
        #[arg(long)]
        guide: String,
        #[arg(long)]
        id: Option<String>,
        #[arg(long)]
        all: bool,
        #[arg(long)]
        force: bool,
    },
    /// Compare specs, manifests and SVG assets without running D2.
    Status {
        #[arg(long)]
        guide: String,
    },
    /// Remove unreferenced generated D2/SVG files.
    Clean {
        #[arg(long)]
        guide: String,
    },
}

#[derive(Debug, Subcommand)]
enum VisualCommand {
    Compare {
        #[arg(long)]
        reference: PathBuf,
        #[arg(long)]
        current: PathBuf,
        /// Page list such as `1,4,18-22`.
        #[arg(long, default_value = "1")]
        pages: String,
        #[arg(long, default_value_t = 120.0)]
        dpi: f32,
        #[arg(long, default_value = "target/visual")]
        out: PathBuf,
        #[arg(long, default_value_t = 12.0)]
        threshold: f64,
        /// Maximum fraction of changed page pixels (0.02 = 2%).
        #[arg(long, default_value_t = 0.02)]
        ratio_threshold: f64,
        /// Exit non-zero when the threshold is exceeded.
        #[arg(long)]
        fail_on_diff: bool,
    },
}

#[derive(Debug, Subcommand)]
enum TableCommand {
    /// Extract a table from one PDF page; --guide uses layout blocks.
    Extract {
        #[arg(long)]
        source: PathBuf,
        #[arg(long)]
        page: usize,
        /// Read layout table blocks from this guide instead of the legacy
        /// native-text draft extractor.
        #[arg(long)]
        guide: Option<String>,
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
enum StylesCommand {
    /// List style packs, optionally with guide compatibility.
    List {
        /// Optional guide slug for a compatibility column.
        #[arg(long)]
        guide: Option<String>,
    },
    /// List semantic palettes.
    Palettes,
    /// List font pairings.
    Fonts,
    /// Recommend a style and page budget from a reader task.
    Recommend {
        /// exam-summary, exam-revision, concept-summary, study, deep, reference,
        /// practice or one-page.
        #[arg(long)]
        intent: Option<String>,
        /// Structure profile for compatibility filtering.
        #[arg(long)]
        structure: Option<String>,
        /// Domain profile for compatibility filtering.
        #[arg(long)]
        domain: Option<String>,
        /// Optional page budget override, e.g. "5-8".
        #[arg(long)]
        target_pages: Option<String>,
    },
    /// Estimate pages before a full build.
    Estimate {
        #[arg(long)]
        guide: String,
        /// Optional style override; defaults to the guide style.
        #[arg(long)]
        style: Option<String>,
        /// Optional page budget override.
        #[arg(long)]
        target_pages: Option<String>,
    },
    /// Change style/palette/fonts without touching guide content.
    Restyle {
        #[arg(long)]
        guide: String,
        /// Style/layout id.
        #[arg(long)]
        style: String,
        /// Optional palette override.
        #[arg(long)]
        palette: Option<String>,
        /// Optional font-pairing override.
        #[arg(long)]
        fonts: Option<String>,
    },
    /// Show one pack, its tokens and labels.
    Info {
        id: String,
        #[arg(long, default_value = "tr")]
        language: String,
        /// Optional palette override.
        #[arg(long)]
        palette: Option<String>,
        /// Optional font-pairing override.
        #[arg(long)]
        fonts: Option<String>,
        /// Optional domain profile to include.
        #[arg(long)]
        domain: Option<String>,
    },
    /// Render a small style preview PDF.
    Preview {
        id: String,
        #[arg(long, default_value = "tr")]
        language: String,
        /// Optional palette override.
        #[arg(long)]
        palette: Option<String>,
        /// Optional font-pairing override.
        #[arg(long)]
        fonts: Option<String>,
        #[arg(long)]
        out: Option<PathBuf>,
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
    style: folisift_core::StyleProfile,
    content_words: usize,
    prose_words: usize,
    section_count: usize,
    source_section_count: usize,
    empty_sections: Vec<String>,
}

fn audit_options_for(
    workspace: &folisift_core::Workspace,
    style: &folisift_core::StyleProfile,
) -> Result<folisift_parse::AuditOptions, Box<dyn std::error::Error>> {
    let loaded = styles::load_with(
        workspace.root(),
        &style.aesthetic,
        Some(style.palette.as_str()),
        Some(style.fonts.as_str()),
    )?;
    Ok(folisift_parse::AuditOptions::from_tokens(&loaded.tokens))
}

fn write_pdf(path: &std::path::Path, bytes: &[u8]) -> std::io::Result<()> {
    let parent = path.parent().unwrap_or_else(|| std::path::Path::new("."));
    std::fs::create_dir_all(parent)?;
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("output.pdf");
    let temporary = parent.join(format!(".{name}.{}.tmp", std::process::id()));
    std::fs::write(&temporary, bytes)?;
    std::fs::rename(&temporary, path)
}

fn budget_sections(markdown: &str) -> Vec<(String, usize)> {
    let mut sections: Vec<(String, usize)> = Vec::new();
    let mut current: Option<usize> = None;
    for line in markdown.lines() {
        if let Some(heading) = line.strip_prefix("## ") {
            sections.push((heading.trim().to_string(), 0));
            current = Some(sections.len() - 1);
            continue;
        }
        if let Some(index) = current {
            let line = line.trim();
            if !line.is_empty() && !line.starts_with("<!--") {
                sections[index].1 += line.chars().count();
            }
        }
    }
    sections
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

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            let diagnostic = folisift_core::Diagnostic::from_message(&error.to_string());
            if let Ok(json) = serde_json::to_string(&diagnostic) {
                eprintln!("diagnostic: {json}");
            }
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let workspace = Workspace::open(&cli.root)?;

    match cli.command {
        Command::Mcp {
            http,
            http_token,
            profile,
        } => {
            if let Some(address) = http {
                folisift_mcp::serve_http_profile(
                    workspace.root(),
                    &address,
                    http_token.as_deref(),
                    Some(&profile),
                )
                .await?;
            } else {
                folisift_mcp::serve_profile(workspace.root(), Some(&profile)).await?;
            }
        }
        Command::WorkspaceInfo => {
            let guides = list_guides(&workspace)?;
            let value = json!({
                "root": workspace.root().display().to_string(),
                "version": version::VERSION,
                "build": version::BUILD,
                "mcp_tool_schema_version": version::MCP_TOOL_SCHEMA_VERSION,
                "guides": guides.len(),
                "structures": folisift_core::structures::summary(),
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
            let output_ok = std::fs::create_dir_all(&output).is_ok()
                && output
                    .metadata()
                    .map(|m| !m.permissions().readonly())
                    .unwrap_or(false);
            let font_count = folisift_render::embedded_font_count();
            let model = find_model_dir(workspace.root());
            let model_detail = model
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "not installed; `links build` is unavailable".to_string());
            let model_verified = model
                .as_ref()
                .is_some_and(|path| verify_model_dir(path).is_ok());
            let checks = vec![
                json!({"name": "pdfium", "ok": pdfium.is_ok(), "detail": pdfium.as_ref().err().map(ToString::to_string).unwrap_or_else(|| folisift_parse::pdfium::PDFIUM_VERSION.to_string())}),
                json!({"name": "cache", "ok": cache_ok, "detail": cache.display().to_string()}),
                json!({"name": "output", "ok": output_ok, "detail": output.display().to_string()}),
                json!({"name": "guides", "ok": invalid_guides == 0, "detail": format!("{} guide(s), {invalid_guides} invalid", guides.len()), "warning": if guides.is_empty() { Some("no guides yet") } else { None }}),
                json!({"name": "fonts", "ok": font_count > 0, "detail": format!("{font_count} embedded face(s)")}),
                json!({"name": "structures", "ok": !folisift_core::structures::packs().is_empty(), "detail": format!("{} pack(s)", folisift_core::structures::packs().len())}),
                json!({"name": "semantic-model", "ok": true, "available": model.is_some(), "verified": model_verified, "required": false, "detail": model_detail}),
                {
                    let d2 = diagram_cmd::engine_info();
                    json!({"name": "d2", "ok": d2["ok"], "required": d2["required"], "detail": d2["detail"], "path": d2["path"], "version": d2["version"]})
                },
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
            guide,
            strict,
        } => {
            let pdf = workspace.resolve_read(&pdf)?;
            let options = match guide {
                Some(slug) => {
                    let guide = Guide::load(&workspace, &slug)?;
                    audit_options_for(&workspace, &guide.style())?
                }
                None => folisift_parse::AuditOptions::default(),
            };
            let report = folisift_parse::audit_pdf_with(&pdf, expected_pages, options)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            if !report.passed {
                return Err("layout audit failed".into());
            }
            if strict && !report.underfilled_pages.is_empty() {
                let pages = report
                    .underfilled_pages
                    .iter()
                    .map(|page| page.page)
                    .collect::<Vec<_>>();
                return Err(format!("underfilled body pages: {pages:?}").into());
            }
        }
        Command::GuideStatus { slug } => {
            let guide = Guide::load(&workspace, &slug)?;
            let style = guide.style();
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
                style,
                content_words: markdown.split_whitespace().count(),
                prose_words: prose_words(&markdown),
                section_count: body_chars.len(),
                source_section_count: body_chars
                    .keys()
                    .filter(|number| number.chars().next().is_some_and(|c| c.is_ascii_digit()))
                    .count(),
                empty_sections,
            };
            println!("{}", serde_json::to_string_pretty(&status)?);
        }
        Command::Claims { guide, out } => {
            let guide = Guide::load(&workspace, &guide)?;
            let markdown = std::fs::read_to_string(&guide.content)?;
            let report = folisift_core::claims::extract_claims(&markdown);
            let out = match out {
                Some(path) => workspace.resolve_write(&path)?,
                None => guide.dir.join("data/claims.md"),
            };
            report.write_markdown(&out)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "guide": guide.slug,
                    "output": out.display().to_string(),
                    "total": report.total,
                    "referenced": report.referenced,
                    "unreferenced": report.unreferenced,
                    "note": "A citation is traceability, not verification; inspect the cited page.",
                }))?
            );
        }
        Command::Cards {
            guide,
            out,
            review_intervals,
        } => {
            let guide = Guide::load(&workspace, &guide)?;
            let markdown = std::fs::read_to_string(&guide.content)?;
            let intervals = match review_intervals {
                Some(value) => {
                    let values = value
                        .split(',')
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                        .map(|value| value.parse::<u32>())
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(|_| {
                            std::io::Error::new(
                                std::io::ErrorKind::InvalidInput,
                                "review intervals must be comma-separated positive days",
                            )
                        })?;
                    folisift_core::cards::normalize_review_intervals(values).map_err(|error| {
                        std::io::Error::new(std::io::ErrorKind::InvalidInput, error)
                    })?
                }
                None => folisift_core::cards::normalize_review_intervals(
                    guide
                        .config
                        .review_intervals
                        .clone()
                        .unwrap_or_else(|| vec![1, 3, 7, 14, 30]),
                )
                .unwrap_or_default(),
            };
            let cards =
                folisift_core::cards::cards_from_markdown(&markdown, &guide.slug, &intervals);
            let out = match out {
                Some(path) => workspace.resolve_write(&path)?,
                None => guide.dir.join("data/cards.csv"),
            };
            if let Some(parent) = out.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&out, folisift_core::cards::to_csv(&cards))?;
            let warning = if cards.is_empty() {
                Some("no recall/answer pairs found; add ::: recall and ::: answer blocks")
            } else {
                None
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "guide": guide.slug,
                    "output": out.display().to_string(),
                    "cards": cards.len(),
                    "review_intervals_days": intervals,
                    "warning": warning,
                }))?
            );
        }
        Command::Links { command } => {
            match command {
                LinksCommand::Build {
                    guide,
                    top_k,
                    min_cosine,
                    min_score,
                    explore,
                    no_model,
                    model_dir,
                } => {
                    let guide = Guide::load(&workspace, &guide)?;
                    let layouts = read_layouts(&guide.dir.join("data/blocks"))?;
                    if layouts.is_empty() {
                        return Err("no layout blocks found; run `folisift analyze` or `prepare_guide` first".into());
                    }
                    let (top_k, min_cosine, min_score) = if explore {
                        (8, 0.65, 0.45)
                    } else {
                        (top_k, min_cosine, min_score)
                    };
                    let options = LinkOptions {
                        top_k,
                        min_cosine,
                        min_score,
                        ..LinkOptions::default()
                    };
                    let sections = layouts
                        .iter()
                        .flat_map(|layout| sections_from_layout(layout, options.max_section_chars))
                        .collect::<Vec<_>>();
                    if sections.is_empty() {
                        return Err("layout blocks produced no sections".into());
                    }
                    let model =
                        if no_model {
                            None
                        } else {
                            let dir = model_dir
                        .or_else(|| find_model_dir(workspace.root()))
                        .ok_or("model directory not found; set FOLISIFT_MODEL_DIR or --model-dir")?;
                            Some(Model2VecEmbedder::load(&dir)?)
                        };
                    let model_name = model.as_ref().map(|_| DEFAULT_MODEL.to_string());
                    let report = LinkIndex::build(
                        &sections,
                        model.as_ref().map(|model| model as &dyn Embedder),
                        model_name,
                        &options,
                    )?;
                    let out = guide.dir.join("data/document-links.json");
                    write_report(&out, &report)?;
                    write_markdown_report(&guide.dir.join("data/document-links.md"), &report)?;
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&json!({
                            "guide": guide.slug,
                            "output": out.display().to_string(),
                            "sections": report.sections.len(),
                            "links": report.links.len(),
                            "model": report.model,
                            "blocks": layouts.len(),
                        }))?
                    );
                }
                LinksCommand::Read {
                    guide,
                    section,
                    limit,
                } => {
                    let guide = Guide::load(&workspace, &guide)?;
                    let report = read_report(&guide.dir.join("data/document-links.json"))?;
                    let links = report
                        .links
                        .iter()
                        .filter(|link| {
                            section
                                .as_deref()
                                .is_none_or(|section| link.from == section || link.to == section)
                        })
                        .take(limit)
                        .map(|link| {
                            let from_heading = report
                                .sections
                                .iter()
                                .find(|section| section.id == link.from)
                                .and_then(|section| section.heading.clone());
                            let to_heading = report
                                .sections
                                .iter()
                                .find(|section| section.id == link.to)
                                .and_then(|section| section.heading.clone());
                            json!({
                                "from": link.from,
                                "from_heading": from_heading,
                                "from_source": link.from_source,
                                "from_page": link.from_page,
                                "to": link.to,
                                "to_heading": to_heading,
                                "to_source": link.to_source,
                                "to_page": link.to_page,
                                "relation": link.relation,
                                "direction": link.direction,
                                "confidence": link.confidence,
                                "cosine": link.cosine,
                                "lexical": link.lexical,
                                "score": link.score,
                            })
                        })
                        .collect::<Vec<_>>();
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&json!({
                            "guide": guide.slug,
                            "model": report.model,
                            "total": report.links.len(),
                            "returned": links.len(),
                            "links": links,
                        }))?
                    );
                }
                LinksCommand::Status { guide } => {
                    let guide = Guide::load(&workspace, &guide)?;
                    let blocks = read_layouts(&guide.dir.join("data/blocks"))?;
                    let links_path = guide.dir.join("data/document-links.json");
                    let links = read_report(&links_path).ok();
                    let model = find_model_dir(workspace.root());
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&json!({
                            "guide": guide.slug,
                            "blocks": blocks.iter().map(|layout| json!({
                                "source": layout.source,
                                "pages": layout.page_count,
                            })).collect::<Vec<_>>(),
                            "blocks_ready": !blocks.is_empty(),
                            "model": model.as_ref().map(|path| path.display().to_string()),
                            "model_ready": model.is_some(),
                            "model_verified": model.as_ref().is_some_and(|path| verify_model_dir(path).is_ok()),
                            "links": links.as_ref().map(|report| report.links.len()),
                            "links_ready": links.is_some(),
                            "output": links_path.display().to_string(),
                        }))?
                    );
                }
            }
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
            let blocks_dir = out
                .parent()
                .unwrap_or_else(|| std::path::Path::new("data"))
                .join("blocks");
            let analysis = analyze_sources_with_layout(&sources, low_threshold).await?;
            let inventory = analysis.inventory;
            inventory.write_files(&out, &md)?;
            let review_path = out
                .parent()
                .unwrap_or_else(|| std::path::Path::new("data"))
                .join("headings.review.md");
            std::fs::write(&review_path, inventory.headings_review())?;
            let block_paths = write_layouts(&analysis.layouts, &blocks_dir)?;
            let cache = SourceCache::new(workspace.root().join("data/cache"));
            let mut texts = Vec::new();
            for source in &sources {
                let text = cache.load(source, false).await?;
                texts.push(json!({
                    "source": source.display().to_string(),
                    "text": text.cache_path.display().to_string(),
                    "blake3": text.blake3,
                    "cache_hit": text.cache_hit,
                }));
            }
            let value = json!({
                "inventory": out.display().to_string(),
                "markdown": md.display().to_string(),
                "headings_review": review_path.display().to_string(),
                "blocks": block_paths
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect::<Vec<_>>(),
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
            style,
            structure,
            language,
            domain,
            palette,
            fonts,
            intent,
            target_pages,
        } => {
            let sources = sources
                .iter()
                .map(|path| workspace.resolve_read(path))
                .collect::<Result<Vec<_>, _>>()?;
            let recommendation = intent.as_deref().and_then(folisift_core::intent::recommend);
            let style = if style == "paper" && recommendation.is_some() {
                recommendation
                    .as_ref()
                    .map_or("paper", |item| item.style.as_str())
                    .to_string()
            } else {
                style
            };
            let structure = if structure == "a-g" && recommendation.is_some() {
                recommendation
                    .as_ref()
                    .map_or("a-g", |item| item.structure.as_str())
                    .to_string()
            } else {
                structure
            };
            let title_for_content = title.clone();
            let style = folisift_core::StyleProfile {
                aesthetic: style,
                structure,
                language: language.clone(),
                source_language: language.clone(),
                domain,
                palette: palette.unwrap_or_default(),
                fonts: fonts.unwrap_or_default(),
                ..folisift_core::StyleProfile::default()
            };
            let loaded_style = styles::load_with(
                workspace.root(),
                &style.aesthetic,
                Some(style.palette.as_str()),
                Some(style.fonts.as_str()),
            )?;
            styles::validate_profile(&loaded_style.pack, &style)?;
            let target_pages = target_pages.or_else(|| {
                intent
                    .as_deref()
                    .and_then(folisift_core::intent::recommend)
                    .map(|recommendation| recommendation.target_pages)
            });
            let guide = scaffold(
                &workspace,
                &ScaffoldOptions {
                    slug,
                    title,
                    course,
                    sources: sources.clone(),
                    style: style.clone(),
                    output_intent: intent,
                    target_pages,
                },
            )?;
            let analysis =
                analyze_sources_with_layout(&sources, folisift_parse::LOW_TEXT_THRESHOLD).await?;
            let inventory = analysis.inventory;
            write_layouts(&analysis.layouts, &guide.dir.join("data/blocks"))?;
            let source_ids = sources
                .iter()
                .filter_map(|path| path.file_stem().and_then(|value| value.to_str()))
                .collect::<Vec<_>>()
                .join(", ");
            let mut body = format!(
                "---\ntitle: {title_for_content:?}\nrunning: {title_for_content:?}\nlanguage: {:?}\nsources: [{source_ids}]\n---\n\n",
                style.language
            );
            if style.language == "en" {
                body.push_str(
                    "::: front kicker=\"Front matter · Reading note\" title=\"About this guide\" lead=\"This guide turns source slides into an exam-oriented study text.\"\nThis opening will be written by the host LLM.\n:::\n\n::: toc title=\"Guide map\" note=\"Page numbers refer to this PDF.\"\n:::\n\n",
                );
            } else {
                body.push_str(
                    "::: front kicker=\"Ön kısım · Okuma notu\" title=\"Bu kitapçık hakkında\" lead=\"Bu kılavuz, kaynak slaytlardan sınav odaklı bir çalışma metni üretir.\"\nBu bölüm host LLM tarafından yazılacaktır.\n:::\n\n::: toc title=\"Kılavuzun haritası\" note=\"Sayfa numaraları bu PDF’e aittir.\"\n:::\n\n",
                );
            }
            if style.structure == "irac" {
                let mut headings = String::from("# Source headings\n\n");
                for source in &inventory.sources {
                    headings.push_str(&format!(
                        "## {}\n\n",
                        source.filename.trim_end_matches(".pdf")
                    ));
                    for heading in &source.headings {
                        headings
                            .push_str(&format!("- {} (s. {})\n", heading.heading, heading.page));
                    }
                    headings.push('\n');
                }
                std::fs::write(guide.dir.join("source-headings.md"), headings)?;
                for case in 1..=10 {
                    if style.language == "en" {
                        body.push_str(&format!(
                            "## Case {case}\n\n### Issue\n\nDescribe the legal issue. [PD0, p. 1]\n\n### Rule\n\nState the applicable rule. [PD0, p. 1]\n\n### Application\n\nApply the rule to the facts. [PD0, p. 1]\n\n### Conclusion\n\nGive the reasoned conclusion. [PD0, p. 1]\n\n"
                        ));
                    } else {
                        body.push_str(&format!(
                            "## Dava {case}\n\n### Sorun\n\nÇözülecek hukuki soruyu yaz. [PD0, s. 1]\n\n### Kural\n\nUygulanacak norm ve içtihadı yaz. [PD0, s. 1]\n\n### Uygulama\n\nKuralı olaya uygula. [PD0, s. 1]\n\n### Sonuç\n\nGerekçeli sonucu yaz. [PD0, s. 1]\n\n"
                        ));
                    }
                }
            } else {
                for (index, source) in inventory.sources.iter().enumerate() {
                    let id = source.filename.trim_end_matches(".pdf");
                    let name = source.title_or_filename().replace('"', "'");
                    body.push_str(&format!(
                    "::: chapter part=\"{part}\" side=\"{side}: {id}\" kicker=\"{kicker} {index}\" title=\"{name}\" toc=\"{name}\" running=\"{name}\" subs=true\n:::\n\n",
                    part = if style.language == "en" { "B · DETAILED REVIEW" } else { "B · AYRINTILI İNCELEME" },
                    side = if style.language == "en" { "SOURCE" } else { "KAYNAK" },
                    kicker = if style.language == "en" { "CHAPTER" } else { "BÖLÜM" },
                ));
                    for heading in &source.headings {
                        let number = heading
                            .heading
                            .split_whitespace()
                            .next()
                            .unwrap_or_default();
                        if number.matches('.').count() == 1
                            && number.chars().next().is_some_and(|c| c.is_ascii_digit())
                        {
                            let source_label = if style.language == "en" {
                                "Source"
                            } else {
                                "Kaynak"
                            };
                            body.push_str(&format!(
                                "## {}\n\n{source_label}: [{}]\n\n",
                                heading.heading, source.filename
                            ));
                        }
                    }
                }
            }
            let required =
                folisift_core::structures::scaffold_sections(&style.structure, &style.language);
            if !required.is_empty() && style.structure != "irac" {
                if style.language == "en" {
                    body.push_str("::: chapter part=\"C · STRUCTURE\" side=\"FRAMEWORK\" kicker=\"C\" title=\"Structure checklist\" toc=\"Structure checklist\" running=\"Structure checklist\" subs=false\n:::\n\n");
                } else {
                    body.push_str("::: chapter part=\"C · YAPI\" side=\"ÇERÇEVE\" kicker=\"C\" title=\"Yapı denetim listesi\" toc=\"Yapı denetim listesi\" running=\"Yapı denetim listesi\" subs=false\n:::\n\n");
                }
                for (section, hint, level) in required {
                    let hint = if style.language == "en" {
                        "Source content goes here.".to_string()
                    } else {
                        hint
                    };
                    body.push_str(&format!(
                        "{} {section}\n\n{hint}\n\n",
                        "#".repeat(usize::from(level))
                    ));
                }
            }
            if style.language == "en" {
                body.push_str("<!-- audit -->\n::: chapter part=\"AUDIT\" side=\"QUALITY\" kicker=\"AUDIT\" title=\"Coverage audit\" toc=\"Coverage audit\" running=\"Coverage audit\" subs=false\n:::\n\n## Coverage\n\nSource coverage and limitations will be reported here.\n");
            } else {
                body.push_str("<!-- audit -->\n::: chapter part=\"DENETİM\" side=\"KALİTE\" kicker=\"DENETİM\" title=\"Kapsam denetimi\" toc=\"Kapsam denetimi\" running=\"Kapsam denetimi\" subs=false\n:::\n\n## Kapsam\n\nKaynak kapsamı ve sınırlamalar burada raporlanacaktır.\n");
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
        Command::Styles { command } => match command {
            StylesCommand::List { guide } => {
                if let Some(slug) = guide {
                    let guide = Guide::load(&workspace, &slug)?;
                    let profile = guide.style();
                    let candidates = styles::compatibility(
                        workspace.root(),
                        &profile.structure,
                        &profile.domain,
                    )?;
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&json!({
                            "guide": guide.slug,
                            "structure": profile.structure,
                            "domain": profile.domain,
                            "styles": candidates,
                        }))?
                    );
                } else {
                    let packs = styles::list(workspace.root())?;
                    println!("{}", serde_json::to_string_pretty(&packs)?);
                }
            }
            StylesCommand::Palettes => {
                let palettes = styles::list_palettes(workspace.root())?;
                println!("{}", serde_json::to_string_pretty(&palettes)?);
            }
            StylesCommand::Fonts => {
                let fonts = styles::list_fonts(workspace.root())?;
                println!("{}", serde_json::to_string_pretty(&fonts)?);
            }
            StylesCommand::Recommend {
                intent,
                structure,
                domain,
                target_pages,
            } => {
                if let (Some(structure), Some(domain)) = (structure.as_deref(), domain.as_deref()) {
                    let candidates = styles::compatibility(workspace.root(), structure, domain)?;
                    let chosen = candidates.iter().find(|item| item.compatible).cloned();
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&json!({
                            "structure": structure,
                            "domain": domain,
                            "target_pages": target_pages,
                            "chosen": chosen,
                            "candidates": candidates,
                        }))?
                    );
                    return Ok(());
                }
                let intent = intent
                    .ok_or("recommend requires either --intent or --structure and --domain")?;
                let mut recommendation =
                    folisift_core::intent::recommend(&intent).ok_or_else(|| {
                        format!(
                            "unknown output intent {intent:?}; use one of {:?}",
                            folisift_core::intent::OUTPUT_INTENTS
                        )
                    })?;
                if let Some(target) = target_pages {
                    folisift_core::config::validate_target_pages(&target)?;
                    recommendation.target_pages = target;
                }
                println!("{}", serde_json::to_string_pretty(&recommendation)?);
            }
            StylesCommand::Estimate {
                guide,
                style,
                target_pages,
            } => {
                let guide = Guide::load(&workspace, &guide)?;
                let markdown = std::fs::read_to_string(&guide.content)?;
                let style = style.unwrap_or_else(|| guide.style().aesthetic.clone());
                let target = target_pages.or_else(|| guide.config.target_pages.clone());
                let estimate = styles::estimate(&style, &markdown, target.as_deref());
                println!("{}", serde_json::to_string_pretty(&estimate)?);
            }
            StylesCommand::Restyle {
                guide,
                style,
                palette,
                fonts,
            } => {
                let guide = Guide::load(&workspace, &guide)?;
                let before = std::fs::read_to_string(&guide.content)?;
                let loaded = styles::load_with(
                    workspace.root(),
                    &style,
                    palette.as_deref(),
                    fonts.as_deref(),
                )?;
                let mut profile = guide.style();
                profile.aesthetic = style.clone();
                profile.palette = loaded.palette.id.clone();
                profile.fonts = loaded.fonts.id.clone();
                styles::validate_profile(&loaded.pack, &profile)?;
                let config_path = guide.dir.join("config.json");
                let mut config: Value =
                    serde_json::from_str(&std::fs::read_to_string(&config_path)?)?;
                config["style"] = serde_json::to_value(&profile)?;
                config["theme"] = Value::String(style.clone());
                config["style_source"] = Value::String("explicit".to_string());
                config["style_rationale"] = Value::String(format!(
                    "Restyle chose {}; palette {} and fonts {} were validated against the pack.",
                    loaded.pack.name, loaded.palette.id, loaded.fonts.id
                ));
                std::fs::write(
                    &config_path,
                    serde_json::to_string_pretty(&config)?
                        + "
",
                )?;
                let after = std::fs::read_to_string(&guide.content)?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "guide": guide.slug,
                        "style": style,
                        "palette": loaded.palette.id,
                        "fonts": loaded.fonts.id,
                        "content_unchanged": before == after,
                        "content_hash_before": folisift_render::content_hash(&before),
                        "content_hash_after": folisift_render::content_hash(&after),
                    }))?
                );
            }
            StylesCommand::Info {
                id,
                language,
                palette,
                fonts,
                domain,
            } => {
                let loaded =
                    styles::load_with(workspace.root(), &id, palette.as_deref(), fonts.as_deref())?;
                let strings = styles::strings(workspace.root(), &id, &language)?;
                let domain = domain
                    .map(|domain| folisift_render::domains::load(workspace.root(), &domain))
                    .transpose()?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "pack": loaded.pack,
                        "palette": loaded.palette,
                        "fonts": loaded.fonts,
                        "tokens": loaded.tokens,
                        "strings": strings,
                        "domain": domain,
                    }))?
                );
            }
            StylesCommand::Preview {
                id,
                language,
                palette,
                fonts,
                out,
            } => {
                let loaded =
                    styles::load_with(workspace.root(), &id, palette.as_deref(), fonts.as_deref())?;
                let strings = styles::strings(workspace.root(), &id, &language)?;
                let content = styles::preview(workspace.root(), &id, &language)?;
                let style = folisift_core::StyleProfile {
                    aesthetic: loaded.pack.aesthetic.clone(),
                    palette: loaded.palette.id.clone(),
                    fonts: loaded.fonts.id.clone(),
                    structure: loaded
                        .pack
                        .supported_structures
                        .first()
                        .cloned()
                        .unwrap_or_else(|| "a-g".to_string()),
                    language: language.clone(),
                    source_language: language.clone(),
                    voice: loaded
                        .pack
                        .supported_voices
                        .first()
                        .cloned()
                        .unwrap_or_else(|| "academic".to_string()),
                    study_mode: "exam".to_string(),
                    domain: loaded
                        .pack
                        .supported_domains
                        .first()
                        .cloned()
                        .unwrap_or_else(|| "general".to_string()),
                    citation: "page-ref".to_string(),
                };
                folisift_core::structures::validate(&style.structure, &language, &content)?;
                if matches!(
                    style.structure.as_str(),
                    "lecture-notes" | "workbook" | "formula-sheet"
                ) {
                    let issues = folisift_core::gates::genre_issues(&style.structure, &content);
                    if !issues.is_empty() {
                        return Err(format!(
                            "style preview genre gate failed:\n- {}",
                            issues.join("\n- ")
                        )
                        .into());
                    }
                }
                let output = out.unwrap_or_else(|| {
                    PathBuf::from(format!("output/style-preview-{id}-{language}.pdf"))
                });
                let output = workspace.resolve_write(&output)?;
                let rendered = render(
                    &content,
                    &RenderOptions {
                        guide_root: workspace.root(),
                        title: &loaded.pack.name,
                        subtitle: "",
                        course: "Folisift",
                        language: &language,
                        sources: &[],
                        style: Some(&style),
                        styles_root: Some(workspace.root()),
                        cache_dir: Some(&workspace.root().join("data/cache/build")),
                    },
                )?;
                write_pdf(&output, &rendered.pdf)?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "style": id,
                        "language": language,
                        "toc": strings.toc,
                        "figure": strings.figure,
                        "output": output.display().to_string(),
                        "pages": rendered.pages,
                    }))?
                );
            }
        },
        Command::Budget {
            guide,
            target_pages,
        } => {
            let guide = Guide::load(&workspace, &guide)?;
            let markdown = std::fs::read_to_string(&guide.content)?;
            let sections = budget_sections(&markdown);
            let total = sections
                .iter()
                .map(|(_, chars)| *chars)
                .sum::<usize>()
                .max(1);
            let target = target_pages
                .or_else(|| guide.config.target_pages.clone())
                .unwrap_or_else(|| "12".to_string());
            let target_max = target
                .split('-')
                .next_back()
                .and_then(|value| value.trim().parse::<usize>().ok())
                .unwrap_or(12)
                .max(1);
            let rows = sections
                .iter()
                .map(|(heading, chars)| {
                    let pages = ((*chars as f64 / total as f64) * target_max as f64)
                        .round()
                        .max(1.0) as usize;
                    json!({
                        "heading": heading,
                        "body_chars": chars,
                        "estimated_pages": pages,
                        "target_words": pages * 400,
                    })
                })
                .collect::<Vec<_>>();
            let largest = sections
                .iter()
                .max_by_key(|(_, chars)| *chars)
                .map(|(heading, chars)| {
                    json!({
                        "heading": heading,
                        "body_chars": chars,
                        "suggestion": "largest section; cut or split it first if the build exceeds the target",
                    })
                });
            let estimated = rows
                .iter()
                .filter_map(|row| row["estimated_pages"].as_u64())
                .sum::<u64>() as usize;
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "guide": guide.slug,
                    "target_pages": target,
                    "estimated_pages": estimated,
                    "over_budget": estimated > target_max,
                    "sections": rows,
                    "largest": largest,
                }))?
            );
        }
        Command::Qa { guide } => {
            let report = folisift_build::qa(&workspace, &guide).await?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            if !report["passed"].as_bool().unwrap_or(false) {
                return Err("QA gate failed".into());
            }
        }
        Command::Build {
            guide,
            content,
            output,
            check,
        } => {
            let outcome = build_guide(
                &workspace,
                folisift_build::BuildRequest {
                    slug: guide,
                    content,
                    output,
                    check,
                },
            )
            .await?;
            let coverage = outcome.coverage.as_ref().map(|report| {
                json!({
                    "passed": report.passed,
                    "total": report.total,
                    "canonical_total": report.canonical_total,
                    "claimed": report.claimed,
                    "gate_total": report.gate_total,
                    "missing": report.missing,
                    "missing_from_gate": report.missing_from_gate,
                    "empty_sections": report.empty_sections.len(),
                    "weak_sections": report.weak_sections.len(),
                    "duplicate_sections": report.duplicate_sections,
                })
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "output": outcome.output.display().to_string(),
                    "passed": outcome.passed,
                    "pages": outcome.rendered.pages,
                    "content_hash": outcome.rendered.content_hash,
                    "cache_hit": outcome.rendered.cache_hit,
                    "style": outcome.style,
                    "incremental": outcome.incremental,
                    "warnings": outcome.rendered.warnings.len(),
                    "revision_coverage": outcome.revision_coverage.then_some(true),
                    "coverage": coverage,
                }))?
            );
        }
        Command::Visual { command } => match command {
            VisualCommand::Compare {
                reference,
                current,
                pages,
                dpi,
                out,
                threshold,
                ratio_threshold,
                fail_on_diff,
            } => {
                let reference = workspace.resolve_read(&reference)?;
                let current = workspace.resolve_read(&current)?;
                let pages = folisift_parse::parse_page_list(&pages)?;
                let out = workspace.resolve_write(&out)?;
                let report = folisift_parse::compare_pdfs(
                    &reference,
                    &current,
                    &pages.iter().map(|page| *page as usize).collect::<Vec<_>>(),
                    dpi.clamp(72.0, 200.0),
                    &out,
                    threshold,
                    ratio_threshold.clamp(0.0, 1.0),
                )?;
                println!("{}", serde_json::to_string_pretty(&report)?);
                if fail_on_diff && !report.passed {
                    return Err("visual comparison exceeded the threshold".into());
                }
            }
        },
        Command::SectionBrief {
            guide,
            source,
            heading,
            max_chars,
        } => {
            let guide = Guide::load(&workspace, &guide)?;
            let layout = folisift_parse::SourceLayout::read_json(
                &guide.dir.join("data/blocks").join(format!("{source}.json")),
            )?;
            let brief = folisift_parse::section_brief(&layout, &heading, max_chars)
                .ok_or_else(|| format!("section {heading:?} not found in {source}"))?;
            println!("{}", serde_json::to_string_pretty(&brief)?);
        }
        Command::ReadSection {
            guide,
            source,
            heading,
            max_chars,
            offset,
        } => {
            let guide = Guide::load(&workspace, &guide)?;
            let layout = folisift_parse::SourceLayout::read_json(
                &guide.dir.join("data/blocks").join(format!("{source}.json")),
            )?;
            let read = folisift_parse::read_section(&layout, &heading, max_chars, offset)
                .ok_or_else(|| format!("section {heading:?} not found in {source}"))?;
            println!("{}", serde_json::to_string_pretty(&read)?);
        }
        Command::CoverageManifest { guide, action } => {
            let guide = Guide::load(&workspace, &guide)?;
            let path = guide.dir.join("data/coverage-manifest.json");
            match action.as_str() {
                "build" => {
                    let inventory = folisift_parse::analyze_sources(
                        &guide.sources,
                        folisift_parse::LOW_TEXT_THRESHOLD,
                    )
                    .await?;
                    let markdown = std::fs::read_to_string(&guide.content)?;
                    let manifest = folisift_parse::CoverageManifest::build(&inventory, &markdown);
                    manifest.write_json(&path)?;
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&json!({
                            "guide": guide.slug,
                            "output": path.display().to_string(),
                            "rows": manifest.rows.len(),
                        }))?
                    );
                }
                "read" => {
                    let manifest = folisift_parse::CoverageManifest::read_json(&path)?;
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&json!({
                            "guide": guide.slug,
                            "output": path.display().to_string(),
                            "rows": manifest.rows.len(),
                            "manifest": manifest,
                        }))?
                    );
                }
                "validate" => {
                    let inventory = folisift_parse::analyze_sources(
                        &guide.sources,
                        folisift_parse::LOW_TEXT_THRESHOLD,
                    )
                    .await?;
                    let markdown = std::fs::read_to_string(&guide.content)?;
                    let manifest = folisift_parse::CoverageManifest::read_json(&path)?;
                    let report = manifest.report(&inventory, &markdown);
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&json!({
                            "guide": guide.slug,
                            "passed": report.passed,
                            "total": report.total,
                            "claimed": report.claimed,
                            "missing": report.missing_from_gate,
                            "empty_sections": report.empty_sections,
                        }))?
                    );
                    if !report.passed {
                        std::process::exit(1);
                    }
                }
                other => {
                    return Err(format!(
                        "unknown coverage_manifest action {other:?}; use build, read or validate"
                    )
                    .into());
                }
            }
        }
        Command::Graph { command } => match command {
            GraphCommand::Build { guide } => {
                let guide = Guide::load(&workspace, &guide)?;
                let layouts = read_layouts(&guide.dir.join("data/blocks"))?;
                if layouts.is_empty() {
                    return Err(
                        "no layout blocks found; run `folisift analyze` or `prepare_guide` first"
                            .into(),
                    );
                }
                let mut graph = DocumentGraph::build(&layouts);
                if let Ok(markdown) = std::fs::read_to_string(&guide.content) {
                    graph.add_citations(&markdown);
                }
                let out = guide.dir.join("data/document-graph.json");
                graph.write_json(&out)?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "guide": guide.slug,
                        "output": out.display().to_string(),
                        "sources": layouts.len(),
                        "nodes": graph.nodes.len(),
                        "edges": graph.edges.len(),
                        "node_kinds": graph.counts(),
                    }))?
                );
            }
            GraphCommand::Status { guide } => {
                let guide = Guide::load(&workspace, &guide)?;
                let path = guide.dir.join("data/document-graph.json");
                let graph = DocumentGraph::read_json(&path)?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "guide": guide.slug,
                        "output": path.display().to_string(),
                        "nodes": graph.nodes.len(),
                        "edges": graph.edges.len(),
                        "node_kinds": graph.counts(),
                    }))?
                );
            }
            GraphCommand::Query {
                guide,
                query,
                limit,
            } => {
                let guide = Guide::load(&workspace, &guide)?;
                let graph = DocumentGraph::read_json(&guide.dir.join("data/document-graph.json"))?;
                let hits = graph.query(&query, limit);
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "guide": guide.slug,
                        "query": query,
                        "total": hits.len(),
                        "nodes": hits,
                    }))?
                );
            }
            GraphCommand::Path {
                guide,
                from,
                to,
                max_depth,
            } => {
                let guide = Guide::load(&workspace, &guide)?;
                let graph = DocumentGraph::read_json(&guide.dir.join("data/document-graph.json"))?;
                let path = graph.path(&from, &to, max_depth);
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "guide": guide.slug,
                        "from": from,
                        "to": to,
                        "found": !path.is_empty(),
                        "path": path,
                    }))?
                );
            }
            GraphCommand::Explain { guide, node, limit } => {
                let guide = Guide::load(&workspace, &guide)?;
                let graph = DocumentGraph::read_json(&guide.dir.join("data/document-graph.json"))?;
                let value = graph
                    .explain(&node, limit)
                    .ok_or_else(|| format!("graph node {node:?} not found"))?;
                println!("{}", serde_json::to_string_pretty(&value)?);
            }
        },
        Command::Diagram { command } => match command {
            DiagramCommand::Validate { guide, id, all } => {
                let value = diagram_cmd::validate(&workspace, &guide, id.as_deref(), all)?;
                println!("{}", serde_json::to_string_pretty(&value)?);
            }
            DiagramCommand::Build {
                guide,
                id,
                all,
                force,
            } => {
                let value = diagram_cmd::build(&workspace, &guide, id.as_deref(), all, force)?;
                println!("{}", serde_json::to_string_pretty(&value)?);
            }
            DiagramCommand::Status { guide } => {
                let value = diagram_cmd::status(&workspace, &guide)?;
                println!("{}", serde_json::to_string_pretty(&value)?);
            }
            DiagramCommand::Clean { guide } => {
                let value = diagram_cmd::clean(&workspace, &guide)?;
                println!("{}", serde_json::to_string_pretty(&value)?);
            }
        },
        Command::Table { command } => match command {
            TableCommand::Extract {
                source,
                page,
                guide,
                out,
            } => {
                let source = workspace.resolve_read(&source)?;
                let value = if let Some(slug) = guide {
                    let guide = Guide::load(&workspace, &slug)?;
                    let source_id = source
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or_default()
                        .trim_end_matches(".pdf");
                    let layout = folisift_parse::SourceLayout::read_json(
                        &guide
                            .dir
                            .join("data/blocks")
                            .join(format!("{source_id}.json")),
                    )?;
                    let tables = layout
                        .pages
                        .iter()
                        .filter(|page_layout| page_layout.page == page)
                        .flat_map(|page_layout| page_layout.blocks.iter())
                        .filter(|block| block.kind.is_table())
                        .map(|block| json!(block))
                        .collect::<Vec<_>>();
                    json!({
                        "source": source.display().to_string(),
                        "page": page,
                        "mode": "layout",
                        "tables": tables,
                    })
                } else {
                    let tables = folisift_parse::extract_tables(&source, page)?;
                    json!({
                        "source": source.display().to_string(),
                        "page": page,
                        "mode": "native-draft",
                        "tables": tables,
                    })
                };
                if let Some(out) = out {
                    let out = workspace.resolve_write(out)?;
                    std::fs::write(&out, serde_json::to_string_pretty(&value)?)?;
                    println!("{}", out.display());
                } else {
                    println!("{}", serde_json::to_string_pretty(&value)?);
                }
            }
        },
        Command::ReadSource {
            source,
            pages,
            max_chars,
            offset,
        } => {
            let source = workspace.resolve_read(source)?;
            let cache = SourceCache::new(workspace.root().join("data/cache"));
            let result = read_source_at(&cache, &source, Some(&pages), max_chars, offset).await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        Command::ReadBlocks {
            guide,
            source,
            pages,
            max_blocks,
        } => {
            let guide = Guide::load(&workspace, &guide)?;
            let pages = pages.as_deref().map(parse_page_list).transpose()?;
            let view = read_block_view(
                &guide.dir.join("data/blocks"),
                &source,
                pages.as_deref(),
                max_blocks,
            )?;
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "guide": guide.slug,
                    "blocks": view,
                }))?
            );
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
            crop,
        } => {
            let source = workspace.resolve_read(source)?;
            let out = workspace.resolve_write(out)?;
            let pages = pages.as_deref().map(parse_page_list).transpose()?;
            let crop = crop
                .as_deref()
                .map(folisift_parse::CropBox::parse)
                .transpose()?;
            let result = render_pages(&source, pages, &out, contact_sheet, dpi, crop).await?;
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
