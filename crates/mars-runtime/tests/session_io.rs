use mars_runtime::{RuntimeBackend, RustSession};
use std::sync::{Arc, atomic::AtomicBool};

#[test]
fn empty_source_is_rejected_before_compiling() {
    let mut session = RustSession::new().expect("session");
    for source in ["", "   \n\t  "] {
        assert!(session.execute(source).is_err());
    }
    assert_eq!(session.execution_count(), 0);
}

#[test]
fn stdin_writes_fail_without_active_execution() {
    let mut session = RustSession::new().expect("session");
    assert!(session.write_stdin("answer").is_err());

    let writer = session.stdin_writer().expect("writer handle");
    assert!(writer.write("answer").is_err());
    // EOF is always safe to signal.
    assert!(writer.write_eof().is_ok());
}

#[test]
fn execute_with_events_streams_phases_and_chunks() {
    let mut session = RustSession::new().expect("session");
    let mut events = Vec::new();
    let result = session
        .execute_with_events(
            "println!(\"chunked\");",
            None,
            Arc::new(AtomicBool::new(false)),
            &mut |event| events.push(event),
        )
        .expect("execution");

    assert_eq!(result.execution_count, 1);
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
    let stdout: String = events
        .iter()
        .filter_map(|event| match event {
            mars_runtime::StreamEvent::Chunk {
                stream: mars_runtime::ChunkStream::Stdout,
                data,
            } => Some(data.as_str()),
            _ => None,
        })
        .collect();
    assert!(stdout.contains("chunked"));
}

#[test]
fn cancelled_flag_before_start_returns_interrupted() {
    let mut session = RustSession::new().expect("session");
    let cancellation = Arc::new(AtomicBool::new(true));
    let result = session
        .execute_with_control("println!(\"no\");", None, cancellation)
        .expect("result");
    assert!(result.interrupted);
}

#[test]
fn trait_reset_clears_session_state() {
    let mut session = RustSession::new().expect("session");
    session.execute("let kept = 1;").expect("cell");
    assert_eq!(session.execution_count(), 1);
    RuntimeBackend::reset(&mut session).expect("reset");
    assert_eq!(session.execution_count(), 0);
    assert!(session.cells.is_empty());
}

#[test]
fn inspect_variables_reports_live_bindings() {
    let mut session = RustSession::new().expect("session");
    // No committed cells: no bindings, no compile.
    assert!(session.inspect_variables().expect("inspect").is_empty());

    session.execute("let answer = 40 + 2;").expect("cell");
    let variables = session.inspect_variables().expect("inspect");
    let answer = variables
        .iter()
        .find(|variable| variable.name == "answer")
        .expect("answer binding");
    assert_eq!(answer.type_name.as_deref(), Some("i32"));
    assert_eq!(answer.value.as_deref(), Some("42"));
}

#[test]
fn diagnose_short_circuits_and_reports_errors() {
    let mut session = RustSession::new().expect("session");
    assert!(session.diagnose("").expect("empty").is_empty());
    assert!(session.diagnose("   ").expect("blank").is_empty());

    let diagnostics = session.diagnose("let broken = ;").expect("diagnose");
    assert!(!diagnostics.is_empty());
    assert!(
        diagnostics
            .iter()
            .all(|d| d.severity == "error" || d.severity == "warning")
    );

    // A valid candidate cell produces no diagnostics.
    session.execute("let ok = 1;").expect("cell");
    assert!(session.diagnose("let fine = 2;").expect("clean").is_empty());
}
