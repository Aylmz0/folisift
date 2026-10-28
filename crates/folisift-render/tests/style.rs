use std::path::Path;

use folisift_core::StyleProfile;
use folisift_render::{RenderOptions, content_hash, render, styles};

const SAMPLE: &str = include_str!("../../../tests/fixtures/style-sample.md");

fn options(style: &StyleProfile) -> RenderOptions<'_> {
    RenderOptions {
        guide_root: Path::new("."),
        title: "Fallback",
        subtitle: "",
        course: "",
        language: "en",
        sources: &[],
        style: Some(style),
        styles_root: None,
    }
}

#[test]
fn paper_and_clean_share_the_content_hash() {
    let paper = StyleProfile {
        aesthetic: "paper".to_string(),
        language: "en".to_string(),
        source_language: "en".to_string(),
        ..StyleProfile::default()
    };
    let clean = StyleProfile {
        aesthetic: "clean".to_string(),
        structure: "imrad".to_string(),
        language: "en".to_string(),
        source_language: "en".to_string(),
        domain: "cs".to_string(),
        ..StyleProfile::default()
    };
    let paper_pdf = render(SAMPLE, &options(&paper)).expect("paper render");
    let clean_pdf = render(SAMPLE, &options(&clean)).expect("clean render");
    assert!(paper_pdf.pdf.starts_with(b"%PDF-"));
    assert!(clean_pdf.pdf.starts_with(b"%PDF-"));
    assert_eq!(paper_pdf.content_hash, clean_pdf.content_hash);
    assert_eq!(paper_pdf.content_hash, content_hash(SAMPLE));
    assert_ne!(paper_pdf.pdf, clean_pdf.pdf);
    assert!(paper_pdf.pages >= 4);
    assert!(clean_pdf.pages >= 4);
}

#[test]
fn english_strings_have_no_turkish_ui_labels() {
    let strings = styles::strings(Path::new(""), "paper", "en").expect("en strings");
    assert_eq!(strings.toc, "CONTENTS");
    assert_eq!(strings.figure, "Figure");
    let clean = styles::load(Path::new(""), "clean").expect("clean pack");
    assert_eq!(clean.pack.id, "clean");
}

#[test]
fn rejects_incompatible_structure_and_domain() {
    let profile = StyleProfile {
        structure: "soap".to_string(),
        domain: "humanities".to_string(),
        ..StyleProfile::default()
    };
    let error = render(SAMPLE, &options(&profile)).expect_err("must reject");
    assert!(error.to_string().contains("soap"));
}
