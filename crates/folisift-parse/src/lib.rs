//! liteparse adapter, source inventory and page rendering.
//!
//! The production parser is liteparse only. OCR is disabled; low-text pages
//! are routed to rendered screenshots for the host agent to inspect.

pub mod coverage;
pub mod error;
pub mod inventory;
pub mod parser;
pub mod pdfium;
pub mod render;
pub mod source_cache;
pub mod source_read;
pub mod text;

pub use coverage::{
    CoverageReport, DEFAULT_THRESHOLD, MIN_SECTION_CHARS, check_coverage, guide_body_chars,
};
pub use error::{Error, Result};
pub use inventory::{Heading, Inventory, PageRow, SourceInventory, Totals};
pub use parser::{LiteparseParser, SourceParser, analyze_source, analyze_sources, parse_page_list};
pub use pdfium::{PDFIUM_DIR_ENV, init as init_pdfium};
pub use render::{PageImage, render_pages};
pub use source_cache::{PageText, SourceCache, SourceText};
pub use source_read::{
    ReadSourceResult, SearchHit, SearchSourceResult, read_source, search_source,
};
pub use text::LOW_TEXT_THRESHOLD;
