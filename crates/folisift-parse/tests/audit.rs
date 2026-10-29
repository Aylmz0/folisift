use std::path::Path;

use folisift_parse::audit_pdf;

#[test]
fn committed_paper_baseline_passes_layout_and_bookmark_checks() {
    let pdf = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/baselines/pd0-pd1.pdf");
    let audit = audit_pdf(&pdf, Some(22)).expect("audit");
    assert!(audit.passed, "{audit:#?}");
    assert_eq!(audit.pages, 22);
    assert!(audit.tagged);
    assert!(audit.bookmarks > 60);
    assert!(audit.orphan_heading_pages.is_empty());
    assert!(audit.bookmark_mismatches.is_empty());
    assert!(audit.content_box_pages.is_empty());
    assert!(audit.min_font_size >= 6.0);
}
