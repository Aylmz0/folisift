use std::process::Command;

fn workspace() -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(root.path().join("sources")).expect("sources");
    std::fs::create_dir_all(root.path().join("guides/demo")).expect("guide");
    std::fs::write(root.path().join("sources/PD0.pdf"), b"%PDF-1.4\n").expect("source");
    std::fs::write(
        root.path().join("guides/demo/config.json"),
        r#"{
          "schema_version": 3,
          "slug": "demo",
          "title": "Demo",
          "language": "en",
          "content": "guide.md",
          "output": "../../output/demo.pdf",
          "sources": ["../../sources/PD0.pdf"],
          "target_pages": "2-4",
          "review_intervals": [1, 3, 7],
          "theme": "reader",
          "style": {
            "aesthetic": "reader",
            "structure": "concept-summary",
            "language": "en",
            "source_language": "en",
            "voice": "academic",
            "study_mode": "deep",
            "domain": "cs",
            "citation": "page-ref"
          },
          "style_source": "explicit",
          "style_rationale": "Test fixture for claims and cards."
        }"#,
    )
    .expect("config");
    std::fs::write(
        root.path().join("guides/demo/guide.md"),
        r#"---
title: Demo
source_scope: "PD0:1-1"
language: en
sources: [PD0]
---

## Big picture

Retrieval improves delayed recall by 20% after one week [PD0, p. 1].

::: example
A learner closes the page and writes the definition from memory. [PD0, p. 1]
:::

::: table caption="Tasks" source="PD0, p. 1"
| Task | Action |
|---|---|
| Retrieval | Produce from memory |
:::

## Self-check

::: recall
What improves delayed recall? [PD0, p. 1]
:::

::: answer
Retrieval practice improves delayed recall. [PD0, p. 1]
:::
"#,
    )
    .expect("guide");
    root
}

#[test]
fn claims_and_cards_write_review_artifacts() {
    let root = workspace();
    let claims = Command::new(env!("CARGO_BIN_EXE_folisift"))
        .args([
            "claims",
            "--root",
            root.path().to_str().unwrap(),
            "--guide",
            "demo",
            "--out",
            "data/claims.md",
        ])
        .output()
        .expect("claims");
    assert!(
        claims.status.success(),
        "{}",
        String::from_utf8_lossy(&claims.stderr)
    );
    let report =
        std::fs::read_to_string(root.path().join("data/claims.md")).expect("claims report");
    assert!(report.contains("20%"), "{report}");
    assert!(report.contains("PD0 s. 1"), "{report}");

    let cards = Command::new(env!("CARGO_BIN_EXE_folisift"))
        .args([
            "cards",
            "--root",
            root.path().to_str().unwrap(),
            "--guide",
            "demo",
            "--out",
            "data/cards.csv",
            "--review-intervals",
            "7,1,3,3",
        ])
        .output()
        .expect("cards");
    assert!(
        cards.status.success(),
        "{}",
        String::from_utf8_lossy(&cards.stderr)
    );
    let csv = std::fs::read_to_string(root.path().join("data/cards.csv")).expect("cards csv");
    assert!(csv.contains("review-1d review-3d review-7d"), "{csv}");
    assert!(
        csv.contains("Retrieval practice improves delayed recall."),
        "{csv}"
    );
}

#[test]
fn cards_rejects_an_invalid_interval() {
    let root = workspace();
    let cards = Command::new(env!("CARGO_BIN_EXE_folisift"))
        .args([
            "cards",
            "--root",
            root.path().to_str().unwrap(),
            "--guide",
            "demo",
            "--review-intervals",
            "0",
        ])
        .output()
        .expect("cards");
    assert!(!cards.status.success());
    assert!(
        String::from_utf8_lossy(&cards.stderr).contains("review intervals"),
        "{}",
        String::from_utf8_lossy(&cards.stderr)
    );
}
