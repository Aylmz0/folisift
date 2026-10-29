use std::path::Path;

use folisift_core::StyleProfile;
use folisift_render::{RenderOptions, render};

fn options<'a>(style: &'a StyleProfile, root: &'a Path) -> RenderOptions<'a> {
    RenderOptions {
        guide_root: root,
        title: "Table parity",
        subtitle: "",
        course: "",
        language: "en",
        sources: &[],
        style: Some(style),
        styles_root: None,
        cache_dir: None,
    }
}

fn table_markdown() -> String {
    let mut body = String::from(
        "---\ntitle: \"Table parity\"\nlanguage: en\nsources: [PD0]\n---\n\n\
         ::: chapter part=\"B\" title=\"Tables\" toc=\"Tables\"\n:::\n\n",
    );
    for index in 1..=24 {
        body.push_str(&format!(
            "::: table caption=\"Comparison {index}\" source=\"PD0, s. {index}\" widths=\"25,35,40\" id=\"tab-{index}\"\n\
             | Criterion | Compilation | Interpretation |\n\
             |---|---|---|\n\
             | When | {index} time | every run |\n\
             | Output | target code | result |\n\
             :::\n\n"
        ));
    }
    body
}

#[test]
fn twenty_four_tables_render_and_audit_in_both_styles() {
    let root = tempfile::tempdir().expect("tempdir");
    let content = table_markdown();
    for aesthetic in ["paper", "clean"] {
        let style = StyleProfile {
            aesthetic: aesthetic.to_string(),
            language: "en".to_string(),
            source_language: "en".to_string(),
            ..StyleProfile::default()
        };
        let rendered = render(&content, &options(&style, root.path())).expect("render");
        let pdf = root.path().join(format!("{aesthetic}.pdf"));
        std::fs::write(&pdf, &rendered.pdf).expect("pdf");
        let audit = folisift_parse::audit_pdf(&pdf, None).expect("audit");
        assert!(audit.pages > 1, "{aesthetic}");
        assert!(audit.tagged, "{aesthetic}");
        assert!(audit.bookmarks > 0, "{aesthetic}: {audit:#?}");
        assert!(
            audit.content_box_pages.is_empty(),
            "{aesthetic}: {audit:#?}"
        );
        assert!(
            audit.orphan_heading_pages.is_empty(),
            "{aesthetic}: {audit:#?}"
        );
        assert!(
            audit.image_overflow_pages.is_empty(),
            "{aesthetic}: {audit:#?}"
        );
        assert!(audit.min_font_size >= 6.0, "{aesthetic}: {audit:#?}");
    }
}
