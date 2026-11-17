//! MCP (Model Context Protocol) wire types.
//!
//! Implements the subset of the protocol Project Mars uses: `initialize`, `ping`,
//! `tools/list`, and `tools/call` over newline-delimited JSON-RPC.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const PROTOCOL_VERSION: &str = "2025-11-25";
pub const SERVER_NAME: &str = "mars";
pub const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// MCP tool annotations — the four hints hosts use to warn users before a
/// call. Every tool declares all four explicitly so directories that require
/// them accept the listing.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Annotations {
    pub read_only_hint: bool,
    pub destructive_hint: bool,
    pub idempotent_hint: bool,
    pub open_world_hint: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
    /// JSON Schema for `structuredContent`; `{"type":"object"}` for all tools
    /// since handlers return JSON objects (non-object payloads are wrapped).
    #[serde(rename = "outputSchema", skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<Value>,
    /// Always serialized; defaults when parsing tools from servers that omit
    /// annotations (the field is optional in the MCP spec).
    #[serde(default)]
    pub annotations: Annotations,
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
    /// Structured twin of the JSON text payload (always an object).
    #[serde(
        rename = "structuredContent",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub structured_content: Option<Value>,
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
            structured_content: None,
            is_error: None,
        }
    }

    /// JSON result: text carries the pretty payload and `structuredContent`
    /// carries the same data as an object (arrays/scalars are wrapped so the
    /// payload always validates against the tools' `outputSchema`).
    pub fn ok_json(value: &Value) -> Self {
        let structured = match value {
            Value::Object(_) => value.clone(),
            Value::Array(items) => json!({ "items": items }),
            other => json!({ "value": other }),
        };
        let mut result = Self::ok(serde_json::to_string_pretty(value)
            .unwrap_or_else(|_| value.to_string()));
        result.structured_content = Some(structured);
        result
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            content: vec![TextContent {
                kind: "text".to_owned(),
                text: message.into(),
            }],
            structured_content: None,
            is_error: Some(true),
        }
    }
}

/// Human-readable display title derived from the tool name
/// (`mars_notebook_new` -> `Mars Notebook New`).
pub(crate) fn titleize(name: &str) -> String {
    name.split(['_', '-'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn initialize_result() -> Value {
    serde_json::json!({
        "protocolVersion": PROTOCOL_VERSION,
        "capabilities": {
            "tools": { "listChanged": false },
            "prompts": { "listChanged": false },
            "resources": { "listChanged": false }
        },
        "serverInfo": { "name": SERVER_NAME, "version": SERVER_VERSION },
    })
}
