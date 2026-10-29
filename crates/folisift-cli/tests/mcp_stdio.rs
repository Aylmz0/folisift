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
          }
        }"#,
    )
    .expect("config");
    std::fs::write(root.path().join("guides/demo/guide.md"), "# Demo\n").expect("guide");
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
    assert_eq!(tools.len(), 13);
    for expected in [
        "read_source",
        "search_source",
        "build_guide",
        "check_coverage",
        "diagram",
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
