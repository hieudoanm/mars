use mars_mcp::jsonrpc::{Request, Response, write_message};
use serde_json::Value;

#[test]
fn round_trips_request_and_response() {
    let request: Request =
        serde_json::from_str(r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#).expect("parse");
    assert_eq!(request.id, Some(Value::from(1)));
    assert_eq!(request.method, "ping");
    assert!(request.params.is_none());

    let mut buf = Vec::new();
    write_message(
        &mut buf,
        &Response::ok(Value::from(1), serde_json::json!({})),
    )
    .expect("write");
    assert!(buf.ends_with(b"\n"));
    let text = String::from_utf8(buf).expect("utf8");
    let value: Value = serde_json::from_str(text.trim()).expect("json");
    assert_eq!(value["result"], serde_json::json!({}));
}
