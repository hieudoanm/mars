use crate::error::KernelError;
use mars_core::ExecutionStatus;
use mars_protocol::{KernelRequest, KernelResponse, RequestEnvelope, ResponseEnvelope};
use mars_runtime::{ExecutionResult, RuntimeBackend, RuntimeError};
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
            runtime: crate::runtime::create_runtime(backend, dependencies, None, offline, false)
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
            runtime: crate::runtime::create_runtime(
                backend,
                dependencies,
                cargo_lock,
                offline,
                locked,
            )
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
