use mars_kernel::RuntimeBackendKind;
use mars_mcp::jsonrpc::{METHOD_NOT_FOUND, PARSE_ERROR};
use mars_mcp::server::{MarsMcpServer, serve};
use serde_json::Value;
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
