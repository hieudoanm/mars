use crate::cell::Cell;
use crate::error::DocumentError;
use mars_core::PaperSize;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const CURRENT_FORMAT_VERSION: u32 = 2;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NotebookMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub dependencies: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cargo_lock: Option<String>,
    #[serde(default, skip_serializing_if = "PaperSize::is_default")]
    pub page_size: PaperSize,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

fn default_language() -> String {
    "rust".to_owned()
}

impl Default for NotebookMetadata {
    fn default() -> Self {
        Self {
            title: None,
            language: default_language(),
            dependencies: BTreeMap::new(),
            cargo_lock: None,
            page_size: PaperSize::default(),
            extra: serde_json::Map::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Notebook {
    pub format_version: u32,
    #[serde(default)]
    pub metadata: NotebookMetadata,
    pub cells: Vec<Cell>,
}

impl Default for Notebook {
    fn default() -> Self {
        Self::new()
    }
}

impl Notebook {
    pub fn new() -> Self {
        Self {
            format_version: CURRENT_FORMAT_VERSION,
            metadata: NotebookMetadata::default(),
            cells: Vec::new(),
        }
    }

    pub fn add_cell(&mut self, cell: Cell) -> Result<(), DocumentError> {
        if self.cells.iter().any(|existing| existing.id == cell.id) {
            return Err(DocumentError::DuplicateCellId(cell.id.to_string()));
        }
        self.cells.push(cell);
        Ok(())
    }

    pub fn insert_cell(&mut self, index: usize, cell: Cell) -> Result<(), DocumentError> {
        if index > self.cells.len() {
            return Err(DocumentError::CellIndexOutOfBounds(index));
        }
        if self.cells.iter().any(|existing| existing.id == cell.id) {
            return Err(DocumentError::DuplicateCellId(cell.id.to_string()));
        }
        self.cells.insert(index, cell);
        Ok(())
    }

    pub fn remove_cell(&mut self, index: usize) -> Result<Cell, DocumentError> {
        if index >= self.cells.len() {
            return Err(DocumentError::CellIndexOutOfBounds(index));
        }
        Ok(self.cells.remove(index))
    }

    pub fn move_cell(&mut self, from: usize, to: usize) -> Result<(), DocumentError> {
        if from >= self.cells.len() || to >= self.cells.len() {
            return Err(DocumentError::CellIndexOutOfBounds(from.max(to)));
        }
        let cell = self.cells.remove(from);
        self.cells.insert(to, cell);
        Ok(())
    }

    pub fn cell(&self, index: usize) -> Result<&Cell, DocumentError> {
        self.cells
            .get(index)
            .ok_or(DocumentError::CellIndexOutOfBounds(index))
    }

    pub fn cell_mut(&mut self, index: usize) -> Result<&mut Cell, DocumentError> {
        self.cells
            .get_mut(index)
            .ok_or(DocumentError::CellIndexOutOfBounds(index))
    }
}
