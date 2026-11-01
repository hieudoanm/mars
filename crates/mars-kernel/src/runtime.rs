use mars_runtime::{RuntimeBackend, RuntimeError, RustSession};

pub fn create_runtime(
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
