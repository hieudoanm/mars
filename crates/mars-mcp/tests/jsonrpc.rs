use mars_mcp::jsonrpc::{
    INVALID_PARAMS, METHOD_NOT_FOUND, PARSE_ERROR, Request, Response, read_message, write_message,
};
use serde_json::{Value, json};
use std::io::Cursor;

#[test]
fn round_trips_request_and_response() {
    let request: Request = serde_json::from_value(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list"
    }))
    .expect("request");
    assert_eq!(request.id, Some(json!(1)));
    assert_eq!(request.method, "tools/list");
    assert!(request.params.is_none());

    let response = Response::ok(json!(1), json!({ "tools": [] }));
    let value = serde_json::to_value(&response).expect("serialize");
    assert_eq!(value["jsonrpc"], "2.0");
    assert_eq!(value["id"], 1);
    assert!(value.get("error").is_none());

    let error = Response::err(json!(2), METHOD_NOT_FOUND, "nope");
    let value = serde_json::to_value(&error).expect("serialize");
    assert_eq!(value["error"]["code"], METHOD_NOT_FOUND);
    assert!(value.get("result").is_none());
}

#[test]
fn response_error_omits_result_field() {
    let response = Response::err(Value::Null, PARSE_ERROR, "bad json");
    let encoded = serde_json::to_string(&response).expect("serialize");
    assert!(!encoded.contains("\"result\""));
    assert!(encoded.contains(&PARSE_ERROR.to_string()));
}

#[test]
fn read_message_handles_eof_invalid_and_valid_lines() {
    let mut empty = Cursor::new(Vec::new());
    assert!(read_message(&mut empty).expect("eof").is_none());

    let mut invalid = Cursor::new(b"not json\n".to_vec());
    assert!(read_message(&mut invalid).is_err());

    let mut valid = Cursor::new(b"{\"a\":1}\n{\"b\":2}\n".to_vec());
    assert_eq!(
        read_message(&mut valid).expect("first").expect("some"),
        json!({"a": 1})
    );
    assert_eq!(
        read_message(&mut valid).expect("second").expect("some"),
        json!({"b": 2})
    );
    assert!(read_message(&mut valid).expect("eof").is_none());
}

#[test]
fn write_message_emits_one_json_line() {
    let mut output = Vec::new();
    write_message(&mut output, &json!({ "ok": true })).expect("write");
    let text = String::from_utf8(output).expect("utf8");
    assert!(text.ends_with('\n'));
    assert_eq!(text.lines().count(), 1);
    let parsed: Value = serde_json::from_str(text.trim()).expect("json");
    assert_eq!(parsed["ok"], true);
}

#[test]
fn error_code_constants_match_json_rpc_spec() {
    assert_eq!(PARSE_ERROR, -32700);
    assert_eq!(INVALID_PARAMS, -32602);
    assert_eq!(METHOD_NOT_FOUND, -32601);
}
