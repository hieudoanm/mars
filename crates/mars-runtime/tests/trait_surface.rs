use mars_runtime::output::StreamEvent;
use mars_runtime::{RuntimeBackend, RustSession};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

/// Exercises every `RuntimeBackend` method through the trait object so the
/// delegation impls in `session_execution.rs` are covered (inherent methods
/// shadow them for direct `RustSession` calls).
#[test]
fn trait_object_dispatch_covers_backend_surface() {
    let session = RustSession::new().expect("session");
    let mut backend: Box<dyn RuntimeBackend> = Box::new(session);
    let cancelled = Arc::new(AtomicBool::new(false));

    let result = backend
        .execute_with_control("let trait_value = 40 + 2;", None, Arc::clone(&cancelled))
        .expect("execute");
    assert_eq!(result.execution_count, 1);
    assert!(!result.interrupted);

    let mut events = Vec::new();
    let streamed = backend
        .execute_with_events(
            "println!(\"via trait\");",
            Some(Duration::from_secs(60)),
            Arc::clone(&cancelled),
            &mut |event: StreamEvent| events.push(event),
        )
        .expect("streamed execute");
    assert_eq!(streamed.execution_count, 2);
    assert!(!events.is_empty());

    let variables = backend.inspect_variables().expect("inspect");
    assert!(variables.iter().any(|v| v.name == "trait_value"));

    let diagnostics = backend.diagnose("let fine = 1;").expect("diagnose");
    assert!(diagnostics.is_empty());

    // No active execution: stdin writes fail, EOF signalling is a no-op.
    assert!(backend.write_stdin("answer").is_err());
    let writer = backend.stdin_writer().expect("stdin writer");
    assert!(writer.write("answer").is_err());
    assert!(writer.write_eof().is_ok());

    assert_eq!(backend.execution_count(), 2);
    assert!(backend.cargo_lock().expect("cargo lock").is_none());

    backend.reset().expect("reset");
    assert_eq!(backend.execution_count(), 0);
}

#[test]
fn runtime_failure_produces_error_output() {
    let mut session = RustSession::new().expect("session");
    let result = session
        .execute(r#"panic!("boom");"#)
        .expect("execution result");
    assert!(
        result
            .outputs
            .iter()
            .any(|output| matches!(output, mars_core::Output::Error { .. })),
        "expected an error output, got {:?}",
        result.outputs
    );
    assert!(!result.interrupted);
}

#[test]
fn cancellation_during_execution_interrupts_result() {
    let mut session = RustSession::new().expect("session");
    let cancelled = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&cancelled);
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(300));
        flag.store(true, std::sync::atomic::Ordering::Relaxed);
    });
    let result = session
        .execute_with_control("loop {}", None, cancelled)
        .expect("execution result");
    assert!(result.interrupted, "expected interrupted result");
}

#[test]
fn trailing_expression_compile_failure_falls_back_to_diagnostics() {
    let mut session = RustSession::new().expect("session");
    // `x +` is not a valid expression, so both the discard and display
    // program variants fail to compile; diagnostics should be returned.
    let diagnostics = session.diagnose("let x = 1;\nx +").expect("diagnose");
    assert!(!diagnostics.is_empty());
    let _ = session.execute("let x = 1;\nx +");
}
