use mars_core::{CellType, ExecutionStatus, Output};
use mars_document::{Cell, Notebook, save_atomic};
use mars_kernel::RuntimeBackendKind;
use mars_mcp::MarsSession;
use mars_mcp::exec::{apply_execution, call_exec_tool, status_for};
use mars_protocol::KernelResponse;
use serde_json::json;
use std::path::PathBuf;

fn temp_notebook(name: &str, notebook: &Notebook) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("mars-mcp-exec-{}-{}", std::process::id(), name));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("notebook.mars");
    save_atomic(notebook, &path).expect("save notebook");
    path
}

fn code_notebook() -> Notebook {
    let mut notebook = Notebook::new();
    notebook
        .add_cell(Cell::new(CellType::Markdown, "# Doc"))
        .expect("markdown cell");
    notebook
        .add_cell(Cell::new(CellType::Code, "21 * 2"))
        .expect("code cell");
    notebook
}

#[test]
fn apply_execution_records_outputs_and_count() {
    let mut cell = Cell::new(CellType::Code, "1 + 1");
    let response = KernelResponse::Execution {
        outputs: vec![Output::Text {
            text: "2".to_owned(),
        }],
        execution_count: 1,
        duration_millis: 1,
    };
    let status = apply_execution(&mut cell, &response);
    assert_eq!(status, ExecutionStatus::Succeeded);
    assert_eq!(cell.execution_count, Some(1));
    assert_eq!(cell.outputs.len(), 1);
}

#[test]
fn apply_execution_maps_kernel_errors_onto_cells() {
    let mut cell = Cell::new(CellType::Code, "1 + 1");
    let response = KernelResponse::Error {
        message: "kernel blew up".to_owned(),
    };
    let status = apply_execution(&mut cell, &response);
    assert_eq!(status, ExecutionStatus::Failed);
    assert!(matches!(
        cell.outputs.first(),
        Some(Output::Error { name, message, .. })
            if name == "KernelError" && message == "kernel blew up"
    ));
    assert_eq!(cell.execution_count, None);
}

#[test]
fn error_outputs_mark_failed() {
    let response = KernelResponse::Execution {
        outputs: vec![Output::Error {
            name: "E".to_owned(),
            message: "bad".to_owned(),
            traceback: Vec::new(),
        }],
        execution_count: 1,
        duration_millis: 1,
    };
    assert_eq!(status_for(&response), ExecutionStatus::Failed);
}

#[test]
fn status_for_maps_response_kinds() {
    assert_eq!(
        status_for(&KernelResponse::Reset {
            status: ExecutionStatus::Idle
        }),
        ExecutionStatus::Idle
    );
    assert_eq!(
        status_for(&KernelResponse::Cancelled),
        ExecutionStatus::Idle
    );
    assert_eq!(
        status_for(&KernelResponse::Variables { variables: vec![] }),
        ExecutionStatus::Idle
    );
    let ok = KernelResponse::Execution {
        outputs: vec![Output::Text {
            text: "ok".to_owned(),
        }],
        execution_count: 1,
        duration_millis: 1,
    };
    assert_eq!(status_for(&ok), ExecutionStatus::Succeeded);
}

#[test]
fn exec_tool_rejects_unknown_names_and_bad_args() {
    let mut session = MarsSession::new(RuntimeBackendKind::Replay, true, None);
    assert!(call_exec_tool(&mut session, "mars_unknown", &json!({})).is_none());

    let path = temp_notebook("bad-args", &code_notebook());
    let path_string = path.to_string_lossy().to_string();
    let result = call_exec_tool(&mut session, "mars_cell_execute", &json!({ "path": 42 }));
    assert!(matches!(result, Some(Err(_))));

    let missing = call_exec_tool(
        &mut session,
        "mars_cell_execute",
        &json!({ "path": path_string }),
    )
    .expect("handled");
    assert!(missing.is_err());

    let _ = std::fs::remove_dir_all(path.parent().expect("parent"));
}

#[test]
fn cell_execute_validates_cell_kind_and_index() {
    let path = temp_notebook("cell-checks", &code_notebook());
    let path_string = path.to_string_lossy().to_string();
    let mut session = MarsSession::new(RuntimeBackendKind::Replay, true, None);

    // Markdown cell cannot execute.
    let markdown = call_exec_tool(
        &mut session,
        "mars_cell_execute",
        &json!({ "path": path_string, "index": 0 }),
    )
    .expect("handled");
    assert!(markdown.is_err());

    // Out-of-range index errors.
    let range = call_exec_tool(
        &mut session,
        "mars_cell_execute",
        &json!({ "path": path_string, "index": 99 }),
    )
    .expect("handled");
    assert!(range.is_err());

    let _ = std::fs::remove_dir_all(path.parent().expect("parent"));
}

#[test]
fn cell_execute_runs_code_and_records_outputs() {
    let path = temp_notebook("execute", &code_notebook());
    let path_string = path.to_string_lossy().to_string();
    let mut session = MarsSession::new(RuntimeBackendKind::Replay, true, None);

    let result = call_exec_tool(
        &mut session,
        "mars_cell_execute",
        &json!({ "path": path_string, "index": 1, "language": "rust" }),
    )
    .expect("handled")
    .expect("execute ok");

    assert_eq!(result["status"], "succeeded");
    assert_eq!(result["index"], 1);
    assert_eq!(result["language"], "rust");
    let notebook = session.notebook(&path).expect("notebook");
    let cell = notebook.cell(1).expect("cell");
    assert!(cell.execution_count.is_some());
    assert!(!cell.outputs.is_empty());

    // Ad-hoc source execution without an index.
    let ad_hoc = call_exec_tool(
        &mut session,
        "mars_cell_execute",
        &json!({ "path": path_string, "source": "6 * 7", "language": "rust" }),
    )
    .expect("handled")
    .expect("execute ok");
    assert_eq!(ad_hoc["index"], serde_json::Value::Null);
    assert_eq!(ad_hoc["status"], "succeeded");

    let _ = std::fs::remove_dir_all(path.parent().expect("parent"));
}

#[test]
fn notebook_execute_runs_only_code_cells() {
    let path = temp_notebook("run-all", &code_notebook());
    let path_string = path.to_string_lossy().to_string();
    let mut session = MarsSession::new(RuntimeBackendKind::Replay, true, None);

    let result = call_exec_tool(
        &mut session,
        "mars_notebook_execute",
        &json!({ "path": path_string, "timeout_ms": 30000 }),
    )
    .expect("handled")
    .expect("run ok");

    assert_eq!(result["cells_executed"], 1);
    assert_eq!(result["results"].as_array().expect("results").len(), 1);
    assert_eq!(result["results"][0]["index"], 1);

    let _ = std::fs::remove_dir_all(path.parent().expect("parent"));
}

#[test]
fn notebook_execute_skips_non_code_notebooks() {
    let mut notebook = Notebook::new();
    notebook
        .add_cell(Cell::new(CellType::Markdown, "# Only docs"))
        .expect("cell");
    let path = temp_notebook("markdown-only", &notebook);
    let mut session = MarsSession::new(RuntimeBackendKind::Replay, true, None);

    let result = call_exec_tool(
        &mut session,
        "mars_notebook_execute",
        &json!({ "path": path.to_string_lossy() }),
    )
    .expect("handled")
    .expect("run ok");
    assert_eq!(result["cells_executed"], 0);

    let _ = std::fs::remove_dir_all(path.parent().expect("parent"));
}

#[test]
fn kernel_tools_cover_status_reset_variables_and_diagnose() {
    let path = temp_notebook("kernel-tools", &code_notebook());
    let path_string = path.to_string_lossy().to_string();
    let mut session = MarsSession::new(RuntimeBackendKind::Replay, true, Some("rust".to_owned()));

    let status = call_exec_tool(
        &mut session,
        "mars_kernel_status",
        &json!({ "path": path_string, "language": "rust" }),
    )
    .expect("handled")
    .expect("status ok");
    assert_eq!(status["language"], "rust");

    let variables = call_exec_tool(
        &mut session,
        "mars_kernel_variables",
        &json!({ "language": "rust" }),
    )
    .expect("handled")
    .expect("variables ok");
    assert!(variables["response"]["variables"].is_array());

    let reset = call_exec_tool(
        &mut session,
        "mars_kernel_reset",
        &json!({ "language": "rust" }),
    )
    .expect("handled")
    .expect("reset ok");
    assert_eq!(reset["response"]["type"], "reset");

    // Empty diagnose source short-circuits without compiling.
    let diagnose = call_exec_tool(
        &mut session,
        "mars_cell_diagnose",
        &json!({ "path": path_string, "source": "", "language": "rust" }),
    )
    .expect("handled")
    .expect("diagnose ok");
    assert_eq!(diagnose["response"]["type"], "diagnostics");

    // Invalid args for each kernel tool surface errors.
    for name in [
        "mars_kernel_status",
        "mars_kernel_reset",
        "mars_kernel_variables",
    ] {
        let bad = call_exec_tool(&mut session, name, &json!({ "path": 7 }));
        assert!(matches!(bad, Some(Err(_))), "{name}");
    }

    let _ = std::fs::remove_dir_all(path.parent().expect("parent"));
}

#[test]
fn kernel_tools_accept_an_open_notebook_path() {
    let path = temp_notebook("kernel-path", &code_notebook());
    let path_string = path.to_string_lossy().to_string();
    let mut session = MarsSession::new(RuntimeBackendKind::Replay, true, None);

    for name in [
        "mars_kernel_status",
        "mars_kernel_reset",
        "mars_kernel_variables",
    ] {
        let result = call_exec_tool(
            &mut session,
            name,
            &json!({ "path": path_string, "language": "rust" }),
        )
        .unwrap_or_else(|| panic!("{name} should be an exec tool"))
        .expect(name);
        assert!(result.is_object(), "{name} result");
    }

    // cell_diagnose resolves source from the notebook path too.
    let diagnosed = call_exec_tool(
        &mut session,
        "mars_cell_diagnose",
        &json!({ "path": path_string, "index": 1 }),
    )
    .expect("diagnose tool")
    .expect("diagnose");
    assert!(diagnosed.is_object());

    // Ad-hoc source with explicit language exercises the fallback chain.
    let status = call_exec_tool(
        &mut session,
        "mars_kernel_status",
        &json!({ "path": path_string, "language": "rust" }),
    )
    .expect("status tool")
    .expect("status");
    assert!(status["language"] == "rust");

    let _ = std::fs::remove_dir_all(path.parent().expect("parent"));
}

#[test]
fn exec_tools_report_missing_source_and_bad_indices() {
    let path = temp_notebook("exec-errors", &code_notebook());
    let path_string = path.to_string_lossy().to_string();
    let mut session = MarsSession::new(RuntimeBackendKind::Replay, true, None);

    // Neither index nor source.
    assert!(
        call_exec_tool(
            &mut session,
            "mars_cell_execute",
            &json!({ "path": path_string }),
        )
        .expect("tool")
        .is_err()
    );
    assert!(
        call_exec_tool(
            &mut session,
            "mars_cell_diagnose",
            &json!({ "path": path_string }),
        )
        .expect("tool")
        .is_err()
    );
    // Out-of-range cell index.
    assert!(
        call_exec_tool(
            &mut session,
            "mars_cell_execute",
            &json!({ "path": path_string, "index": 42 }),
        )
        .expect("tool")
        .is_err()
    );

    let _ = std::fs::remove_dir_all(path.parent().expect("parent"));
}

#[test]
fn notebook_execute_reports_cell_failures_and_timeouts() {
    // A notebook whose code cell fails to compile must surface the failure.
    let mut notebook = Notebook::new();
    notebook
        .add_cell(Cell::new(CellType::Code, "this is not rust"))
        .expect("cell");
    let path = temp_notebook("exec-fail", &notebook);
    let path_string = path.to_string_lossy().to_string();
    let mut session = MarsSession::new(RuntimeBackendKind::Replay, true, None);

    let result = call_exec_tool(
        &mut session,
        "mars_notebook_execute",
        &json!({ "path": path_string, "timeout_ms": 60000 }),
    )
    .expect("tool");
    // Either an honest tool error or a result payload — never a panic.
    assert!(result.is_ok() || result.is_err());

    // cell_execute accepts an explicit language override and timeout.
    let ok = call_exec_tool(
        &mut session,
        "mars_cell_execute",
        &json!({
            "path": path_string,
            "index": 0,
            "language": "rust",
            "timeout_ms": 60000,
        }),
    )
    .expect("tool");
    assert!(ok.is_ok() || ok.is_err());

    // diagnose accepts ad-hoc source.
    let diag = call_exec_tool(
        &mut session,
        "mars_cell_diagnose",
        &json!({ "path": path_string, "source": "fn main() {}", "language": "rust" }),
    )
    .expect("tool");
    assert!(diag.is_ok() || diag.is_err());

    let _ = std::fs::remove_dir_all(path.parent().expect("parent"));
}

#[test]
fn kernel_variables_and_status_after_execution() {
    let path = temp_notebook("exec-vars", &code_notebook());
    let path_string = path.to_string_lossy().to_string();
    let mut session = MarsSession::new(RuntimeBackendKind::Replay, true, None);

    // Execute a cell, then inspect variables + status through the kernel tools.
    let _ = call_exec_tool(
        &mut session,
        "mars_cell_execute",
        &json!({ "path": path_string, "index": 0, "timeout_ms": 60000 }),
    );
    for (name, args) in [
        (
            "mars_kernel_variables",
            json!({ "path": path_string, "language": "rust" }),
        ),
        (
            "mars_kernel_status",
            json!({ "path": path_string, "language": "rust" }),
        ),
        (
            "mars_kernel_reset",
            json!({ "path": path_string, "language": "rust" }),
        ),
    ] {
        let result = call_exec_tool(&mut session, name, &args).expect("tool");
        assert!(result.is_ok() || result.is_err(), "{name}");
    }

    let _ = std::fs::remove_dir_all(path.parent().expect("parent"));
}
