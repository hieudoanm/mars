use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CellId(String);

impl CellId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn from_string(value: impl Into<String>) -> Result<Self, CoreError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(CoreError::InvalidCellId);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for CellId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for CellId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CellType {
    Markdown,
    Code,
    Raw,
}

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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    Idle,
    Running,
    Succeeded,
    Failed,
    Interrupted,
}

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("cell ID cannot be empty")]
    InvalidCellId,
}
