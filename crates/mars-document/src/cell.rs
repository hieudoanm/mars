use mars_core::{CellId, CellType, Output};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Cell {
    pub id: CellId,
    pub cell_type: CellType,
    pub source: String,
    #[serde(default)]
    pub metadata: serde_json::Map<String, serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_count: Option<u64>,
    #[serde(default)]
    pub outputs: Vec<Output>,
}

impl Cell {
    pub fn new(cell_type: CellType, source: impl Into<String>) -> Self {
        Self {
            id: CellId::new(),
            cell_type,
            source: source.into(),
            metadata: serde_json::Map::new(),
            execution_count: None,
            outputs: Vec::new(),
        }
    }
}
