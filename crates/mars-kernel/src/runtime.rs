use mars_runtime::{RuntimeBackend, RuntimeError, RustSession};
use mars_runtime_python::PythonOptions;
use std::collections::BTreeMap;

/// Construction options passed to every language backend factory.
/// Rust-specific fields (`dependencies`, `cargo_lock`, `offline`, `locked`)
/// are ignored by backends that do not support them.
#[derive(Clone)]
pub struct RuntimeConfig {
    pub backend: crate::kernel::RuntimeBackendKind,
    pub dependencies: BTreeMap<String, String>,
    pub cargo_lock: Option<String>,
    pub offline: bool,
    pub locked: bool,
    /// Python interpreter/version/package options; ignored by other backends.
    pub python_options: PythonOptions,
}

type BackendFactory = fn(&RuntimeConfig) -> Result<Box<dyn RuntimeBackend>, RuntimeError>;

/// Language backend registry. Adding a language means: implement
/// `RuntimeBackend` in its own crate, then append one entry here (and the
/// matching frontend `LanguageDescriptor`).
static BACKENDS: &[(&str, BackendFactory)] = &[
    ("rust", create_rust_runtime),
    ("cpp", create_cpp_runtime),
    ("julia", create_julia_runtime),
    ("python", create_python_runtime),
    ("r", create_r_runtime),
];

/// Languages with a registered execution backend.
pub fn supported_languages() -> impl Iterator<Item = &'static str> {
    BACKENDS.iter().map(|(language, _)| *language)
}

/// Creates the execution backend for a notebook language.
///
/// `backend` selects the implementation strategy for languages that have
/// more than one (currently only Rust: `Replay` or experimental `Evcxr`).
/// Rust-specific options (`dependencies`, `cargo_lock`, `offline`, `locked`)
/// are ignored by non-Rust backends.
pub fn create_runtime(
    language: &str,
    backend: crate::kernel::RuntimeBackendKind,
    dependencies: BTreeMap<String, String>,
    cargo_lock: Option<String>,
    offline: bool,
    locked: bool,
) -> Result<Box<dyn RuntimeBackend>, RuntimeError> {
    create_runtime_with_config(
        language,
        RuntimeConfig {
            backend,
            dependencies,
            cargo_lock,
            offline,
            locked,
            python_options: PythonOptions::default(),
        },
    )
}

/// Creates the execution backend with full construction options, including
/// per-language settings such as `python_options`.
pub fn create_runtime_with_config(
    language: &str,
    config: RuntimeConfig,
) -> Result<Box<dyn RuntimeBackend>, RuntimeError> {
    BACKENDS
        .iter()
        .find(|(id, _)| *id == language)
        .map(|(_, factory)| factory(&config))
        .unwrap_or_else(|| {
            Err(RuntimeError::Backend(format!(
                "unsupported notebook language: {language}"
            )))
        })
}

fn create_rust_runtime(config: &RuntimeConfig) -> Result<Box<dyn RuntimeBackend>, RuntimeError> {
    match config.backend {
        crate::kernel::RuntimeBackendKind::Replay => Ok(Box::new(RustSession::with_configuration(
            config.dependencies.clone(),
            config.cargo_lock.clone(),
            config.offline,
            config.locked,
        )?)),
        crate::kernel::RuntimeBackendKind::Evcxr => {
            #[cfg(feature = "experimental-evcxr")]
            {
                if config.locked || config.cargo_lock.is_some() {
                    return Err(RuntimeError::Backend(
                        "Evcxr does not support Project Mars embedded Cargo.lock or --locked execution"
                            .to_owned(),
                    ));
                }
                Ok(Box::new(
                    mars_runtime_evcxr::EvcxrSession::with_configuration(
                        config.dependencies.clone(),
                        config.offline,
                    )?,
                ))
            }
            #[cfg(not(feature = "experimental-evcxr"))]
            {
                Err(RuntimeError::Backend(
                    "Evcxr is experimental and not enabled; rebuild with --features experimental-evcxr"
                        .to_owned(),
                ))
            }
        }
    }
}

fn create_cpp_runtime(config: &RuntimeConfig) -> Result<Box<dyn RuntimeBackend>, RuntimeError> {
    mars_runtime_cpp::CppSession::new(
        config
            .dependencies
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
        config.cargo_lock.clone(),
        config.offline,
        config.locked,
    )
    .map(|session| Box::new(session) as Box<dyn RuntimeBackend>)
}

fn create_julia_runtime(_config: &RuntimeConfig) -> Result<Box<dyn RuntimeBackend>, RuntimeError> {
    mars_runtime_julia::JuliaSession::new()
        .map(|session| Box::new(session) as Box<dyn RuntimeBackend>)
}

fn create_python_runtime(config: &RuntimeConfig) -> Result<Box<dyn RuntimeBackend>, RuntimeError> {
    mars_runtime_python::PythonSession::with_options(config.python_options.clone())
        .map(|session| Box::new(session) as Box<dyn RuntimeBackend>)
}

fn create_r_runtime(_config: &RuntimeConfig) -> Result<Box<dyn RuntimeBackend>, RuntimeError> {
    mars_runtime_r::RSession::new().map(|session| Box::new(session) as Box<dyn RuntimeBackend>)
}

/// Route Evcxr's re-executed runtime process into its user-code loop.
pub fn runtime_hook() {
    #[cfg(feature = "experimental-evcxr")]
    mars_runtime_evcxr::runtime_hook();
}

/// Parses the `python` section of notebook metadata (`metadata.python`)
/// into `PythonOptions`. Missing or malformed sections yield defaults.
pub fn python_options_from_metadata(
    extra: &serde_json::Map<String, serde_json::Value>,
) -> PythonOptions {
    extra
        .get("python")
        .and_then(|value| serde_json::from_value(value.clone()).ok())
        .unwrap_or_default()
}
