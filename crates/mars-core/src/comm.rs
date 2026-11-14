use serde::{Deserialize, Serialize};

/// A Jupyter-style comm message exchanged between a language runtime and
/// the front end (widget state, custom channels). `comm_type` is one of
/// `comm_open`, `comm_msg`, or `comm_close`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommMessage {
    pub comm_id: String,
    pub comm_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_name: Option<String>,
    #[serde(default)]
    pub data: serde_json::Value,
    #[serde(default)]
    pub metadata: serde_json::Value,
}
