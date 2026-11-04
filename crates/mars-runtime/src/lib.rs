pub mod backend;
pub mod cancellation;
pub mod compilation;
pub mod diagnostics;
pub mod error;
pub mod execution;
pub mod expression;
pub mod manifest;
pub mod output;
pub mod session;
pub mod session_execution;
pub mod toml;

pub use backend::RuntimeBackend;
pub use cancellation::CancellationToken;
pub use compilation::{cargo_manifest, format_compiler_diagnostics, validate_dependencies};
pub use error::RuntimeError;
pub use output::{
    ExecutionResult, extract_standalone_svg, has_visible_text, normalize_stdout,
    parse_evcxr_content,
};
pub use session::RustSession;
