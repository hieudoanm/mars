use mars_kernel::{Kernel, RuntimeBackendKind};
use mars_protocol::{KernelRequest, RequestEnvelope};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

fn envelope(request: KernelRequest) -> RequestEnvelope {
    RequestEnvelope {
        id: "t".to_owned(),
        request,
    }
}

#[test]
fn backend_kind_from_environment() {
    // SAFETY: single-threaded assertion; no other test reads this variable.
    unsafe { std::env::set_var("MARS_EXPERIMENTAL_RUNTIME", "evcxr") };
    assert_eq!(
        RuntimeBackendKind::from_environment(),
        RuntimeBackendKind::Evcxr
    );
    unsafe { std::env::set_var("MARS_EXPERIMENTAL_RUNTIME", "bogus") };
    assert_eq!(
        RuntimeBackendKind::from_environment(),
        RuntimeBackendKind::Replay
    );
    unsafe { std::env::remove_var("MARS_EXPERIMENTAL_RUNTIME") };
    assert_eq!(
        RuntimeBackendKind::from_environment(),
        RuntimeBackendKind::Replay
    );
}

#[test]
fn kernel_constructors_delegate_to_language_factory() {
    assert!(Kernel::new().is_ok());
    assert!(Kernel::with_dependencies(BTreeMap::new()).is_ok());
    assert!(Kernel::with_backend(BTreeMap::new(), RuntimeBackendKind::Replay, false).is_ok());
    assert!(
        Kernel::with_configuration(
            BTreeMap::new(),
            Some("lock".to_owned()),
            true,
            false,
            RuntimeBackendKind::Replay,
        )
        .is_ok()
    );
    assert!(Kernel::for_language("rust", RuntimeBackendKind::Replay).is_ok());
    assert!(
        Kernel::for_language_with_configuration(
            "cobol",
            BTreeMap::new(),
            None,
            false,
            false,
            RuntimeBackendKind::Replay,
        )
        .is_err()
    );
}

#[test]
fn kernel_helpers_delegate_to_runtime() {
    let mut kernel = Kernel::new().expect("kernel");
    assert!(kernel.cargo_lock().expect("lock").is_none());
    let _ = kernel.stdin_writer();
    assert!(kernel.inspect_variables().expect("vars").is_empty());
    let diagnostics = kernel.diagnose("fn main() {}").expect("diagnose");
    assert!(diagnostics.is_empty());
    // write_stdin without an active execution reports an error.
    let result = kernel.handle(envelope(KernelRequest::Input {
        text: "hello".to_owned(),
    }));
    match result.response {
        mars_protocol::KernelResponse::Input | mars_protocol::KernelResponse::Error { .. } => {}
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn kernel_executes_and_reports_status() {
    let mut kernel = Kernel::new().expect("kernel");
    let response = kernel.handle(envelope(KernelRequest::Execute {
        source: "let kernel_test_value = 1;".to_owned(),
        timeout_millis: Some(120_000),
        language: None,
        options: None,
    }));
    match response.response {
        mars_protocol::KernelResponse::Execution {
            execution_count, ..
        } => assert_eq!(execution_count, 1),
        other => panic!("unexpected response: {other:?}"),
    }
    let status = kernel.handle(envelope(KernelRequest::Status));
    match status.response {
        mars_protocol::KernelResponse::Status { status, .. } => {
            assert_eq!(status, mars_core::ExecutionStatus::Succeeded);
        }
        other => panic!("unexpected response: {other:?}"),
    }
    let variables = kernel.handle(envelope(KernelRequest::Variables));
    match variables.response {
        mars_protocol::KernelResponse::Variables { variables } => {
            assert!(variables.iter().any(|v| v.name == "kernel_test_value"));
        }
        other => panic!("unexpected response: {other:?}"),
    }
    kernel.handle(envelope(KernelRequest::Reset));
    let status = kernel.handle(envelope(KernelRequest::Status));
    match status.response {
        mars_protocol::KernelResponse::Status {
            status,
            execution_count,
        } => {
            assert_eq!(status, mars_core::ExecutionStatus::Idle);
            assert_eq!(execution_count, 0);
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn kernel_execute_with_events_streams() {
    let mut kernel = Kernel::new().expect("kernel");
    let mut phases = Vec::new();
    let response = kernel.execute_with_events(
        "let streamed = 2;",
        Some(Duration::from_secs(120)),
        Arc::new(AtomicBool::new(false)),
        &mut |event| {
            if let mars_runtime::StreamEvent::Phase { .. } = event {
                phases.push(());
            }
        },
    );
    assert!(matches!(
        response,
        mars_protocol::KernelResponse::Execution { .. }
    ));
    assert!(!phases.is_empty(), "expected phase events");
}
