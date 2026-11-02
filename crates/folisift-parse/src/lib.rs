//! liteparse adapter, source inventory and page rendering.
//!
//! The production parser is liteparse only. OCR is disabled; low-text pages
//! are routed to rendered screenshots for the host agent to inspect.

pub mod audit;
pub mod coverage;
pub mod error;
pub mod inventory;
pub mod layout;
pub mod parser;
pub mod pdfium;
pub mod render;
pub mod revision_coverage;
pub mod source_cache;
pub mod source_read;
pub mod table;
pub mod text;
pub mod visual;
pub mod visual_audit;

pub use audit::{AuditOptions, LayoutAudit, PageAudit, audit_pdf, audit_pdf_with};
pub use coverage::{
    CoverageReport, DEFAULT_THRESHOLD, MIN_SECTION_CHARS, check_coverage, guide_body_chars,
};
pub use error::{Error, Result};
pub use inventory::{CanonicalSection, Heading, Inventory, PageRow, SourceInventory, Totals};
pub use layout::{
    BlockView, LAYOUT_SCHEMA_VERSION, LayoutBlock, LayoutBlockKind, LayoutCell, LayoutSummary,
    PageLayout, PageLayoutSummary, Rect as LayoutRect, SourceLayout, read_block_view,
    write_layouts,
};
pub use parser::{
    Analysis, AnalyzedSource, LiteparseParser, SourceParser, analyze_source, analyze_sources,
    analyze_sources_with_layout, parse_page_list,
};
pub use pdfium::{PDFIUM_DIR_ENV, init as init_pdfium};
pub use render::CropBox;
pub use render::{PageImage, render_pages};
pub use revision_coverage::{
    RevisionCoverageReport, revision_coverage_issues, revision_coverage_report,
};
pub use source_cache::{PageText, SourceCache, SourceText};
pub use source_read::{
    ReadSourceResult, SearchHit, SearchSourceResult, read_source, read_source_at, search_source,
};
pub use table::{BBox, ExtractedTable, extract_tables};
pub use text::LOW_TEXT_THRESHOLD;
pub use visual::{VisualPage, VisualReport, compare_pdfs};
pub use visual_audit::{
    DECISIONS as VISUAL_DECISIONS, VISUAL_AUDIT_SCHEMA_VERSION, VisualAudit, VisualEntry,
    VisualReport as GuideVisualReport,
};
