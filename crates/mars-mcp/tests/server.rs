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
    let tools = list["result"]["tools"].as_array().expect("tools");
    assert!(tools.len() >= 10);
    for tool in tools {
        let annotations = &tool["annotations"];
        for hint in [
            "readOnlyHint",
            "destructiveHint",
            "idempotentHint",
            "openWorldHint",
        ] {
            assert!(
                annotations.get(hint).is_some_and(|v| v.is_boolean()),
                "{} missing boolean annotation {hint}: {annotations}",
                tool["name"]
            );
        }
    }
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

#[test]
fn prompts_and_resources_dispatch() {
    let mut server = MarsMcpServer::new(RuntimeBackendKind::Replay, true, None);
    let call = |server: &mut MarsMcpServer, method: &str, params: Option<Value>| {
        server
            .handle(Request {
                id: Some(json!(1)),
                method: method.to_owned(),
                params,
            })
            .expect("response")
    };

    // prompts/list advertises the built-in prompts.
    let list = call(&mut server, "prompts/list", None);
    let prompts = list.result.expect("result")["prompts"]
        .as_array()
        .expect("prompts")
        .clone();
    for name in ["notebook_walkthrough", "debug_cell"] {
        assert!(prompts.iter().any(|p| p["name"] == name), "missing {name}");
        let got = call(&mut server, "prompts/get", Some(json!({"name": name})));
        assert!(got.result.is_some(), "{name} should resolve");
        assert!(got.result.expect("result")["messages"].is_array());
    }
    let unknown = call(&mut server, "prompts/get", Some(json!({"name": "nope"})));
    assert!(unknown.error.is_some());

    // resources/list starts empty; resources/read rejects bad URIs.
    let empty = call(&mut server, "resources/list", None);
    assert_eq!(empty.result.expect("result")["resources"], json!([]));
    let bad_uri = call(
        &mut server,
        "resources/read",
        Some(json!({"uri": "https://x"})),
    );
    assert!(bad_uri.error.is_some());

    // Preload a notebook: it appears as a resource and reads back as JSON.
    let dir = std::env::temp_dir().join(format!("mars-mcp-res-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("book.mars");
    mars_document::save_atomic(&mars_document::Notebook::new(), &path).expect("save");
    server.preload_notebook(&path).expect("preload");

    let listed = call(&mut server, "resources/list", None);
    let resources = listed.result.expect("result")["resources"]
        .as_array()
        .expect("resources")
        .clone();
    assert_eq!(resources.len(), 1);
    let uri = resources[0]["uri"].as_str().expect("uri").to_owned();
    assert!(uri.starts_with("mars://notebook/"));

    let read = call(&mut server, "resources/read", Some(json!({"uri": uri})));
    let contents = read.result.expect("result")["contents"]
        .as_array()
        .expect("contents")
        .clone();
    assert!(
        contents[0]["text"]
            .as_str()
            .expect("text")
            .contains("\"cells\"")
    );

    // A well-formed URI for a notebook that is not open is an error.
    let other = dir.join("other.mars");
    mars_document::save_atomic(&mars_document::Notebook::new(), &other).expect("save");
    let not_open = call(
        &mut server,
        "resources/read",
        Some(json!({"uri": format!("mars://notebook/{}", other.display())})),
    );
    assert!(not_open.error.is_some());

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn tools_call_rejects_missing_params_and_name() {
    let mut server = MarsMcpServer::new(RuntimeBackendKind::Replay, true, None);
    let bare = server
        .handle(Request {
            id: Some(json!(1)),
            method: "tools/call".to_owned(),
            params: None,
        })
        .expect("response");
    assert!(bare.error.is_some());

    let no_name = server
        .handle(Request {
            id: Some(json!(2)),
            method: "tools/call".to_owned(),
            params: Some(json!({"arguments": {}})),
        })
        .expect("response");
    assert!(no_name.error.is_some());
}
