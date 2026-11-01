use mars_runtime::ExecutionResult;
use std::process::Child;

pub fn terminate_evaluator(process_handle: &std::sync::Arc<std::sync::Mutex<Child>>) {
    if let Ok(mut child) = process_handle.lock() {
        let _ = child.kill();
    }
}

pub fn interrupted_result(
    duration: std::time::Duration,
    execution_count: u64,
    message: &str,
) -> ExecutionResult {
    ExecutionResult {
        outputs: vec![mars_core::Output::Error {
            name: "Interrupted".to_owned(),
            message: message.to_owned(),
            traceback: Vec::new(),
        }],
        duration,
        execution_count,
        interrupted: true,
    }
}
