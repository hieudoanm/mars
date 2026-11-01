use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Output {
    Text {
        text: String,
    },
    Error {
        name: String,
        message: String,
        traceback: Vec<String>,
    },
    Html {
        html: String,
    },
    Image {
        mime_type: String,
        data: String,
    },
    Table {
        columns: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    Latex {
        latex: String,
    },
    Chart {
        spec: serde_json::Value,
    },
    MimeBundle {
        data: serde_json::Map<String, serde_json::Value>,
        metadata: serde_json::Map<String, serde_json::Value>,
    },
}
