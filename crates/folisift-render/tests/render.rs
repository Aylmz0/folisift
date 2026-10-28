use std::path::Path;

use folisift_render::{RenderOptions, render};

#[test]
fn renders_a_minimal_paper_document() {
    let root = tempfile::tempdir().expect("tempdir");
    let content = r#"---
title: "Test Kılavuzu"
subtitle: "Faz 3"
course: "BMÜ325"
sources: [PD0]
language: tr
---

# Bölüm 0

## 0.1 Giriş

Türkçe karakter: ğüşıöç.

::: definition
**Tanım:** deneme.
:::

| A | B |
|---|---|
| 1 | 2 |
"#;
    let rendered = render(
        content,
        &RenderOptions {
            guide_root: Path::new(root.path()),
            title: "Fallback",
            subtitle: "",
            course: "",
            language: "tr",
            sources: &["PD0".to_string()],
            style: None,
            styles_root: None,
        },
    )
    .expect("render");
    assert!(rendered.pages >= 2);
    assert!(rendered.pdf.starts_with(b"%PDF-"));
    assert!(rendered.warnings.is_empty());
}
