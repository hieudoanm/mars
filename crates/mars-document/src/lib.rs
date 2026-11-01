use mars_core::{CellId, CellType, Output};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

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
            extra: serde_json::Map::new(),
        }
    }
}

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

    pub fn validate(&self) -> Result<(), DocumentError> {
        if self.format_version != CURRENT_FORMAT_VERSION {
            return Err(DocumentError::UnsupportedFormatVersion(self.format_version));
        }
        let mut ids = HashSet::with_capacity(self.cells.len());
        for cell in &self.cells {
            if cell.id.as_str().trim().is_empty() {
                return Err(DocumentError::EmptyCellId);
            }
            if !ids.insert(&cell.id) {
                return Err(DocumentError::DuplicateCellId(cell.id.to_string()));
            }
            if matches!(cell.cell_type, CellType::Markdown | CellType::Raw)
                && (cell.execution_count.is_some() || !cell.outputs.is_empty())
            {
                return Err(DocumentError::NonCodeExecutionData(cell.id.to_string()));
            }
        }
        Ok(())
    }

    pub fn to_json_pretty(&self) -> Result<String, DocumentError> {
        self.validate()?;
        serde_json::to_string_pretty(self).map_err(DocumentError::Serialization)
    }

    pub fn from_json(input: &str) -> Result<Self, DocumentError> {
        let mut value: serde_json::Value =
            serde_json::from_str(input).map_err(DocumentError::Deserialization)?;
        migrate_value(&mut value)?;
        let notebook: Self =
            serde_json::from_value(value).map_err(DocumentError::Deserialization)?;
        notebook.validate()?;
        Ok(notebook)
    }

    pub fn save_atomic(&self, path: impl AsRef<Path>) -> Result<(), DocumentError> {
        let path = path.as_ref();
        let content = self.to_json_pretty()?;
        let temporary_path = temporary_path(path);
        fs::write(&temporary_path, format!("{content}\n")).map_err(|source| DocumentError::Io {
            path: temporary_path.clone(),
            source,
        })?;
        if let Err(source) = fs::rename(&temporary_path, path) {
            let _ = fs::remove_file(&temporary_path);
            return Err(DocumentError::Io {
                path: path.to_path_buf(),
                source,
            });
        }

        Ok(())
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, DocumentError> {
        let path = path.as_ref();
        let content = fs::read_to_string(path).map_err(|source| DocumentError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        Self::from_json(&content)
    }
}

fn migrate_value(value: &mut serde_json::Value) -> Result<(), DocumentError> {
    let version = value
        .get("format_version")
        .and_then(serde_json::Value::as_u64)
        .ok_or(DocumentError::MissingFormatVersion)? as u32;
    match version {
        1 => {
            let object = value
                .as_object_mut()
                .ok_or(DocumentError::InvalidDocumentShape)?;
            object.insert(
                "format_version".to_owned(),
                serde_json::Value::from(CURRENT_FORMAT_VERSION),
            );
            Ok(())
        }
        CURRENT_FORMAT_VERSION => Ok(()),
        other => Err(DocumentError::UnsupportedFormatVersion(other)),
    }
}

#[derive(Clone, Debug)]
pub struct NotebookHistory {
    past: Vec<Notebook>,
    future: Vec<Notebook>,
    capacity: usize,
}

impl NotebookHistory {
    pub fn new(initial: Notebook, capacity: usize) -> Self {
        Self {
            past: vec![initial],
            future: Vec::new(),
            capacity: capacity.max(1),
        }
    }

    pub fn current(&self) -> Option<&Notebook> {
        self.past.last()
    }

    pub fn record(&mut self, notebook: Notebook) {
        self.past.push(notebook);
        self.future.clear();
        if self.past.len() > self.capacity {
            self.past.remove(0);
        }
    }

    pub fn undo(&mut self) -> Option<&Notebook> {
        if self.past.len() <= 1 {
            return None;
        }
        let current = self.past.pop()?;
        self.future.push(current);
        self.past.last()
    }

    pub fn redo(&mut self) -> Option<&Notebook> {
        let next = self.future.pop()?;
        self.past.push(next);
        self.past.last()
    }
}

fn temporary_path(path: &Path) -> PathBuf {
    let mut temporary = path.to_path_buf();
    let suffix = path
        .file_name()
        .and_then(|name| name.to_str())
        .map_or_else(|| "notebook".to_owned(), ToOwned::to_owned);
    temporary.set_file_name(format!(".{suffix}.tmp"));
    temporary
}

#[derive(Debug, thiserror::Error)]
pub enum DocumentError {
    #[error("cell index {0} is out of bounds")]
    CellIndexOutOfBounds(usize),
    #[error("duplicate cell ID: {0}")]
    DuplicateCellId(String),
    #[error("cell ID cannot be empty")]
    EmptyCellId,
    #[error("format version {0} is not supported")]
    UnsupportedFormatVersion(u32),
    #[error("notebook is missing format_version")]
    MissingFormatVersion,
    #[error("notebook root must be a JSON object")]
    InvalidDocumentShape,
    #[error("non-code cell {0} cannot contain execution data")]
    NonCodeExecutionData(String),
    #[error("failed to serialize notebook")]
    Serialization(#[source] serde_json::Error),
    #[error("failed to deserialize notebook")]
    Deserialization(#[source] serde_json::Error),
    #[error("I/O operation failed for {path}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_notebook_json() {
        let mut notebook = Notebook::new();
        notebook
            .add_cell(Cell::new(CellType::Markdown, "# Hello"))
            .expect("unique ID");
        notebook
            .add_cell(Cell::new(CellType::Code, "1 + 1"))
            .expect("unique ID");

        let json = notebook.to_json_pretty().expect("serializes");
        let restored = Notebook::from_json(&json).expect("deserializes");

        assert_eq!(notebook, restored);
    }

    #[test]
    fn supports_cell_operations() {
        let mut notebook = Notebook::new();
        notebook
            .add_cell(Cell::new(CellType::Code, "first"))
            .expect("unique ID");
        notebook
            .insert_cell(0, Cell::new(CellType::Markdown, "intro"))
            .expect("valid index");
        notebook.move_cell(0, 1).expect("valid indexes");

        assert_eq!(notebook.cell(0).expect("cell exists").source, "first");
        assert_eq!(
            notebook.remove_cell(1).expect("cell exists").source,
            "intro"
        );
    }

    #[test]
    fn migrates_format_version_one() {
        let notebook = Notebook::from_json(
            r#"{"format_version":1,"metadata":{"language":"rust"},"cells":[]}"#,
        )
        .expect("migrates");
        assert_eq!(notebook.format_version, CURRENT_FORMAT_VERSION);
    }

    #[test]
    fn supports_undo_and_redo_history() {
        let initial = Notebook::new();
        let mut history = NotebookHistory::new(initial.clone(), 4);
        let mut changed = initial;
        changed
            .add_cell(Cell::new(CellType::Code, "1 + 1"))
            .expect("adds");
        history.record(changed.clone());
        assert_eq!(history.current(), Some(&changed));
        assert!(history.undo().is_some());
        assert!(history.redo().is_some());
    }

    #[test]
    fn rejects_execution_data_on_markdown() {
        let mut notebook = Notebook::new();
        let mut cell = Cell::new(CellType::Markdown, "text");
        cell.outputs.push(Output::Text {
            text: "invalid".to_owned(),
        });
        notebook.cells.push(cell);

        assert!(matches!(
            notebook.validate(),
            Err(DocumentError::NonCodeExecutionData(_))
        ));
    }
}
