use mars_kernel::RuntimeBackendKind;
use mars_mcp::jsonrpc::{METHOD_NOT_FOUND, PARSE_ERROR, Request};
use mars_mcp::server::{MarsMcpServer, serve};
use serde_json::{Value, json};
use std::io::Cursor;

fn run_session(input: &str) -> String {
    let mut output = Vec::new();
    let mut server = MarsMcpServer::new(RuntimeBackendKind::Replay, true, None);
    serve(
        Cursor::new(input.as_bytes().to_vec()),
        &mut output,
        &mut server,
    )
    .expect("serve");
    String::from_utf8(output).expect("utf8")
}

#[test]
fn initialize_and_list_tools() {
    let input = concat!(
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
        "\n",
        r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
        "\n",
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#,
        "\n",
    );
    let output = run_session(input);
    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines.len(), 2, "notifications produce no response");
    let init: Value = serde_json::from_str(lines[0]).expect("init json");
    assert_eq!(init["result"]["serverInfo"]["name"], "mars");
    let list: Value = serde_json::from_str(lines[1]).expect("list json");
    assert!(list["result"]["tools"].as_array().expect("tools").len() >= 10);
}

#[test]
fn rejects_unknown_method() {
    let output = run_session(r#"{"jsonrpc":"2.0","id":9,"method":"bogus"}"#);
    let value: Value = serde_json::from_str(output.trim()).expect("json");
    assert_eq!(value["error"]["code"], METHOD_NOT_FOUND);
}

#[test]
fn malformed_json_gets_parse_error() {
    let output = run_session("not json\n");
    let value: Value = serde_json::from_str(output.trim()).expect("json");
    assert_eq!(value["error"]["code"], PARSE_ERROR);
}

#[test]
fn ping_returns_empty_result() {
    let output = run_session(r#"{"jsonrpc":"2.0","id":5,"method":"ping"}"#);
    let value: Value = serde_json::from_str(output.trim()).expect("json");
    assert_eq!(value["id"], 5);
    assert_eq!(value["result"], serde_json::json!({}));
}

#[test]
fn tools_call_validates_params() {
    let output = run_session(concat!(
        r#"{"jsonrpc":"2.0","id":1,"method":"tools/call"}"#,
        "\n",
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{}}"#,
        "\n",
        r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"arguments":{}}}"#,
        "\n",
    ));
    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines.len(), 3);
    for line in lines {
        let value: Value = serde_json::from_str(line).expect("json");
        assert!(value["error"].is_object(), "expected error: {value}");
    }
}

#[test]
fn tools_call_unknown_tool_is_tool_error_not_rpc_error() {
    let output = run_session(
        r#"{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"mars_nope","arguments":{}}}"#,
    );
    let value: Value = serde_json::from_str(output.trim()).expect("json");
    assert!(value["error"].is_null() || value.get("error").is_none());
    assert_eq!(value["result"]["isError"], true);
    assert!(
        value["result"]["content"][0]["text"]
            .as_str()
            .is_some_and(|text| text.contains("unknown tool"))
    );
}

#[test]
fn tools_call_document_and_exec_tools() {
    let dir = std::env::temp_dir().join(format!("mars-mcp-server-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("server.mars");
    mars_document::save_atomic(&mars_document::Notebook::new(), &path).expect("save");
    let path_json = serde_json::to_string(&path.to_string_lossy().to_string()).expect("json");

    let input = format!(
        concat!(
            r#"{{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{{"name":"mars_notebook_open","arguments":{{"path":{path}}}}}}}"#,
            "\n",
            r#"{{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{{"name":"mars_kernel_status","arguments":{{"language":"rust"}}}}}}"#,
            "\n",
        ),
        path = path_json
    );
    let output = run_session(&input);
    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines.len(), 2);
    for line in lines {
        let value: Value = serde_json::from_str(line).expect("json");
        assert!(value.get("error").is_none(), "unexpected error: {value}");
        assert!(value["result"]["content"].is_array());
    }

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn preload_notebook_loads_into_session() {
    let dir = std::env::temp_dir().join(format!("mars-mcp-preload-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("notebook.mars");
    mars_document::save_atomic(&mars_document::Notebook::new(), &path).expect("save");

    let mut server = MarsMcpServer::new(RuntimeBackendKind::Replay, true, None);
    server.preload_notebook(&path).expect("preload");
    let response = server
        .handle(Request {
            id: Some(json!(1)),
            method: "tools/call".to_owned(),
            params: Some(json!({
                "name": "mars_notebook_inspect",
                "arguments": { "path": path.to_string_lossy() }
            })),
        })
        .expect("response");
    assert!(response.result.is_some());

    // Preloading a missing notebook errors.
    assert!(server.preload_notebook(&dir.join("missing.mars")).is_err());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn notifications_and_non_request_json_get_no_response_or_parse_error() {
    let mut server = MarsMcpServer::new(RuntimeBackendKind::Replay, true, None);
    // A JSON-RPC notification (no id) still dispatches but returns None.
    let notification = Request {
        id: None,
        method: "notifications/initialized".to_owned(),
        params: None,
    };
    assert!(server.handle(notification).is_none());

    // Serve loop: a request-shaped message with unknown method gets a response;
    // a bare JSON value that is not a Request gets a parse error.
    let input = "{\"jsonrpc\":\"2.0\",\"id\":5,\"method\":\"bogus/method\"}\n[1,2,3]\n";
    let mut output = Vec::new();
    serve(input.as_bytes(), &mut output, &mut server).expect("serve");
    let text = String::from_utf8(output).expect("utf8");
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 2);
    let first: serde_json::Value = serde_json::from_str(lines[0]).expect("json");
    assert_eq!(first["id"], json!(5));
    assert!(first["error"].is_object() || first["result"].is_object());
    let second: serde_json::Value = serde_json::from_str(lines[1]).expect("json");
    assert!(second["error"].is_object());
}

#[test]
fn tools_call_dispatches_lab_tools() {
    // Point lab storage at a temp dir; settings are process-global but no
    // other test in this binary touches lab state.
    let dir = std::env::temp_dir().join(format!("mars-mcp-server-lab-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    mars_lab::config::set_settings(mars_lab::config::Settings::new(
        format!("sqlite://{}", dir.join("lab.db").display()),
        dir.join("data"),
    ));

    let input = concat!(
        r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"mars_lab_algorithms","arguments":{}}}"#,
        "\n",
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"mars_lab_datasets","arguments":{}}}"#,
        "\n",
        r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"mars_lab_jobs","arguments":{"limit":10}}}"#,
        "\n",
    );
    let output = run_session(input);
    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines.len(), 3);
    for line in lines {
        let value: Value = serde_json::from_str(line).expect("json");
        assert!(value.get("error").is_none(), "unexpected error: {value}");
        assert!(value["result"]["content"].is_array());
    }

    let _ = std::fs::remove_dir_all(&dir);
}
