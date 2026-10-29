use std::path::Path;
use std::process::Command;

fn setup(root: &Path) {
    std::fs::create_dir_all(root.join("sources")).expect("sources");
    std::fs::write(root.join("sources/PD0.pdf"), b"%PDF-1.4\n").expect("source");
    std::fs::create_dir_all(root.join("guides/demo/diagrams")).expect("guide");
    std::fs::write(
        root.join("guides/demo/config.json"),
        r#"{
          "schema_version": 3,
          "slug": "demo",
          "title": "D2 demo",
          "language": "en",
          "content": "guide.md",
          "output": "../../output/demo.pdf",
          "sources": ["../../sources/PD0.pdf"],
          "style": {
            "aesthetic": "paper",
            "structure": "a-g",
            "language": "en",
            "source_language": "en",
            "voice": "academic",
            "study_mode": "exam",
            "domain": "cs",
            "citation": "page-ref"
          }
        }"#,
    )
    .expect("config");
    std::fs::write(
        root.join("guides/demo/guide.md"),
        "---\ntitle: \"D2 demo\"\nlanguage: en\nsources: [PD0]\n---\n\n\
         ::: chapter part=\"B\" title=\"Diagram\" toc=\"Diagram\"\n:::\n\n\
         ## 1.1 Pipeline\n\n::: diagram id=\"pipeline\"\n:::\n",
    )
    .expect("guide");
    std::fs::write(
        root.join("guides/demo/diagrams/pipeline.json"),
        r#"{
          "schema_version": 2,
          "id": "pipeline",
          "kind": "flow",
          "title": "Pipeline",
          "caption": "A source program is analysed and converted into target code.",
          "alt": "A source program is analysed and converted into target code.",
          "source": "PD0, s. 4",
          "direction": "right",
          "layout": "tala",
          "nodes": [
            {"id": "src", "label": "Source program"},
            {"id": "lex", "label": "Lexical analysis"},
            {"id": "code", "label": "Code generation"}
          ],
          "edges": [
            {"from": "src", "to": "lex", "label": "characters"},
            {"from": "lex", "to": "code", "label": "tokens"}
          ]
        }"#,
    )
    .expect("spec");
}

#[test]
fn builds_and_embeds_a_d2_diagram_without_d2_at_build_time() {
    let Ok(d2) = std::env::var("FOLISIFT_D2") else {
        eprintln!("skipping: set FOLISIFT_D2 to a d2 v0.9.0 binary");
        return;
    };
    let root = tempfile::tempdir().expect("tempdir");
    setup(root.path());
    let first = Command::new(env!("CARGO_BIN_EXE_folisift"))
        .args(["diagram", "build", "--guide", "demo", "--id", "pipeline"])
        .env("FOLISIFT_D2", &d2)
        .current_dir(root.path())
        .output()
        .expect("diagram build");
    assert!(
        first.status.success(),
        "diagram build failed: {}",
        String::from_utf8_lossy(&first.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&first.stdout).expect("build JSON");
    assert_eq!(value["items"][0]["cache_hit"], false);
    let manifest = value["items"][0]["manifest"]
        .as_str()
        .expect("manifest path");
    assert!(Path::new(manifest).is_file());

    let second = Command::new(env!("CARGO_BIN_EXE_folisift"))
        .args(["diagram", "build", "--guide", "demo", "--id", "pipeline"])
        .env("FOLISIFT_D2", &d2)
        .current_dir(root.path())
        .output()
        .expect("second build");
    let value: serde_json::Value = serde_json::from_slice(&second.stdout).expect("cache JSON");
    assert_eq!(value["items"][0]["cache_hit"], true);

    let build = Command::new(env!("CARGO_BIN_EXE_folisift"))
        .args(["build", "--guide", "demo"])
        .env_remove("FOLISIFT_D2")
        .current_dir(root.path())
        .output()
        .expect("guide build");
    assert!(
        build.status.success(),
        "embed-only build failed: {}",
        String::from_utf8_lossy(&build.stderr)
    );
    assert!(root.path().join("output/demo.pdf").is_file());

    let manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(manifest).expect("manifest"))
            .expect("manifest JSON");
    let svg = root
        .path()
        .join("guides/demo")
        .join(manifest["svg"].as_str().expect("svg"));
    std::fs::remove_file(svg).expect("remove svg");
    let failed = Command::new(env!("CARGO_BIN_EXE_folisift"))
        .args(["build", "--guide", "demo"])
        .env_remove("FOLISIFT_D2")
        .current_dir(root.path())
        .output()
        .expect("missing asset build");
    assert!(!failed.status.success());
    assert!(
        String::from_utf8_lossy(&failed.stderr).contains("diagram"),
        "{}",
        String::from_utf8_lossy(&failed.stderr)
    );
}
