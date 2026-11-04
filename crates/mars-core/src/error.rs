use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("cell ID cannot be empty")]
    InvalidCellId,
}
