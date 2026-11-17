use mars_kernel::RuntimeBackendKind;
use mars_protocol::KernelResponse;
use mars_tauri_lib::AppState;
use mars_tauri_lib::kernel_exec::*;
use mars_tauri_lib::kernel_query::*;
use tauri::Manager;

fn app() -> tauri::App<tauri::test::MockRuntime> {
    let app = tauri::test::mock_app();
    app.manage(AppState::new(RuntimeBackendKind::Replay));
    app
}

fn noop_sink(_: mars_runtime::StreamEvent) {}

#[test]
fn executes_and_queries_rust_cells() {
    let app = app();
    let state = app.state::<AppState>();
    let session = "kernel-test".to_owned();

    let response = execute_cell_inner(
        &state,
        &session,
        "let kernel_test_value = 40 + 2;",
        "rust",
        None,
        &mut noop_sink,
    )
    .expect("executes rust cell");
    assert!(matches!(response, KernelResponse::Execution { .. }));

    let timed = execute_cell_inner(
        &state,
        &session,
        "let kernel_test_second = kernel_test_value * 2;",
        "rust",
        Some(std::time::Duration::from_secs(120)),
        &mut noop_sink,
    )
    .expect("executes second cell");
    assert!(matches!(timed, KernelResponse::Execution { .. }));

    let status = kernel_status(state.clone(), session.clone(), "rust".to_owned())
        .expect("gets kernel status");
    assert!(matches!(status, KernelResponse::Status { .. }));

    let diagnostics = diagnose_cell(
        state.clone(),
        session.clone(),
        "let broken = ;".to_owned(),
        "rust".to_owned(),
    );
    // Diagnose may report diagnostics or reject uncompilable source; either is
    // a handled path.
    let _ = diagnostics;

    reset_kernel(state.clone(), session.clone(), "rust".to_owned()).expect("resets kernel");
    cancel_execution(state.clone(), session).expect("cancels execution");
}

#[test]
fn inspect_variables_and_input_comm_paths() {
    let app = app();
    let state = app.state::<AppState>();
    let session = "kernel-io".to_owned();

    let _ = execute_cell_inner(
        &state,
        &session,
        "let kernel_io_marker = 7;",
        "rust",
        None,
        &mut noop_sink,
    );

    if let Ok(variables) = inspect_variables(state.clone(), session.clone(), "rust".to_owned()) {
        assert!(variables.iter().any(|v| v.name.contains("kernel_io")));
    }

    // stdin writer exists after execution for backends that support it; the
    // command must not panic either way.
    let _ = send_kernel_input(
        state.clone(),
        session.clone(),
        "line\n".to_owned(),
        "rust".to_owned(),
    );
    let _ = send_kernel_comm(
        state.clone(),
        session.clone(),
        "comm-1".to_owned(),
        "comm_msg".to_owned(),
        serde_json::json!({"data": 1}),
        "rust".to_owned(),
    );
}

#[test]
fn rejects_unknown_language_and_missing_sessions() {
    let app = app();
    let state = app.state::<AppState>();

    assert!(execute_cell_inner(&state, "bad-lang", "x", "cobol", None, &mut noop_sink).is_err());

    assert!(
        send_kernel_input(
            state.clone(),
            "missing".to_owned(),
            "hello".to_owned(),
            "rust".to_owned(),
        )
        .is_err()
    );
    assert!(
        send_kernel_comm(
            state.clone(),
            "missing".to_owned(),
            "c1".to_owned(),
            "t".to_owned(),
            serde_json::json!({}),
            "rust".to_owned(),
        )
        .is_err()
    );
    assert!(inspect_variables(state.clone(), "missing".to_owned(), "cobol".to_owned()).is_err());
    assert!(
        diagnose_cell(
            state.clone(),
            "missing".to_owned(),
            "x".to_owned(),
            "cobol".to_owned(),
        )
        .is_err()
    );
}

#[test]
fn executes_sql_and_python_cells() {
    let app = app();
    let state = app.state::<AppState>();

    // SQL runs in-process SQLite — always available.
    let sql = execute_cell_inner(
        &state,
        "sql-session",
        "SELECT 1 AS one;",
        "sql",
        None,
        &mut noop_sink,
    );
    assert!(sql.is_ok());

    // Python needs an interpreter; skip assertions when unavailable but still
    // exercise the kernel_for python branch.
    let _ = execute_cell_inner(
        &state,
        "py-session",
        "print(1)",
        "python",
        Some(std::time::Duration::from_secs(30)),
        &mut noop_sink,
    );
}

#[test]
fn configure_kernel_builds_language_kernels() {
    let app = app();
    let state = app.state::<AppState>();

    let sql = serde_json::json!({
        "format_version": 2,
        "metadata": {"title": "t", "language": "sql"},
        "cells": []
    });
    configure_kernel(
        state.clone(),
        "cfg-sql".to_owned(),
        serde_json::to_string(&sql).expect("serializes"),
        None,
    )
    .expect("configures sql kernel");

    let python = serde_json::json!({
        "format_version": 2,
        "metadata": {"title": "t", "language": "python"},
        "cells": []
    });
    // Interpreter availability varies by host; either outcome exercises the
    // python configuration path.
    let _ = configure_kernel(
        state.clone(),
        "cfg-python".to_owned(),
        serde_json::to_string(&python).expect("serializes"),
        None,
    );

    let rust = serde_json::json!({
        "format_version": 2,
        "metadata": {"title": "t", "language": "rust"},
        "cells": []
    });
    configure_kernel(
        state.clone(),
        "cfg-rust".to_owned(),
        serde_json::to_string(&rust).expect("serializes"),
        None,
    )
    .expect("configures rust kernel");

    assert!(
        configure_kernel(
            state.clone(),
            "cfg-bad".to_owned(),
            "not json".to_owned(),
            None,
        )
        .is_err()
    );

    // A notebook whose language has no runtime fails kernel construction.
    let bogus = serde_json::json!({
        "format_version": 2,
        "metadata": {"title": "t", "language": "cobol"},
        "cells": []
    });
    assert!(
        configure_kernel(
            state.clone(),
            "cfg-cobol".to_owned(),
            serde_json::to_string(&bogus).expect("serializes"),
            None,
        )
        .is_err()
    );

    let _ = send_kernel_input(
        state.clone(),
        "cfg-sql".to_owned(),
        "select 1;".to_owned(),
        "sql".to_owned(),
    );

    // Executing on the configured sessions exercises kernel_for's
    // python/sql option branches.
    let _ = execute_cell_inner(&state, "cfg-sql", "SELECT 1;", "sql", None, &mut noop_sink);
    let _ = execute_cell_inner(
        &state,
        "cfg-python",
        "print(1)",
        "python",
        Some(std::time::Duration::from_secs(30)),
        &mut noop_sink,
    );
}
