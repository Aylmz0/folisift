use std::path::{Path, PathBuf};

use folisift_parse::audit_pdf;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/pdf")
        .join(name)
}

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
    assert!(audit.image_overflow_pages.is_empty());
    assert!(audit.min_font_size >= 6.0);
}

#[test]
fn seeded_orphan_heading_fails() {
    let audit = audit_pdf(&fixture("orphan-heading.pdf"), None).expect("audit");
    assert!(!audit.passed);
    assert_eq!(audit.orphan_heading_pages, vec![1], "{audit:#?}");
}

#[test]
fn seeded_content_box_overflow_fails() {
    let audit = audit_pdf(&fixture("box-overflow.pdf"), None).expect("audit");
    assert!(!audit.passed);
    assert_eq!(audit.content_box_pages, vec![1], "{audit:#?}");
}

#[test]
fn seeded_image_overflow_fails() {
    let audit = audit_pdf(&fixture("image-overflow.pdf"), None).expect("audit");
    assert!(!audit.passed);
    assert_eq!(audit.image_pages, 1, "{audit:#?}");
    assert_eq!(audit.image_overflow_pages, vec![1], "{audit:#?}");
}

#[test]
fn seeded_bookmark_mismatch_fails() {
    let audit = audit_pdf(&fixture("bookmark-mismatch.pdf"), None).expect("audit");
    assert!(!audit.passed);
    assert_eq!(audit.bookmarks, 1, "{audit:#?}");
    assert_eq!(audit.bookmark_mismatches.len(), 1, "{audit:#?}");
    assert!(audit.bookmark_mismatches[0].contains("Wrong destination"));
}
