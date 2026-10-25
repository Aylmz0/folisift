use std::fs;
use std::path::Path;

use folisift_core::{
    Error, Guide, ScaffoldOptions, Workspace, list_guides, scaffold, validate_slug,
};

fn fixture() -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(root.path().join("sources")).expect("sources");
    fs::create_dir_all(root.path().join("output")).expect("output");
    fs::create_dir_all(root.path().join("guides/demo")).expect("guide");
    fs::write(root.path().join("sources/PD0.pdf"), b"%PDF-1.7\n").expect("source");
    fs::write(root.path().join("guides/demo/guide.html"), "<html></html>").expect("content");
    fs::write(
        root.path().join("guides/demo/config.json"),
        r#"{
          "slug": "demo",
          "title": "Demo Guide",
          "language": "tr",
          "content": "guide.html",
          "output": "../../output/demo.pdf",
          "sources": ["../../sources/PD0.pdf"],
          "target_pages": "18-22"
        }"#,
    )
    .expect("config");
    root
}

fn workspace(root: &Path) -> Workspace {
    Workspace::open(root).expect("workspace")
}

#[test]
fn loads_a_valid_guide() {
    let root = fixture();
    let guide = Guide::load(&workspace(root.path()), "demo").expect("guide");

    assert_eq!(guide.slug, "demo");
    assert_eq!(guide.config.title, "Demo Guide");
    assert_eq!(guide.config.schema_version, 1);
    assert_eq!(guide.sources.len(), 1);
    assert!(guide.content.ends_with("guide.html"));
    assert!(guide.output.ends_with("output/demo.pdf"));
}

#[test]
fn lists_guides_with_status() {
    let root = fixture();
    let guides = list_guides(&workspace(root.path())).expect("list");
    assert_eq!(guides.len(), 1);
    assert!(guides[0].valid);
    assert_eq!(guides[0].slug, "demo");
}

#[test]
fn rejects_paths_above_the_workspace() {
    let root = fixture();
    let workspace = workspace(root.path());

    let error = workspace
        .resolve_write("../outside.pdf")
        .expect_err("must reject");
    assert!(matches!(error, Error::PathEscape { .. }));
}

#[test]
fn rejects_absolute_paths_outside_the_workspace() {
    let root = fixture();
    let workspace = workspace(root.path());
    let outside = tempfile::tempdir().expect("outside");
    let path = outside.path().join("outside.pdf");

    let error = workspace.resolve_write(path).expect_err("must reject");
    assert!(matches!(error, Error::PathEscape { .. }));
}

#[test]
fn sources_are_read_only() {
    let root = fixture();
    let workspace = workspace(root.path());

    let error = workspace
        .resolve_write("sources/PD0.pdf")
        .expect_err("sources are read-only");
    assert!(matches!(error, Error::ReadOnly { .. }));
}

#[test]
fn guides_and_output_are_writable() {
    let root = fixture();
    let workspace = workspace(root.path());

    assert!(workspace.resolve_write("guides/demo/data/new.json").is_ok());
    assert!(workspace.resolve_write("output/new.pdf").is_ok());
}

#[test]
fn rejects_traversal_inside_a_config_path() {
    let root = fixture();
    fs::write(
        root.path().join("guides/demo/config.json"),
        r#"{
          "slug": "demo",
          "title": "Demo",
          "content": "../../sources/PD0.pdf",
          "output": "../../output/demo.pdf",
          "sources": ["../../sources/PD0.pdf"]
        }"#,
    )
    .expect("config");

    let error = Guide::load(&workspace(root.path()), "demo").expect_err("must reject");
    assert!(matches!(error, Error::Config { .. }));
}

#[test]
fn loads_a_schema_two_markdown_guide() {
    let root = fixture();
    fs::create_dir_all(root.path().join("guides/markdown")).expect("guide");
    fs::write(root.path().join("guides/markdown/guide.md"), "# Demo").expect("content");
    fs::write(
        root.path().join("guides/markdown/config.json"),
        r#"{
          "schema_version": 2,
          "slug": "markdown",
          "title": "Markdown Guide",
          "content": "guide.md",
          "output": "../../output/markdown.pdf",
          "sources": ["../../sources/PD0.pdf"]
        }"#,
    )
    .expect("config");

    let guide = Guide::load(&workspace(root.path()), "markdown").expect("guide");
    assert_eq!(guide.config.schema_version, 2);
    assert!(guide.content.ends_with("guide.md"));
}

#[test]
fn rejects_an_unsupported_schema_version() {
    let root = fixture();
    fs::write(
        root.path().join("guides/demo/config.json"),
        r#"{
          "schema_version": 3,
          "slug": "demo",
          "title": "Demo",
          "content": "guide.html",
          "output": "../../output/demo.pdf",
          "sources": ["../../sources/PD0.pdf"]
        }"#,
    )
    .expect("config");

    let error = Guide::load(&workspace(root.path()), "demo").expect_err("must reject");
    assert!(matches!(error, Error::UnsupportedSchema { version: 3 }));
}

#[cfg(unix)]
#[test]
fn rejects_a_symlink_that_escapes_the_workspace() {
    let root = fixture();
    let outside = tempfile::tempdir().expect("outside");
    fs::write(outside.path().join("secret.pdf"), b"secret").expect("outside file");
    std::os::unix::fs::symlink(outside.path(), root.path().join("linked")).expect("symlink");

    let workspace = workspace(root.path());
    let error = workspace
        .resolve_read("linked/secret.pdf")
        .expect_err("must reject");
    assert!(matches!(error, Error::PathEscape { .. }));
}

#[test]
fn scaffolds_a_schema_two_guide() {
    let root = fixture();
    let guide = scaffold(
        &workspace(root.path()),
        &ScaffoldOptions {
            slug: "pd2".to_string(),
            title: "Bölüm 2".to_string(),
            course: "BMÜ325".to_string(),
            sources: vec![root.path().join("sources/PD0.pdf")],
        },
    )
    .expect("scaffold");

    assert_eq!(guide.config.schema_version, 2);
    assert!(guide.content.ends_with("guide.md"));
    assert!(root.path().join("guides/pd2/guide.md").is_file());
    assert!(root.path().join("guides/pd2/data").is_dir());
}

#[test]
fn rejects_invalid_slugs() {
    for slug in ["", "Demo", "demo_guide", "-demo", "demo-", "demo--guide"] {
        assert!(validate_slug(slug).is_err(), "{slug:?} should be invalid");
    }
    assert!(validate_slug("pd0-pd1").is_ok());
}

#[test]
fn rejects_invalid_target_pages() {
    let root = fixture();
    fs::write(
        root.path().join("guides/demo/config.json"),
        r#"{
          "slug": "demo",
          "title": "Demo",
          "content": "guide.html",
          "output": "../../output/demo.pdf",
          "sources": ["../../sources/PD0.pdf"],
          "target_pages": "22-18"
        }"#,
    )
    .expect("config");

    let error = Guide::load(&workspace(root.path()), "demo").expect_err("must reject");
    assert!(matches!(error, Error::InvalidTargetPages { .. }));
}
