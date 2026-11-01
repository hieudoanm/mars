use mars_core::ExecutionStatus;
use mars_protocol::{KernelRequest, KernelResponse, RequestEnvelope, ResponseEnvelope};
use mars_runtime::{RuntimeError, RustSession};
use std::{
    collections::BTreeMap,
    sync::{Arc, atomic::AtomicBool},
    time::Duration,
};

pub struct Kernel {
    runtime: RustSession,
    status: ExecutionStatus,
}

impl Kernel {
    pub fn new() -> Result<Self, KernelError> {
        Self::with_dependencies(BTreeMap::new())
    }

    pub fn with_dependencies(dependencies: BTreeMap<String, String>) -> Result<Self, KernelError> {
        Ok(Self {
            runtime: RustSession::with_dependencies(dependencies).map_err(KernelError::Runtime)?,
            status: ExecutionStatus::Idle,
        })
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
            KernelRequest::Reset => {
                self.runtime.reset();
                self.status = ExecutionStatus::Idle;
                KernelResponse::Reset {
                    status: self.status.clone(),
                }
            }
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
