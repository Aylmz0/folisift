//! Markdown + directives to Typst PDF.
//!
//! The content format is deliberately small: CommonMark, YAML frontmatter and
//! four block directives. Guides never contain raw Typst.

pub mod diagram;
pub mod domains;
pub mod error;
pub mod markdown;
pub mod styles;
pub mod world;

use std::path::Path;

use typst::{World, WorldExt, compile};
use typst_layout::PagedDocument;

pub use error::{Error, Result};
pub use markdown::{
    Frontmatter, markdown_to_typst, markdown_to_typst_styled, markdown_to_typst_styled_with_table,
    parse_document,
};
pub use styles::{LoadedStyle, StylePack, StyleStrings};

/// Stable content hash used for cross-style parity checks.
pub fn content_hash(content: &str) -> String {
    blake3::hash(content.as_bytes()).to_hex().to_string()
}

/// Number of font faces compiled into the renderer.
pub fn embedded_font_count() -> usize {
    world::font_count()
}
use world::TypstWorld;

const PAPER_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../templates/paper.typ"
));
const CLEAN_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../templates/clean.typ"
));

pub struct RenderOptions<'a> {
    /// Directory containing `assets/` and the generated Typst source.
    pub guide_root: &'a Path,
    pub title: &'a str,
    pub subtitle: &'a str,
    pub course: &'a str,
    pub language: &'a str,
    pub sources: &'a [String],
    /// Style axes from the guide config; `None` means paper defaults.
    pub style: Option<&'a folisift_core::StyleProfile>,
    /// Workspace root for `styles/` overrides; built-ins work without it.
    pub styles_root: Option<&'a Path>,
    /// Optional build-cache directory. Cache failures never fail a build.
    pub cache_dir: Option<&'a Path>,
}

#[derive(Debug, Clone)]
pub struct Rendered {
    pub pdf: Vec<u8>,
    pub pages: usize,
    pub content_hash: String,
    pub cache_hit: bool,
    pub warnings: Vec<String>,
}

/// Render one guide Markdown document to PDF bytes.
pub fn render(content: &str, options: &RenderOptions<'_>) -> Result<Rendered> {
    let (frontmatter, body) = parse_document(content)?;
    let profile = options.style.cloned().unwrap_or_default();
    profile.validate().map_err(Error::StyleConfig)?;
    let styles_root = options.styles_root.unwrap_or(Path::new(""));
    let loaded = styles::load(styles_root, &profile.aesthetic)?;
    styles::validate_profile(&loaded.pack, &profile)?;
    let strings = styles::strings(styles_root, &loaded.pack.id, &profile.language)?;
    let cache_key = build_cache_key(content, &profile, &loaded, &strings);
    if let Some(cache_dir) = options.cache_dir
        && let Some(cached) = load_cached(cache_dir, &cache_key)
    {
        return Ok(cached);
    }
    let diagram_theme = diagram::DiagramTheme::from_tokens(&loaded.tokens, "Inter");
    let body = diagram::compile_body(&body, options.guide_root, &diagram_theme)?;
    let body =
        markdown_to_typst_styled_with_table(&body, &strings.figure, &strings.toc, &strings.table)?;
    let template = match loaded.pack.layout.as_str() {
        "paper" => PAPER_TEMPLATE,
        "clean" => CLEAN_TEMPLATE,
        other => {
            return Err(Error::Style {
                id: loaded.pack.id.clone(),
                message: format!("unsupported layout family {other:?}"),
            });
        }
    };
    let template = styles::apply_tokens(template, &loaded.tokens, &loaded.pack.layout)?;

    let title = frontmatter.title.as_deref().unwrap_or(options.title);
    let subtitle = frontmatter.subtitle.as_deref().unwrap_or(options.subtitle);
    let course = frontmatter.course.as_deref().unwrap_or(options.course);
    let language = frontmatter.language.as_deref().unwrap_or(options.language);
    let sources = if frontmatter.sources.is_empty() {
        options.sources
    } else {
        &frontmatter.sources
    };

    let mut source_list = sources
        .iter()
        .map(|source| typst_string(source))
        .collect::<Vec<_>>()
        .join(", ");
    if !source_list.is_empty() {
        source_list.push(',');
    }

    let running = frontmatter.running.as_deref().unwrap_or(title);
    let cover_notes = frontmatter
        .cover_notes
        .iter()
        .map(|note| typst_string(note))
        .collect::<Vec<_>>()
        .join(", ");
    let cover_notes = if cover_notes.is_empty() {
        "()".to_string()
    } else {
        format!("({cover_notes},)")
    };
    let source = format!(
        "{template}\n\n#show: paper.with(\n  title: {},\n  subtitle: {},\n  course: {},\n  language: {},\n  sources: ({}),\n  running: {},\n  pilot: {},\n  deck: {},\n  cover_kicker: {},\n  cover_title: {},\n  cover_motif: {},\n  cover_notes: {},\n)\n\n{}",
        typst_string(title),
        typst_string(subtitle),
        typst_string(course),
        typst_string(language),
        source_list,
        typst_string(running),
        optional_string(frontmatter.pilot.as_deref()),
        optional_string(frontmatter.deck.as_deref()),
        optional_string(frontmatter.cover_kicker.as_deref()),
        optional_string(frontmatter.cover_title.as_deref()),
        optional_string(frontmatter.cover_motif.as_deref()),
        cover_notes,
        body,
    );

    let world = TypstWorld::new(options.guide_root, source).map_err(Error::Typst)?;
    let warned = compile::<PagedDocument>(&world);
    let document = warned
        .output
        .map_err(|errors| Error::Typst(format_diagnostics(&world, &errors)))?;
    let pages = document.pages().len();
    let pdf = typst_pdf::pdf(&document, &typst_pdf::PdfOptions::default())
        .map_err(|errors| Error::Typst(format!("{errors:?}")))?;
    let warnings = warned
        .warnings
        .iter()
        .map(|warning| format!("{warning:?}"))
        .collect();

    let rendered = Rendered {
        pdf,
        pages,
        content_hash: content_hash(content),
        cache_hit: false,
        warnings,
    };
    if let Some(cache_dir) = options.cache_dir {
        store_cached(cache_dir, &cache_key, &rendered);
    }
    Ok(rendered)
}

fn build_cache_key(
    content: &str,
    profile: &folisift_core::StyleProfile,
    loaded: &LoadedStyle,
    strings: &StyleStrings,
) -> String {
    let profile = serde_json::to_string(profile).unwrap_or_default();
    let tokens = serde_json::to_string(&loaded.tokens).unwrap_or_default();
    let strings = serde_json::to_string(strings).unwrap_or_default();
    let template_hash = match loaded.pack.layout.as_str() {
        "paper" => blake3::hash(PAPER_TEMPLATE.as_bytes()).to_hex().to_string(),
        "clean" => blake3::hash(CLEAN_TEMPLATE.as_bytes()).to_hex().to_string(),
        _ => String::new(),
    };
    blake3::hash(
        format!(
            "{}\n{content}\n{profile}\n{}\n{tokens}\n{strings}\n{template_hash}\n{}",
            env!("CARGO_PKG_VERSION"),
            loaded.pack.id,
            env!("CARGO_PKG_VERSION"),
        )
        .as_bytes(),
    )
    .to_hex()
    .to_string()
}

fn cache_paths(cache_dir: &Path, key: &str) -> (std::path::PathBuf, std::path::PathBuf) {
    (
        cache_dir.join(format!("{key}.pdf")),
        cache_dir.join(format!("{key}.json")),
    )
}

fn load_cached(cache_dir: &Path, key: &str) -> Option<Rendered> {
    let (pdf_path, meta_path) = cache_paths(cache_dir, key);
    let pdf = std::fs::read(pdf_path).ok()?;
    let meta: serde_json::Value = serde_json::from_slice(&std::fs::read(meta_path).ok()?).ok()?;
    let pages = meta.get("pages")?.as_u64()? as usize;
    let content_hash = meta.get("content_hash")?.as_str()?.to_string();
    let warnings = meta
        .get("warnings")
        .and_then(serde_json::Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(ToString::to_string))
                .collect()
        })
        .unwrap_or_default();
    Some(Rendered {
        pdf,
        pages,
        content_hash,
        cache_hit: true,
        warnings,
    })
}

fn store_cached(cache_dir: &Path, key: &str, rendered: &Rendered) {
    if std::fs::create_dir_all(cache_dir).is_err() {
        return;
    }
    let (pdf_path, meta_path) = cache_paths(cache_dir, key);
    let _ = std::fs::write(pdf_path.with_extension("pdf.tmp"), &rendered.pdf)
        .and_then(|_| std::fs::rename(pdf_path.with_extension("pdf.tmp"), &pdf_path));
    let meta = serde_json::json!({
        "pages": rendered.pages,
        "content_hash": rendered.content_hash,
        "warnings": rendered.warnings,
    });
    let _ = std::fs::write(meta_path.with_extension("json.tmp"), meta.to_string())
        .and_then(|_| std::fs::rename(meta_path.with_extension("json.tmp"), &meta_path));
}

fn format_diagnostics(world: &TypstWorld, errors: &[typst::diag::SourceDiagnostic]) -> String {
    errors
        .iter()
        .map(|error| {
            let location = world.range(error.span).map(|range| {
                let line = world
                    .source(world.main())
                    .map(|source| source.text()[..range.start].lines().count())
                    .unwrap_or(0);
                format!("line {line}: ")
            });
            format!("{}{}", location.unwrap_or_default(), error.message)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn typst_string(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn optional_string(value: Option<&str>) -> String {
    value.map_or_else(|| "none".to_string(), typst_string)
}
