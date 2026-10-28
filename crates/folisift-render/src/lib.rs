//! Markdown + directives to Typst PDF.
//!
//! The content format is deliberately small: CommonMark, YAML frontmatter and
//! four block directives. Guides never contain raw Typst.

pub mod error;
pub mod markdown;
pub mod styles;
pub mod world;

use std::path::Path;

use typst::{World, WorldExt, compile};
use typst_layout::PagedDocument;

pub use error::{Error, Result};
pub use markdown::{Frontmatter, markdown_to_typst, markdown_to_typst_styled, parse_document};
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
}

#[derive(Debug, Clone)]
pub struct Rendered {
    pub pdf: Vec<u8>,
    pub pages: usize,
    pub content_hash: String,
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
    let body = markdown_to_typst_styled(&body, &strings.figure, &strings.toc)?;
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

    Ok(Rendered {
        pdf,
        pages,
        content_hash: content_hash(content),
        warnings,
    })
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
