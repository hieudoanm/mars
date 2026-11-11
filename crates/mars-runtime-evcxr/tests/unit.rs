//! Unit tests for the evcxr backend helpers that do not require spinning up
//! an Evcxr evaluator process.

use mars_runtime_evcxr::context::toml_string;
use mars_runtime_evcxr::control::{interrupted_result, terminate_evaluator};
use mars_runtime_evcxr::runtime_hook;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[test]
fn runtime_hook_is_a_noop_outside_the_evaluator() {
    // Returns immediately unless EVCXR_IS_RUNTIME / WRAP_RUSTC are set.
    runtime_hook();
}

#[test]
fn toml_string_escapes_quotes_and_backslashes() {
    assert_eq!(toml_string("serde"), "\"serde\"");
    assert_eq!(toml_string("a\"b"), "\"a\\\"b\"");
    assert_eq!(toml_string("a\\b"), "\"a\\\\b\"");
}

#[test]
fn interrupted_result_builds_an_interrupted_error_output() {
    let result = interrupted_result(Duration::from_millis(5), 7, "execution cancelled");
    assert!(result.interrupted);
    assert_eq!(result.execution_count, 7);
    assert_eq!(result.duration, Duration::from_millis(5));
    assert!(result.outputs.iter().any(|output| matches!(
        output,
        mars_core::Output::Error { name, message, .. }
            if name == "Interrupted" && message == "execution cancelled"
    )));
}

#[test]
fn terminate_evaluator_kills_the_process() {
    let child = Command::new("sleep")
        .arg("30")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn sleep");
    let handle: Arc<Mutex<Child>> = Arc::new(Mutex::new(child));
    terminate_evaluator(&handle);
    let mut guard = handle.lock().expect("lock");
    let status = guard.wait().expect("wait");
    assert!(!status.success());
}
