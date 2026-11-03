use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

#[test]
fn mcp_stdio_lists_tools() {
    let root = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(root.path().join("guides/demo")).expect("guide dir");
    std::fs::create_dir_all(root.path().join("sources")).expect("sources dir");
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
          "style": {
            "aesthetic": "paper",
            "structure": "a-g",
            "language": "en",
            "source_language": "en",
            "voice": "academic",
            "study_mode": "exam",
            "domain": "cs",
            "citation": "page-ref"
          },
          "style_rationale": "Test fixture for the paper layout."
        }"#,
    )
    .expect("config");
    std::fs::write(
        root.path().join("guides/demo/guide.md"),
        r#"---
title: Demo
language: en
sources: [PD0]
---

## 1.1 Test

Recall improves by 20% after one week [PD0, p. 1].

::: recall
What is X? [PD0, p. 1]
:::

::: answer
X is Y. [PD0, p. 1]
:::
"#,
    )
    .expect("guide");
    let mut child = Command::new(env!("CARGO_BIN_EXE_folisift"))
        .arg("mcp")
        .arg("--root")
        .arg(root.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn folisift mcp");

    let mut stdin = child.stdin.take().expect("stdin");
    let stdout = child.stdout.take().expect("stdout");
    let mut reader = BufReader::new(stdout);

    let initialize = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-11-25",
            "capabilities": {},
            "clientInfo": {"name": "stdio-test", "version": "0"}
        }
    });
    writeln!(stdin, "{initialize}").expect("write initialize");
    stdin.flush().expect("flush initialize");
    let mut line = String::new();
    reader.read_line(&mut line).expect("read initialize");
    let response: serde_json::Value = serde_json::from_str(&line).expect("initialize JSON");
    assert_eq!(response["result"]["serverInfo"]["name"], "folisift");

    writeln!(
        stdin,
        "{}",
        serde_json::json!({"jsonrpc": "2.0", "method": "notifications/initialized"})
    )
    .expect("write initialized");
    writeln!(
        stdin,
        "{}",
        serde_json::json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}})
    )
    .expect("write tools/list");
    stdin.flush().expect("flush tools/list");
    line.clear();
    reader.read_line(&mut line).expect("read tools/list");
    let response: serde_json::Value = serde_json::from_str(&line).expect("tools JSON");
    let tools = response["result"]["tools"]
        .as_array()
        .expect("tools array")
        .iter()
        .filter_map(|tool| tool["name"].as_str())
        .collect::<Vec<_>>();
    assert_eq!(tools.len(), 32);
    for expected in [
        "read_source",
        "search_source",
        "build_guide",
        "check_coverage",
        "claims",
        "cards",
        "read_blocks",
        "section_brief",
        "read_section",
        "links",
        "graph",
        "diagram",
        "audit_guide",
        "qa_guide",
        "source_brief",
        "guide_plan",
        "cost_report",
        "visual_brief",
        "source_topics",
        "citation_snippet",
        "table_extract",
        "verify_citations",
        "source_insights",
    ] {
        assert!(tools.contains(&expected), "missing {expected}");
    }

    writeln!(
        stdin,
        "{}",
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 20,
            "method": "tools/call",
            "params": {"name": "list_guides", "arguments": {}}
        })
    )
    .expect("write list guides");
    stdin.flush().expect("flush list guides");
    line.clear();
    reader.read_line(&mut line).expect("read list guides");
    let response: serde_json::Value = serde_json::from_str(&line).expect("list guides JSON");
    assert!(
        response["result"]["structuredContent"]["guides"].is_array(),
        "{response}"
    );

    writeln!(
        stdin,
        "{}",
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 21,
            "method": "tools/call",
            "params": {"name": "style", "arguments": {"action": "list"}}
        })
    )
    .expect("write style list");
    stdin.flush().expect("flush style list");
    line.clear();
    reader.read_line(&mut line).expect("read style list");
    let response: serde_json::Value = serde_json::from_str(&line).expect("style list JSON");
    assert!(
        response["result"]["structuredContent"]["styles"].is_array(),
        "{response}"
    );

    writeln!(
        stdin,
        "{}",
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 22,
            "method": "tools/call",
            "params": {"name": "claims", "arguments": {"slug": "demo", "out": "data/claims.md"}}
        })
    )
    .expect("write claims");
    stdin.flush().expect("flush claims");
    line.clear();
    reader.read_line(&mut line).expect("read claims");
    let response: serde_json::Value = serde_json::from_str(&line).expect("claims JSON");
    assert!(
        response["result"]["structuredContent"]["total"]
            .as_u64()
            .is_some_and(|total| total >= 1),
        "{response}"
    );
    assert!(
        root.path().join("data/claims.md").is_file(),
        "claims report missing"
    );

    writeln!(
        stdin,
        "{}",
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 23,
            "method": "tools/call",
            "params": {"name": "cards", "arguments": {"slug": "demo", "out": "data/cards.csv", "review_intervals": [1, 3, 7]}}
        })
    )
    .expect("write cards");
    stdin.flush().expect("flush cards");
    line.clear();
    reader.read_line(&mut line).expect("read cards");
    let response: serde_json::Value = serde_json::from_str(&line).expect("cards JSON");
    assert!(
        response["result"]["structuredContent"]["cards"].as_u64() == Some(1),
        "{response}"
    );
    assert!(
        root.path().join("data/cards.csv").is_file(),
        "cards CSV missing"
    );

    writeln!(
        stdin,
        "{}",
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 24,
            "method": "tools/call",
            "params": {"name": "links", "arguments": {"action": "status", "guide": "demo"}}
        })
    )
    .expect("write links status");
    stdin.flush().expect("flush links status");
    line.clear();
    reader.read_line(&mut line).expect("read links status");
    let response: serde_json::Value = serde_json::from_str(&line).expect("links JSON");
    assert_eq!(
        response["result"]["structuredContent"]["blocks_ready"], false,
        "{response}"
    );

    let spec = serde_json::json!({
        "schema_version": 2,
        "id": "pipeline",
        "kind": "flow",
        "title": "Pipeline",
        "caption": "A source program becomes target code.",
        "alt": "A source program is analysed and converted into target code.",
        "source": "PD0, s. 4",
        "direction": "right",
        "layout": "tala",
        "nodes": [{"id": "src", "label": "Source"}, {"id": "out", "label": "Target"}],
        "edges": [{"from": "src", "to": "out", "label": "compile"}]
    });
    writeln!(
        stdin,
        "{}",
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {"name": "diagram", "arguments": {"action": "save", "guide": "demo", "spec": spec}}
        })
    )
    .expect("write diagram save");
    stdin.flush().expect("flush diagram save");
    line.clear();
    reader.read_line(&mut line).expect("read diagram save");
    let response: serde_json::Value = serde_json::from_str(&line).expect("diagram save JSON");
    assert!(response.to_string().contains("pipeline"), "{response}");

    writeln!(
        stdin,
        "{}",
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {"name": "diagram", "arguments": {"action": "status", "guide": "demo"}}
        })
    )
    .expect("write diagram status");
    stdin.flush().expect("flush diagram status");
    line.clear();
    reader.read_line(&mut line).expect("read diagram status");
    let response: serde_json::Value = serde_json::from_str(&line).expect("diagram status JSON");
    assert!(response.to_string().contains("missing"), "{response}");

    writeln!(
        stdin,
        "{}",
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "tools/call",
            "params": {"name": "diagram", "arguments": {"action": "render_plan", "guide": "demo", "id": "pipeline"}}
        })
    )
    .expect("write diagram plan");
    stdin.flush().expect("flush diagram plan");
    line.clear();
    reader.read_line(&mut line).expect("read diagram plan");
    let response: serde_json::Value = serde_json::from_str(&line).expect("diagram plan JSON");
    assert!(
        response.to_string().contains("folisift diagram build"),
        "{response}"
    );

    writeln!(
        stdin,
        "{}",
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 6,
            "method": "tools/call",
            "params": {"name": "style", "arguments": {"action": "preview", "id": "clean", "language": "en", "output": "output/clean.pdf"}}
        })
    )
    .expect("write style preview");
    stdin.flush().expect("flush style preview");
    line.clear();
    reader.read_line(&mut line).expect("read style preview");
    let response: serde_json::Value = serde_json::from_str(&line).expect("style preview JSON");
    assert!(
        !response.to_string().contains("\"isError\":true"),
        "{response}"
    );

    drop(stdin);
    let status = child.wait().expect("wait");
    assert!(status.success(), "server exited with {status}");
}

#[test]
fn mcp_core_profile_lists_ten_tools() {
    let root = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(root.path().join("guides")).expect("guides");
    std::fs::create_dir_all(root.path().join("sources")).expect("sources");
    let mut child = Command::new(env!("CARGO_BIN_EXE_folisift"))
        .arg("mcp")
        .arg("--root")
        .arg(root.path())
        .arg("--profile")
        .arg("core")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn folisift mcp");
    let mut stdin = child.stdin.take().expect("stdin");
    let stdout = child.stdout.take().expect("stdout");
    let mut reader = BufReader::new(stdout);
    writeln!(
        stdin,
        "{}",
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-11-25",
                "capabilities": {},
                "clientInfo": {"name": "profile-test", "version": "0"}
            }
        })
    )
    .expect("initialize");
    writeln!(
        stdin,
        "{}",
        serde_json::json!({"jsonrpc": "2.0", "method": "notifications/initialized"})
    )
    .expect("initialized");
    writeln!(
        stdin,
        "{}",
        serde_json::json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}})
    )
    .expect("tools/list");
    stdin.flush().expect("flush");
    let mut line = String::new();
    reader.read_line(&mut line).expect("initialize response");
    line.clear();
    reader.read_line(&mut line).expect("tools response");
    let response: serde_json::Value = serde_json::from_str(&line).expect("tools JSON");
    let tools = response["result"]["tools"].as_array().expect("tools");
    assert_eq!(tools.len(), 10, "{response}");
    let names = tools
        .iter()
        .filter_map(|tool| tool["name"].as_str())
        .collect::<Vec<_>>();
    assert!(names.contains(&"qa_guide"));
    assert!(!names.contains(&"graph"));
    child.kill().expect("kill");
    child.wait().expect("wait");
}
