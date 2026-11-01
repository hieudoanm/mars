use std::path::PathBuf;

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
