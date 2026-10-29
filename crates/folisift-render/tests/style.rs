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
        cache_dir: None,
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
fn renders_a_captioned_table() {
    let root = tempfile::tempdir().expect("tempdir");
    let content = r#"---
title: "Table test"
language: en
sources: [PD0]
---

::: chapter part="B" title="Tables" toc="Tables"
:::

::: table caption="Compilation vs interpretation" source="PD0, s. 106" widths="35,65"
| Criterion | Meaning |
|---|---|
| Compilation | Code is generated once. |
| Interpretation | Each statement is translated at run time. |
:::
"#;
    let profile = StyleProfile {
        aesthetic: "paper".to_string(),
        language: "en".to_string(),
        source_language: "en".to_string(),
        ..StyleProfile::default()
    };
    let mut options = options(&profile);
    options.guide_root = root.path();
    options.cache_dir = None;
    let rendered = render(content, &options).expect("table render");
    assert!(rendered.pdf.starts_with(b"%PDF-"));
    assert!(rendered.pages >= 1);
}

#[test]
fn compiles_a_diagram_block_to_svg() {
    let root = tempfile::tempdir().expect("tempdir");
    let content = r#"---
title: "Diagram test"
language: en
sources: [PD0]
---

::: chapter part="B" title="Flow" toc="Flow"
:::

```diagram
{
  "kind": "flow",
  "title": "Compiler pipeline",
  "alt": "Source program flows through analysis to generated code.",
  "nodes": [
    {"id": "src", "label": "Source program"},
    {"id": "lex", "label": "Lexical analysis"},
    {"id": "parse", "label": "Syntax analysis"},
    {"id": "code", "label": "Code generation"}
  ],
  "edges": [
    {"from": "src", "to": "lex", "label": "characters"},
    {"from": "lex", "to": "parse", "label": "tokens"},
    {"from": "parse", "to": "code", "label": "AST"}
  ]
}
```
"#;
    let profile = StyleProfile {
        aesthetic: "paper".to_string(),
        language: "en".to_string(),
        source_language: "en".to_string(),
        ..StyleProfile::default()
    };
    let mut options = options(&profile);
    options.guide_root = root.path();
    options.cache_dir = None;
    let rendered = render(content, &options).expect("diagram render");
    assert!(rendered.pdf.starts_with(b"%PDF-"));
    let svg = std::fs::read_dir(root.path().join("assets/generated"))
        .expect("generated dir")
        .filter_map(Result::ok)
        .find(|entry| entry.path().extension().and_then(|value| value.to_str()) == Some("svg"))
        .expect("svg");
    let text = std::fs::read_to_string(svg.path()).expect("svg text");
    assert!(text.contains("role=\"img\""));
    assert!(text.contains("<title"));
    assert!(text.contains("Lexical analysis"));
}

#[test]
fn reuses_the_build_cache() {
    let root = tempfile::tempdir().expect("tempdir");
    let cache = root.path().join("cache");
    let profile = StyleProfile {
        aesthetic: "paper".to_string(),
        language: "en".to_string(),
        source_language: "en".to_string(),
        ..StyleProfile::default()
    };
    let mut render_options = options(&profile);
    render_options.guide_root = Path::new(".");
    render_options.cache_dir = Some(&cache);
    let first = render(SAMPLE, &render_options).expect("first render");
    let second = render(SAMPLE, &render_options).expect("second render");
    assert!(!first.cache_hit);
    assert!(second.cache_hit);
    assert_eq!(first.pdf, second.pdf);
    assert_eq!(first.content_hash, second.content_hash);
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
