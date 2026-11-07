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
                language: None,
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
    fn streams_phase_and_output_events_during_execution() {
        let mut kernel = Kernel::new().expect("kernel");
        let mut events = Vec::new();
        let response = kernel.execute_with_events(
            "println!(\"streamed\");",
            None,
            std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            &mut |event| events.push(event),
        );

        assert!(matches!(
            response,
            KernelResponse::Execution {
                execution_count: 1,
                ..
            }
        ));
        assert!(events.iter().any(|event| matches!(
            event,
            mars_runtime::StreamEvent::Phase {
                phase: mars_runtime::StreamPhase::Compiling
            }
        )));
        assert!(events.iter().any(|event| matches!(
            event,
            mars_runtime::StreamEvent::Phase {
                phase: mars_runtime::StreamPhase::Running
            }
        )));
        let streamed = events
            .iter()
            .filter_map(|event| match event {
                mars_runtime::StreamEvent::Chunk {
                    stream: mars_runtime::ChunkStream::Stdout,
                    data,
                } => Some(data.as_str()),
                _ => None,
            })
            .collect::<String>();
        assert!(streamed.contains("streamed"));
    }

    #[test]
    fn executes_python_cells() {
        let mut kernel =
            Kernel::for_language("python", RuntimeBackendKind::Replay).expect("python kernel");
        let execution = kernel.handle(RequestEnvelope {
            id: "execute".to_owned(),
            request: KernelRequest::Execute {
                source: "value = 6\nvalue * 7".to_owned(),
                timeout_millis: None,
                language: None,
            },
        });
        assert!(matches!(
            execution.response,
            KernelResponse::Execution {
                execution_count: 1,
                ..
            }
        ));
    }

    #[test]
    fn executes_r_cells() {
        let Ok(mut kernel) = Kernel::for_language("r", RuntimeBackendKind::Replay) else {
            // Rscript is not installed on this machine.
            return;
        };
        let execution = kernel.handle(RequestEnvelope {
            id: "execute-r".to_owned(),
            request: KernelRequest::Execute {
                source: "value <- 6\nvalue * 7".to_owned(),
                timeout_millis: None,
                language: None,
            },
        });
        assert!(matches!(
            execution.response,
            KernelResponse::Execution {
                execution_count: 1,
                ..
            }
        ));
    }

    #[test]
    fn executes_julia_cells() {
        let Ok(mut kernel) = Kernel::for_language("julia", RuntimeBackendKind::Replay) else {
            // Julia is not installed on this machine.
            return;
        };
        let execution = kernel.handle(RequestEnvelope {
            id: "execute-julia".to_owned(),
            request: KernelRequest::Execute {
                source: "value = 6\nvalue * 7".to_owned(),
                timeout_millis: None,
                language: None,
            },
        });
        assert!(matches!(
            execution.response,
            KernelResponse::Execution {
                execution_count: 1,
                ..
            }
        ));
    }

    #[test]
    fn rejects_unknown_languages() {
        assert!(Kernel::for_language("cobol", RuntimeBackendKind::Replay).is_err());
    }

    #[test]
    fn reports_interrupted_status_for_timed_out_execution() {
        let mut kernel = Kernel::new().expect("kernel");
        let response = kernel.handle(RequestEnvelope {
            id: "timeout".to_owned(),
            request: KernelRequest::Execute {
                source: "std::thread::sleep(std::time::Duration::from_millis(100));".to_owned(),
                timeout_millis: Some(10),
                language: None,
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
