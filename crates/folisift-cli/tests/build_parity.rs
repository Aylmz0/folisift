use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

fn workspace() -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(root.path().join("sources")).expect("sources");
    std::fs::create_dir_all(root.path().join("guides/demo")).expect("guide");
    std::fs::copy(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/pdf/orphan-heading.pdf"
        ),
        root.path().join("sources/PD0.pdf"),
    )
    .expect("source");
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
          "theme": "reader",
          "style": {
            "aesthetic": "reader",
            "structure": "a-g",
            "language": "en",
            "source_language": "en",
            "voice": "academic",
            "study_mode": "exam",
            "domain": "cs",
            "citation": "page-ref"
          },
          "style_rationale": "CLI/MCP build parity fixture."
        }"#,
    )
    .expect("config");
    std::fs::write(
        root.path().join("guides/demo/guide.md"),
        r#"---
title: "Demo"
language: en
sources: [PD0]
---

::: front kicker="TEST" title="Demo" lead="A parity fixture."
:::

::: toc title="Contents" note="Page numbers refer to this PDF."
:::

## A · Source

A short paragraph with a source reference. [PD0, s. 1]
"#,
    )
    .expect("guide");
    root
}

#[test]
fn cli_and_mcp_builds_agree_on_pages_and_content_hash() {
    let root = workspace();
    let mut child = Command::new(env!("CARGO_BIN_EXE_folisift"))
        .args(["mcp", "--root", root.path().to_str().unwrap()])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn mcp");
    let mut stdin = child.stdin.take().expect("stdin");
    let stdout = child.stdout.take().expect("stdout");
    let mut reader = BufReader::new(stdout);

    let send = |stdin: &mut std::process::ChildStdin, value: serde_json::Value| {
        writeln!(stdin, "{value}").expect("write");
        stdin.flush().expect("flush");
    };
    send(
        &mut stdin,
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-11-25",
                "capabilities": {},
                "clientInfo": {"name": "build-parity", "version": "0"}
            }
        }),
    );
    let mut line = String::new();
    reader.read_line(&mut line).expect("initialize");
    send(
        &mut stdin,
        serde_json::json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
    );
    send(
        &mut stdin,
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "build_guide",
                "arguments": {"slug": "demo", "check": false}
            }
        }),
    );
    line.clear();
    reader.read_line(&mut line).expect("build");
    let mcp: serde_json::Value = serde_json::from_str(&line).expect("mcp JSON");
    let mcp = &mcp["result"]["structuredContent"];
    assert_eq!(mcp["checked"], false, "{mcp}");
    assert_eq!(mcp["passed"], false, "{mcp}");

    drop(stdin);
    let status = child.wait().expect("wait");
    assert!(status.success(), "{status}");

    let cli = Command::new(env!("CARGO_BIN_EXE_folisift"))
        .args([
            "build",
            "--root",
            root.path().to_str().unwrap(),
            "--guide",
            "demo",
        ])
        .output()
        .expect("cli build");
    assert!(
        cli.status.success(),
        "{}",
        String::from_utf8_lossy(&cli.stderr)
    );
    let cli: serde_json::Value = serde_json::from_slice(&cli.stdout).expect("cli JSON");
    assert_eq!(cli["pages"], mcp["pages"], "{cli} {mcp}");
    assert_eq!(cli["content_hash"], mcp["content_hash"], "{cli} {mcp}");
}
