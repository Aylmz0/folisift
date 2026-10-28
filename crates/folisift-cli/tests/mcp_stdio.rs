use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

#[test]
fn mcp_stdio_lists_tools() {
    let root = tempfile::tempdir().expect("tempdir");
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
    assert_eq!(tools.len(), 11);
    for expected in [
        "read_source",
        "search_source",
        "build_guide",
        "check_coverage",
    ] {
        assert!(tools.contains(&expected), "missing {expected}");
    }

    drop(stdin);
    let status = child.wait().expect("wait");
    assert!(status.success(), "server exited with {status}");
}
