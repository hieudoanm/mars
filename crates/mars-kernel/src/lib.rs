pub mod error;
pub mod kernel;
pub mod runtime;

pub use error::KernelError;
pub use kernel::{Kernel, RuntimeBackendKind};
pub use runtime::{create_runtime, runtime_hook};

#[cfg(test)]
mod tests {
    use super::*;
    use mars_core::ExecutionStatus;
    use mars_protocol::{KernelRequest, KernelResponse, RequestEnvelope};

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
