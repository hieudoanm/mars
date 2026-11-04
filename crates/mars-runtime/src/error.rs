use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("cannot execute an empty code cell")]
    EmptySource,
    #[error("invalid Cargo dependency declaration: {name}")]
    InvalidDependency { name: String },
    #[error("failed to access runtime file {path}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("rustc is unavailable")]
    CompilerUnavailable(#[source] std::io::Error),
    #[error("failed to start compiled notebook program")]
    Execution(#[source] std::io::Error),
    #[error("runtime backend failed: {0}")]
    Backend(String),
}
