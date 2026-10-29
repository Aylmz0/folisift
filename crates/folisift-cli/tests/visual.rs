use std::path::Path;

use folisift_core::StyleProfile;
use folisift_render::{RenderOptions, render};

fn options<'a>(style: &'a StyleProfile, guide_root: &'a Path) -> RenderOptions<'a> {
    RenderOptions {
        guide_root,
        title: "Visual baseline",
        subtitle: "",
        course: "",
        language: "en",
        sources: &[],
        style: Some(style),
        styles_root: None,
        cache_dir: None,
    }
}

fn profile() -> StyleProfile {
    StyleProfile {
        aesthetic: "paper".to_string(),
        language: "en".to_string(),
        source_language: "en".to_string(),
        ..StyleProfile::default()
    }
}

fn content(body: &str) -> String {
    format!(
        "---\ntitle: \"Visual baseline\"\nlanguage: en\nsources: [PD0]\n---\n\n::: chapter part=\"B\" title=\"Layout\" toc=\"Layout\"\n:::\n\n## 1.1 Layout\n\n{body}\n"
    )
}

#[test]
fn identical_pdfs_have_no_visual_difference() {
    let root = tempfile::tempdir().expect("tempdir");
    let profile = profile();
    let markdown = content("The same paragraph should render to the same pixels.");
    let rendered = render(&markdown, &options(&profile, root.path())).expect("render");
    let pdf = root.path().join("same.pdf");
    std::fs::write(&pdf, &rendered.pdf).expect("pdf");

    let report =
        folisift_parse::compare_pdfs(&pdf, &pdf, &[1], 72.0, &root.path().join("diff"), 0.5, 0.01)
            .expect("compare");
    assert!(report.passed, "{report:?}");
    assert_eq!(report.pages[0].mean_difference, 0.0);
}

#[test]
fn changed_content_exceeds_the_visual_threshold() {
    let root = tempfile::tempdir().expect("tempdir");
    let profile = profile();
    let first = render(
        &content("Alpha text fills the first page with a stable sentence."),
        &options(&profile, root.path()),
    )
    .expect("first render");
    let second = render(
        &content(
            "Beta text replaces the page with a completely different paragraph and a longer line that moves the layout.",
        ),
        &options(&profile, root.path()),
    )
    .expect("second render");
    let first_pdf = root.path().join("first.pdf");
    let second_pdf = root.path().join("second.pdf");
    std::fs::write(&first_pdf, &first.pdf).expect("first pdf");
    std::fs::write(&second_pdf, &second.pdf).expect("second pdf");

    let pages = (1..=first.pages.max(second.pages)).collect::<Vec<_>>();
    let report = folisift_parse::compare_pdfs(
        &first_pdf,
        &second_pdf,
        &pages,
        72.0,
        &root.path().join("diff"),
        0.1,
        0.001,
    )
    .expect("compare");
    assert!(!report.passed, "{report:?}");
    assert!(
        report
            .pages
            .iter()
            .any(|page| page.mean_difference > 0.1 || page.changed_ratio > 0.001)
    );
}
