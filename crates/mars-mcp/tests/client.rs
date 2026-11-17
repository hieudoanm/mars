use mars_mcp::McpClient;
use serde_json::json;

const MOCK_SERVER: &str = r#"
import sys, json
for line in sys.stdin:
    line = line.strip()
    if not line:
        continue
    try:
        msg = json.loads(line)
    except Exception:
        continue
    mid = msg.get("id")
    if mid is None:
        continue
    method = msg.get("method")
    # Noise the client must skip: a notification and a response for another id.
    sys.stdout.write(json.dumps({"jsonrpc": "2.0", "method": "notifications/progress", "params": {}}) + "\n")
    sys.stdout.write(json.dumps({"jsonrpc": "2.0", "id": 999999, "result": {}}) + "\n")
    sys.stdout.flush()
    params = msg.get("params") or {}
    if method == "tools/call" and params.get("name") == "fail_tool":
        sys.stdout.write(json.dumps({"jsonrpc": "2.0", "id": mid, "error": {"code": -32602, "message": "bad tool"}}) + "\n")
    elif method == "initialize":
        result = {"protocolVersion": "2025-06-18", "capabilities": {}, "serverInfo": {"name": "mock", "version": "0"}}
        sys.stdout.write(json.dumps({"jsonrpc": "2.0", "id": mid, "result": result}) + "\n")
    elif method == "tools/list":
        result = {"tools": [{"name": "mock_tool", "description": "d", "inputSchema": {"type": "object", "properties": {}}}]}
        sys.stdout.write(json.dumps({"jsonrpc": "2.0", "id": mid, "result": result}) + "\n")
    elif method == "tools/call":
        result = {"content": [{"type": "text", "text": "ok"}], "isError": False}
        sys.stdout.write(json.dumps({"jsonrpc": "2.0", "id": mid, "result": result}) + "\n")
    else:
        sys.stdout.write(json.dumps({"jsonrpc": "2.0", "id": mid, "result": {}}) + "\n")
    sys.stdout.flush()
"#;

fn python3() -> Option<String> {
    std::process::Command::new("python3")
        .arg("--version")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|_| "python3".to_owned())
}

#[test]
fn client_round_trips_against_mock_server() {
    let Some(python) = python3() else {
        eprintln!("python3 unavailable; skipping");
        return;
    };
    let mut client = McpClient::spawn(&[
        python,
        "-u".to_owned(),
        "-c".to_owned(),
        MOCK_SERVER.to_owned(),
    ])
    .expect("spawn mock server");

    client.initialize().expect("initialize");

    let tools = client.list_tools().expect("list tools");
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "mock_tool");

    let result = client
        .call_tool("mock_tool", json!({"x": 1}))
        .expect("call tool");
    assert!(!result.is_error.unwrap_or(false));

    let error = client
        .call_tool("fail_tool", json!({}))
        .expect_err("error response should surface");
    assert!(error.contains("bad tool"), "{error}");
}

#[test]
fn client_spawned_process_is_killed_on_drop() {
    // `cat` blocks forever; dropping the client must reap the child.
    let client = McpClient::spawn(&["cat".to_owned()]).expect("spawn cat");
    drop(client);
}

#[test]
fn client_handshake_against_test_server() {
    // In-process smoke test: run the real server on pipes via threads is
    // covered by server tests; here we only verify spawn plumbing fails
    // gracefully for a missing binary.
    let client = McpClient::spawn(&["definitely-not-a-real-binary-xyz".to_owned()]);
    assert!(client.is_err());
}

#[test]
fn spawn_rejects_empty_command() {
    assert!(McpClient::spawn(&[]).is_err());
}

#[test]
fn initialize_fails_when_server_exits_or_speaks_garbage() {
    // The test binary itself exits immediately after printing its test list,
    // so the MCP handshake cannot complete.
    let exe = std::env::current_exe().expect("current exe");
    let mut client = McpClient::spawn(&[exe.to_string_lossy().into_owned(), "--list".to_owned()])
        .expect("spawn");
    assert!(client.initialize().is_err());
}

const BLANK_LINE_SERVER: &str = r#"
import sys, json
for line in sys.stdin:
    line = line.strip()
    if not line:
        continue
    try:
        msg = json.loads(line)
    except Exception:
        continue
    mid = msg.get("id")
    if mid is None:
        continue
    sys.stdout.write("\n")
    sys.stdout.write("   \n")
    sys.stdout.flush()
    if msg.get("method") == "initialize":
        result = {"protocolVersion": "2025-06-18", "capabilities": {}, "serverInfo": {"name": "mock", "version": "0"}}
    else:
        result = {"tools": []}
    sys.stdout.write(json.dumps({"jsonrpc": "2.0", "id": mid, "result": result}) + "\n")
    sys.stdout.flush()
"#;

#[test]
fn client_skips_blank_lines_while_waiting() {
    let Some(python) = python3() else {
        eprintln!("python3 unavailable; skipping");
        return;
    };
    let mut client = McpClient::spawn(&[
        python,
        "-u".to_owned(),
        "-c".to_owned(),
        BLANK_LINE_SERVER.to_owned(),
    ])
    .expect("spawn mock server");
    client.initialize().expect("initialize");
    let tools = client.list_tools().expect("list tools");
    assert!(tools.is_empty());
}

#[test]
fn client_reports_server_closed_connection() {
    let Some(python) = python3() else {
        eprintln!("python3 unavailable; skipping");
        return;
    };
    // Server exits immediately without responding.
    let mut client = McpClient::spawn(&[
        python,
        "-c".to_owned(),
        "import sys; sys.exit(0)".to_owned(),
    ])
    .expect("spawn mock server");
    let err = client.initialize().expect_err("initialize must fail");
    assert!(
        err.contains("closed") || err.contains("response"),
        "err: {err}"
    );
}

const MALFORMED_SERVER: &str = r#"
import sys, json
for line in sys.stdin:
    line = line.strip()
    if not line:
        continue
    try:
        msg = json.loads(line)
    except Exception:
        continue
    mid = msg.get("id")
    if mid is None:
        continue
    method = msg.get("method")
    if method == "initialize":
        result = {"protocolVersion": "2025-06-18", "capabilities": {}, "serverInfo": {"name": "mock", "version": "0"}}
        sys.stdout.write(json.dumps({"jsonrpc": "2.0", "id": mid, "result": result}) + "\n")
    elif method == "tools/list":
        # Malformed: tools is not an array.
        sys.stdout.write(json.dumps({"jsonrpc": "2.0", "id": mid, "result": {"tools": "nope"}}) + "\n")
    elif method == "tools/call":
        # Garbage JSON line first, then a malformed result payload.
        sys.stdout.write("this is not json\n")
        sys.stdout.write(json.dumps({"jsonrpc": "2.0", "id": mid, "result": {"content": 42}}) + "\n")
    else:
        sys.stdout.write(json.dumps({"jsonrpc": "2.0", "id": mid, "result": {}}) + "\n")
    sys.stdout.flush()
"#;

#[test]
fn client_surfaces_malformed_server_payloads() {
    let Some(python) = python3() else {
        eprintln!("python3 unavailable; skipping");
        return;
    };
    let mut client = McpClient::spawn(&[
        python,
        "-u".to_owned(),
        "-c".to_owned(),
        MALFORMED_SERVER.to_owned(),
    ])
    .expect("spawn malformed server");
    client.initialize().expect("initialize");

    let error = client.list_tools().expect_err("bad tools list");
    assert!(error.contains("invalid tools/list"), "{error}");

    let error = client
        .call_tool("anything", json!({}))
        .expect_err("bad call result");
    assert!(error.contains("invalid JSON from server"), "{error}");
}
