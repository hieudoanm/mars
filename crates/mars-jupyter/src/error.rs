use mars_document::DocumentError;

#[derive(Debug, thiserror::Error)]
pub enum JupyterError {
    #[error("failed to deserialize Jupyter notebook")]
    Deserialization(#[source] serde_json::Error),
    #[error("failed to serialize Jupyter notebook")]
    Serialization(#[source] serde_json::Error),
    #[error("Jupyter notebook format {0} is not supported")]
    UnsupportedFormat(u32),
    #[error("Jupyter cell type {0} is not supported")]
    UnsupportedCellType(String),
    #[error("Project Mars output type {0} cannot be exported to Jupyter")]
    UnsupportedOutput(String),
    #[error("invalid Project Mars notebook")]
    Document(#[source] DocumentError),
}
