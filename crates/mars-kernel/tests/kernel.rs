use mars_core::ExecutionStatus;
use mars_kernel::{Kernel, RuntimeBackendKind};
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
    assert!(
        Kernel::with_backend(
            std::collections::BTreeMap::new(),
            RuntimeBackendKind::Replay,
            true
        )
        .is_ok()
    );
    assert!(
        Kernel::with_configuration(
            std::collections::BTreeMap::new(),
            Some("lock".to_owned()),
            true,
            true,
            RuntimeBackendKind::Replay,
        )
        .is_ok()
    );
    assert!(
        Kernel::for_language_with_configuration(
            "rust",
            std::collections::BTreeMap::new(),
            None,
            false,
            false,
            RuntimeBackendKind::Replay,
        )
        .is_ok()
    );
}

#[test]
fn handles_non_execute_requests() {
    let mut kernel = Kernel::new().expect("kernel");

    // Variables on an empty session reports no bindings without compiling.
    let variables = kernel.handle(RequestEnvelope {
        id: "vars".to_owned(),
        request: KernelRequest::Variables,
    });
    assert!(matches!(
        variables.response,
        KernelResponse::Variables { variables } if variables.is_empty()
    ));

    // Diagnose with blank source short-circuits.
    let diagnose = kernel.handle(RequestEnvelope {
        id: "diag".to_owned(),
        request: KernelRequest::Diagnose {
            source: "  ".to_owned(),
        },
    });
    assert!(matches!(
        diagnose.response,
        KernelResponse::Diagnostics { diagnostics } if diagnostics.is_empty()
    ));

    // Input without an active execution is an error, not a hang.
    let input = kernel.handle(RequestEnvelope {
        id: "in".to_owned(),
        request: KernelRequest::Input {
            text: "answer".to_owned(),
        },
    });
    assert!(matches!(input.response, KernelResponse::Error { .. }));

    // Cancel is acknowledged and does not change the idle status.
    let cancel = kernel.handle(RequestEnvelope {
        id: "cancel".to_owned(),
        request: KernelRequest::Cancel,
    });
    assert!(matches!(cancel.response, KernelResponse::Cancelled));

    // Reset returns the kernel to idle.
    let reset = kernel.handle(RequestEnvelope {
        id: "reset".to_owned(),
        request: KernelRequest::Reset,
    });
    assert!(matches!(
        reset.response,
        KernelResponse::Reset {
            status: ExecutionStatus::Idle
        }
    ));
}

#[test]
fn empty_execute_source_returns_error_response() {
    let mut kernel = Kernel::new().expect("kernel");
    let response = kernel.handle(RequestEnvelope {
        id: "empty".to_owned(),
        request: KernelRequest::Execute {
            source: "   ".to_owned(),
            timeout_millis: None,
            language: None,
        },
    });
    assert!(matches!(response.response, KernelResponse::Error { .. }));
    let status = kernel.handle(RequestEnvelope {
        id: "status".to_owned(),
        request: KernelRequest::Status,
    });
    assert!(matches!(
        status.response,
        KernelResponse::Status {
            status: ExecutionStatus::Failed,
            ..
        }
    ));
}

#[test]
fn kernel_helpers_delegate_to_runtime() {
    let mut kernel = Kernel::new().expect("kernel");
    assert!(kernel.cargo_lock().expect("cargo_lock").is_none());
    assert!(kernel.inspect_variables().expect("inspect").is_empty());
    assert!(kernel.diagnose("").expect("diagnose").is_empty());
    assert!(kernel.write_stdin("x").is_err());
    assert!(kernel.stdin_writer().is_some());
    let _ = kernel
        .execute_source(
            "1 + 1",
            None,
            std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        )
        .expect("execute");
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
