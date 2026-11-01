use mars_core::ExecutionStatus;
use mars_protocol::{KernelRequest, KernelResponse, RequestEnvelope, ResponseEnvelope};
use mars_runtime::{ExecutionResult, RuntimeBackend, RuntimeError, RustSession};
use std::{
    collections::BTreeMap,
    sync::{Arc, atomic::AtomicBool},
    time::Duration,
};

pub struct Kernel {
    runtime: Box<dyn RuntimeBackend>,
    status: ExecutionStatus,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum RuntimeBackendKind {
    #[default]
    Replay,
    Evcxr,
}

impl RuntimeBackendKind {
    pub fn from_environment() -> Self {
        match std::env::var("MARS_EXPERIMENTAL_RUNTIME").as_deref() {
            Ok("evcxr") => Self::Evcxr,
            _ => Self::Replay,
        }
    }
}

impl Kernel {
    pub fn new() -> Result<Self, KernelError> {
        Self::with_dependencies(BTreeMap::new())
    }

    pub fn with_dependencies(dependencies: BTreeMap<String, String>) -> Result<Self, KernelError> {
        Self::with_backend(dependencies, RuntimeBackendKind::Replay, false)
    }

    pub fn with_backend(
        dependencies: BTreeMap<String, String>,
        backend: RuntimeBackendKind,
        offline: bool,
    ) -> Result<Self, KernelError> {
        Ok(Self {
            runtime: create_runtime(backend, dependencies, None, offline, false)
                .map_err(KernelError::Runtime)?,
            status: ExecutionStatus::Idle,
        })
    }

    pub fn with_configuration(
        dependencies: BTreeMap<String, String>,
        cargo_lock: Option<String>,
        offline: bool,
        locked: bool,
        backend: RuntimeBackendKind,
    ) -> Result<Self, KernelError> {
        Ok(Self {
            runtime: create_runtime(backend, dependencies, cargo_lock, offline, locked)
                .map_err(KernelError::Runtime)?,
            status: ExecutionStatus::Idle,
        })
    }

    pub fn cargo_lock(&self) -> Result<Option<String>, RuntimeError> {
        self.runtime.cargo_lock()
    }

    pub fn execute_source(
        &mut self,
        source: &str,
        timeout: Option<Duration>,
        cancellation: Arc<AtomicBool>,
    ) -> Result<ExecutionResult, RuntimeError> {
        self.runtime
            .execute_with_control(source, timeout, cancellation)
    }

    pub fn handle(&mut self, envelope: RequestEnvelope) -> ResponseEnvelope {
        self.handle_with_cancellation(envelope, Arc::new(AtomicBool::new(false)))
    }

    pub fn handle_with_cancellation(
        &mut self,
        envelope: RequestEnvelope,
        cancellation: Arc<AtomicBool>,
    ) -> ResponseEnvelope {
        let response = match envelope.request {
            KernelRequest::Execute {
                source,
                timeout_millis,
            } => self.execute(
                &source,
                timeout_millis.map(Duration::from_millis),
                cancellation,
            ),
            KernelRequest::Reset => match self.runtime.reset() {
                Ok(()) => {
                    self.status = ExecutionStatus::Idle;
                    KernelResponse::Reset {
                        status: self.status.clone(),
                    }
                }
                Err(error) => {
                    self.status = ExecutionStatus::Failed;
                    KernelResponse::Error {
                        message: error.to_string(),
                    }
                }
            },
            KernelRequest::Status => KernelResponse::Status {
                status: self.status.clone(),
                execution_count: self.runtime.execution_count(),
            },
        };
        ResponseEnvelope {
            id: envelope.id,
            response,
        }
    }

    fn execute(
        &mut self,
        source: &str,
        timeout: Option<Duration>,
        cancellation: Arc<AtomicBool>,
    ) -> KernelResponse {
        self.status = ExecutionStatus::Running;
        match self
            .runtime
            .execute_with_control(source, timeout, cancellation)
        {
            Ok(result) => {
                let failed = result
                    .outputs
                    .iter()
                    .any(|output| matches!(output, mars_core::Output::Error { .. }));
                self.status = if result.interrupted {
                    ExecutionStatus::Interrupted
                } else if failed {
                    ExecutionStatus::Failed
                } else {
                    ExecutionStatus::Succeeded
                };
                KernelResponse::Execution {
                    outputs: result.outputs,
                    execution_count: result.execution_count,
                    duration_millis: result.duration.as_millis(),
                }
            }
            Err(error) => {
                self.status = ExecutionStatus::Failed;
                KernelResponse::Error {
                    message: error.to_string(),
                }
            }
        }
    }
}

pub fn create_runtime(
    backend: RuntimeBackendKind,
    dependencies: BTreeMap<String, String>,
    cargo_lock: Option<String>,
    offline: bool,
    locked: bool,
) -> Result<Box<dyn RuntimeBackend>, RuntimeError> {
    match backend {
        RuntimeBackendKind::Replay => Ok(Box::new(RustSession::with_configuration(
            dependencies,
            cargo_lock,
            offline,
            locked,
        )?)),
        RuntimeBackendKind::Evcxr => {
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
                    "Evcxr is experimental and not enabled; rebuild with --features experimental-evcxr".to_owned(),
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

#[derive(Debug, thiserror::Error)]
pub enum KernelError {
    #[error("failed to initialize runtime")]
    Runtime(#[source] RuntimeError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use mars_protocol::{KernelRequest, RequestEnvelope};

    #[test]
    fn executes_and_reports_status() {
        let mut kernel = Kernel::new().expect("kernel");
        let execution = kernel.handle(RequestEnvelope {
            id: "execute".to_owned(),
            request: KernelRequest::Execute {
                source: "println!(\"hello\");".to_owned(),
                timeout_millis: None,
            },
        });
        assert!(matches!(
            execution.response,
            KernelResponse::Execution {
                execution_count: 1,
                ..
            }
        ));

        let status = kernel.handle(RequestEnvelope {
            id: "status".to_owned(),
            request: KernelRequest::Status,
        });
        assert!(matches!(
            status.response,
            KernelResponse::Status {
                status: ExecutionStatus::Succeeded,
                execution_count: 1
            }
        ));
    }

    #[test]
    fn reports_interrupted_status_for_timed_out_execution() {
        let mut kernel = Kernel::new().expect("kernel");
        let response = kernel.handle(RequestEnvelope {
            id: "timeout".to_owned(),
            request: KernelRequest::Execute {
                source: "std::thread::sleep(std::time::Duration::from_millis(100));".to_owned(),
                timeout_millis: Some(10),
            },
        });

        assert!(matches!(
            response.response,
            KernelResponse::Execution {
                execution_count: 0,
                ..
            }
        ));
        let status = kernel.handle(RequestEnvelope {
            id: "status".to_owned(),
            request: KernelRequest::Status,
        });
        assert!(matches!(
            status.response,
            KernelResponse::Status {
                status: ExecutionStatus::Interrupted,
                execution_count: 0
            }
        ));
    }
}
