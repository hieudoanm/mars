//! Minimal JSON-RPC 2.0 framing over newline-delimited JSON.
//!
//! This is the transport shared by the MCP server (`mars mcp serve`) and the
//! MCP client: one JSON object per line on stdin/stdout, matching the framing
//! used by `mars kernel serve`.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{self, BufRead, Write};

pub const PARSE_ERROR: i64 = -32700;
pub const INVALID_REQUEST: i64 = -32600;
pub const METHOD_NOT_FOUND: i64 = -32601;
pub const INVALID_PARAMS: i64 = -32602;
pub const INTERNAL_ERROR: i64 = -32603;

/// An incoming JSON-RPC message. `id` is `None` for notifications.
#[derive(Debug, Deserialize, Serialize)]
pub struct Request {
    #[serde(default)]
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Option<Value>,
}

#[derive(Debug, Serialize, serde::Deserialize)]
pub struct RpcError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Debug, Serialize, serde::Deserialize)]
pub struct Response {
    pub jsonrpc: String,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
}

impl Response {
    pub fn ok(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_owned(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn err(id: Value, code: i64, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".to_owned(),
            id,
            result: None,
            error: Some(RpcError {
                code,
                message: message.into(),
                data: None,
            }),
        }
    }
}

/// Read one newline-delimited JSON value. Returns `Ok(None)` on EOF.
pub fn read_message(reader: &mut impl BufRead) -> io::Result<Option<Value>> {
    let mut line = String::new();
    match reader.read_line(&mut line) {
        Ok(0) => Ok(None),
        Ok(_) => serde_json::from_str(line.trim())
            .map(Some)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error)),
        Err(error) => Err(error),
    }
}

pub fn write_message(writer: &mut impl Write, value: &impl Serialize) -> io::Result<()> {
    serde_json::to_writer(&mut *writer, value)?;
    writer.write_all(b"\n")?;
    writer.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
