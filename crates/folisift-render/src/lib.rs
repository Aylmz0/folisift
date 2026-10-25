//! Markdown + directives to Typst PDF.
//!
//! The content format is deliberately small: CommonMark, YAML frontmatter and
//! four block directives. Guides never contain raw Typst.

pub mod error;
pub mod markdown;
pub mod world;

use std::path::Path;

use typst::compile;
use typst_layout::PagedDocument;

pub use error::{Error, Result};
pub use markdown::{Frontmatter, markdown_to_typst, parse_document};
use world::TypstWorld;

const TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../templates/paper.typ"
));

pub struct RenderOptions<'a> {
    /// Directory containing `assets/` and the generated Typst source.
    pub guide_root: &'a Path,
    pub title: &'a str,
    pub subtitle: &'a str,
    pub course: &'a str,
    pub language: &'a str,
    pub sources: &'a [String],
}

#[derive(Debug, Clone)]
pub struct Rendered {
    pub pdf: Vec<u8>,
    pub pages: usize,
    pub warnings: Vec<String>,
}

/// Render one guide Markdown document to PDF bytes.
pub fn render(content: &str, options: &RenderOptions<'_>) -> Result<Rendered> {
    let (frontmatter, body) = parse_document(content)?;
    let body = markdown_to_typst(&body)?;

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

    let source = format!(
        "{TEMPLATE}\n\n#show: paper.with(\n  title: {},\n  subtitle: {},\n  course: {},\n  language: {},\n  sources: ({}),\n)\n\n{}",
        typst_string(title),
        typst_string(subtitle),
        typst_string(course),
        typst_string(language),
        source_list,
        body,
    );

    let world = TypstWorld::new(options.guide_root, source).map_err(Error::Typst)?;
    let warned = compile::<PagedDocument>(&world);
    let document = warned
        .output
        .map_err(|errors| Error::Typst(format!("{errors:?}")))?;
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
        warnings,
    })
}

fn typst_string(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}
