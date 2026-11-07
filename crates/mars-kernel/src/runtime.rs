use mars_runtime::{RuntimeBackend, RuntimeError, RustSession};

/// Creates the execution backend for a notebook language.
///
/// `backend` selects the implementation strategy for languages that have
/// more than one (currently only Rust: `Replay` or experimental `Evcxr`).
/// Rust-specific options (`dependencies`, `cargo_lock`, `offline`, `locked`)
/// are ignored by non-Rust backends.
pub fn create_runtime(
    language: &str,
    backend: crate::kernel::RuntimeBackendKind,
    dependencies: std::collections::BTreeMap<String, String>,
    cargo_lock: Option<String>,
    offline: bool,
    locked: bool,
) -> Result<Box<dyn RuntimeBackend>, RuntimeError> {
    match language {
        "rust" => create_rust_runtime(backend, dependencies, cargo_lock, offline, locked),
        "cpp" => mars_runtime_cpp::CppSession::new(
            dependencies.into_iter().collect(),
            cargo_lock,
            offline,
            locked,
        )
        .map(|session| Box::new(session) as Box<dyn RuntimeBackend>),
        "julia" => mars_runtime_julia::JuliaSession::new()
            .map(|session| Box::new(session) as Box<dyn RuntimeBackend>),
        "python" => mars_runtime_python::PythonSession::new()
            .map(|session| Box::new(session) as Box<dyn RuntimeBackend>),
        "r" => mars_runtime_r::RSession::new()
            .map(|session| Box::new(session) as Box<dyn RuntimeBackend>),
        other => Err(RuntimeError::Backend(format!(
            "unsupported notebook language: {other}"
        ))),
    }
}

fn create_rust_runtime(
    backend: crate::kernel::RuntimeBackendKind,
    dependencies: std::collections::BTreeMap<String, String>,
    cargo_lock: Option<String>,
    offline: bool,
    locked: bool,
) -> Result<Box<dyn RuntimeBackend>, RuntimeError> {
    match backend {
        crate::kernel::RuntimeBackendKind::Replay => Ok(Box::new(RustSession::with_configuration(
            dependencies,
            cargo_lock,
            offline,
            locked,
        )?)),
        crate::kernel::RuntimeBackendKind::Evcxr => {
            #[cfg(feature = "experimental-evcxr")]
            {
                if locked || cargo_lock.is_some() {
                    return Err(RuntimeError::Backend(
                        "Evcxr does not support Mars embedded Cargo.lock or --locked execution"
                            .to_owned(),
                    ));
                }
                Ok(Box::new(
                    mars_runtime_evcxr::EvcxrSession::with_configuration(dependencies, offline)?,
                ))
            }
            #[cfg(not(feature = "experimental-evcxr"))]
            {
                let _ = (dependencies, cargo_lock, offline, locked);
                Err(RuntimeError::Backend(
                    "Evcxr is experimental and not enabled; rebuild with --features experimental-evcxr"
                        .to_owned(),
                ))
            }
        }
    }
}

/// Route Evcxr's re-executed runtime process into its user-code loop.
pub fn runtime_hook() {
    #[cfg(feature = "experimental-evcxr")]
    mars_runtime_evcxr::runtime_hook();
}
