//! MCP (Model Context Protocol) wire types.
//!
//! Implements the subset of the protocol Mars uses: `initialize`, `ping`,
//! `tools/list`, and `tools/call` over newline-delimited JSON-RPC.

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const PROTOCOL_VERSION: &str = "2025-03-26";
pub const SERVER_NAME: &str = "mars";
pub const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextContent {
    #[serde(rename = "type")]
    pub kind: String,
    pub text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CallToolResult {
    pub content: Vec<TextContent>,
    #[serde(rename = "isError", skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

impl CallToolResult {
    pub fn ok(text: impl Into<String>) -> Self {
        Self {
            content: vec![TextContent {
                kind: "text".to_owned(),
                text: text.into(),
            }],
            is_error: None,
        }
    }

    pub fn ok_json(value: &Value) -> Self {
        Self::ok(serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string()))
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            content: vec![TextContent {
                kind: "text".to_owned(),
                text: message.into(),
            }],
            is_error: Some(true),
        }
    }
}

pub fn initialize_result() -> Value {
    serde_json::json!({
        "protocolVersion": PROTOCOL_VERSION,
        "capabilities": { "tools": { "listChanged": false } },
        "serverInfo": { "name": SERVER_NAME, "version": SERVER_VERSION },
    })
}
