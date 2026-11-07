use mars_core::{CellType, ExecutionStatus, Output};
use mars_document::Cell;
use mars_mcp::exec::{apply_execution, status_for};
use mars_protocol::KernelResponse;

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
