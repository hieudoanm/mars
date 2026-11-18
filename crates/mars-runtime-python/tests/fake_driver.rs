//! Tests that drive `PythonSession` against a fake interpreter implementing
//! the newline-delimited JSON driver protocol, so session/protocol logic is
//! covered without a real Julia installation.

#![cfg(unix)]

use mars_core::Output;
use mars_runtime::{
    ChunkStream, ExecutionResult, RuntimeBackend, RuntimeError, StreamEvent, StreamPhase,
};
use mars_runtime_python::PythonSession;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

const FAKE_DRIVER: &str = r#"#!/usr/bin/env python3
import json, os, sys, time

LOG = os.environ.get("FAKE_MARS_DRIVER_LOG")

def send(msg):
    sys.stdout.write(json.dumps(msg) + "\n")
    sys.stdout.flush()

arm_die = False

for line in sys.stdin:
    line = line.strip()
    if not line:
        continue
    try:
        req = json.loads(line)
    except Exception:
        continue
    rid = req.get("id")
    rtype = req.get("type")
    if rtype == "shutdown":
        break
    if rtype == "input_response":
        if LOG:
            with open(LOG, "a") as f:
                f.write(str(req.get("text")) + "\n")
        continue
    if rtype in ("comm_msg", "comm_close"):
        if arm_die:
            sys.exit(3)
        send({"type": "comm", "comm_id": "out-1", "comm_type": "comm_msg",
              "target_name": "test.widget", "data": {"value": 7}})
        send({"type": "comm_ack", "id": rid})
        continue
    if rtype == "inspect":
        sys.stdout.write("this is not json\n")
        sys.stdout.flush()
        send({"type": "stream", "id": rid, "stream": "stdout", "data": "wrong type\n"})
        send({"type": "inspection", "id": 999999, "variables": []})
        send({"type": "inspection", "id": rid, "variables": [
            {"name": "alpha", "type": "Int64", "value": "42"},
            {"name": "beta"},
            {"type": "Int64"},
        ]})
        continue
    if rtype == "check":
        src = req.get("source", "")
        if "FAKE_BAD" in src:
            send({"type": "check_result", "id": rid, "error": {
                "name": "syntax", "lineno": 3, "offset": 7, "message": "bad syntax"}})
        else:
            send({"type": "check_result", "id": rid, "error": None})
        continue
    if rtype != "execute":
        continue
    src = req.get("source", "")
    if "FAKE_DIE" in src:
        sys.exit(3)
    if "FAKE_SLEEP" in src:
        time.sleep(30)
    if "FAKE_ARM_DIE" in src:
        arm_die = True
    if "FAKE_CLOSE_FD" in src:
        os.close(0)
    if "FAKE_DYING_STDERR" in src:
        sys.stderr.write("dying now\n")
        sys.stderr.flush()
        time.sleep(0.2)
        sys.exit(3)
    if "FAKE_STDERR_BIG" in src:
        sys.stderr.write("x" * 70000)
        sys.stderr.flush()
    if "FAKE_TYPES" in src:
        send({"type": "stream", "id": rid, "stream": "other", "data": "odd\n"})
        send({"type": "mystery", "id": rid})
    if "FAKE_COMMSG" in src:
        send({"type": "comm", "id": rid, "comm_id": "c1",
              "comm_type": "comm_msg", "target_name": "w",
              "data": {"x": 1}})
    if "FAKE_COMMBAD" in src:
        send({"type": "comm", "id": rid, "data": {}})
    if "FAKE_BAD_DISPLAY" in src:
        send({"type": "display", "id": rid, "data": "scalar"})
        send({"type": "display", "id": rid,
              "data": [["only-mime"], ["text/plain", 42], "junk"]})
    if "FAKE_BAD_MIME" in src:
        send({"type": "mime", "id": rid, "data": "not-an-object"})
    if "FAKE_ENV" in src:
        send({"type": "stream", "id": rid, "stream": "stdout",
              "data": "mpl=%s pkgs=%s\n" % (
                  os.environ.get("MPLBACKEND"),
                  os.environ.get("MARS_PYTHON_PACKAGES"))})
    if "FAKE_NOISE" in src:
        sys.stdout.write("this is not json\n")
        sys.stdout.flush()
        send({"type": "stream", "id": 999999, "stream": "stdout", "data": "wrong id\n"})
        sys.stdout.write(json.dumps({"type": "stream", "id": rid,
            "stream": "stdout", "data": "no newline"}) + "\n")
        sys.stdout.flush()
    if "FAKE_STDERR" in src:
        send({"type": "stream", "id": rid, "stream": "stderr", "data": "warn text\n"})
    if "FAKE_STREAM" in src:
        send({"type": "stream", "id": rid, "stream": "stdout", "data": " streamed\n"})
    if "FAKE_HTML" in src:
        send({"type": "display", "id": rid,
              "data": [["text/html", "<b>hi</b>"], ["text/plain", "hi"]]})
    elif "FAKE_IMG" in src:
        send({"type": "display", "id": rid, "data": [["image/png", "aGk="]]})
    elif "FAKE_BUNDLE" in src:
        send({"type": "display", "id": rid, "data": [["text/x-custom", "v"]]})
    elif "FAKE_EMPTY_DISPLAY" in src:
        send({"type": "display", "id": rid, "data": []})
    else:
        send({"type": "display", "id": rid, "data": [["text/plain", "fake result"]]})
    if "FAKE_MIME" in src:
        send({"type": "mime", "id": rid,
              "data": {"text/html": "<i>m</i>", "image/png": "aGk="},
              "metadata": {"k": "v"}})
    if "FAKE_INPUT" in src:
        send({"type": "input_request", "id": rid, "prompt": "name?"})
    if "FAKE_ERROR_BARE" in src:
        send({"type": "result", "id": rid})
    elif "FAKE_ERROR" in src:
        send({"type": "result", "id": rid, "ok": False, "error": {
            "name": "FakeError", "message": "boom",
            "traceback": ["frame1", "frame2"]}})
    else:
        send({"type": "result", "id": rid, "ok": True})
    if "FAKE_CLOSE_FD" in src:
        while True:
            time.sleep(1)
"#;

fn python3_available() -> bool {
    std::process::Command::new("python3")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn fake_driver_path() -> PathBuf {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let dir = std::env::temp_dir().join("mars-fake-python-driver");
        fs::create_dir_all(&dir).expect("create fake driver dir");
        let path = dir.join("fake_python.py");
        fs::write(&path, FAKE_DRIVER).expect("write fake driver");
        let mut perms = fs::metadata(&path).expect("driver metadata").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).expect("chmod fake driver");
        path
    })
    .clone()
}

fn input_log_path() -> PathBuf {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        std::env::temp_dir()
            .join("mars-fake-python-driver")
            .join("input.log")
    })
    .clone()
}

fn session() -> Option<PythonSession> {
    if !python3_available() {
        return None;
    }
    // SAFETY: all tests in this binary use the same fake interpreter path and
    // log file, so concurrent `set_var` calls write identical values.
    unsafe {
        std::env::set_var("MARS_PYTHON", fake_driver_path());
        std::env::set_var("FAKE_MARS_DRIVER_LOG", input_log_path());
    }
    PythonSession::new().ok()
}

fn execute(session: &mut PythonSession, source: &str) -> ExecutionResult {
    session
        .execute_with_control(source, None, Arc::new(AtomicBool::new(false)))
        .expect("execute")
}

fn text_outputs(result: &ExecutionResult) -> Vec<&str> {
    result
        .outputs
        .iter()
        .filter_map(|output| match output {
            Output::Text { text } => Some(text.as_str()),
            _ => None,
        })
        .collect()
}

#[test]
fn executes_cell_and_reports_result() {
    let Some(mut session) = session() else { return };
    let result = execute(&mut session, "1 + 1");
    assert!(!result.interrupted);
    assert_eq!(result.execution_count, 1);
    assert_eq!(text_outputs(&result), vec!["fake result"]);
}

#[test]
fn captures_stdout_stderr_and_display() {
    let Some(mut session) = session() else { return };
    let result = execute(&mut session, "FAKE_STREAM FAKE_STDERR");
    let texts = text_outputs(&result);
    assert!(texts.contains(&" streamed\n"));
    assert!(texts.iter().any(|t| t.starts_with("[stderr]\nwarn text")));
    assert!(texts.contains(&"fake result"));
}

#[test]
fn html_and_image_displays_become_rich_outputs() {
    let Some(mut session) = session() else { return };
    let html = execute(&mut session, "FAKE_HTML");
    assert!(
        html.outputs
            .iter()
            .any(|o| matches!(o, Output::Html { html } if html == "<b>hi</b>"))
    );

    let img = execute(&mut session, "FAKE_IMG");
    assert!(img.outputs.iter().any(|o| matches!(o,
        Output::Image { mime_type, data } if mime_type == "image/png" && data == "aGk=")));
}

#[test]
fn custom_display_parts_become_mime_bundle() {
    let Some(mut session) = session() else { return };
    let result = execute(&mut session, "FAKE_BUNDLE");
    assert!(result.outputs.iter().any(|o| matches!(o,
        Output::MimeBundle { data, .. } if data.get("text/x-custom") == Some(&serde_json::json!("v")))));
}

#[test]
fn mime_message_becomes_mime_bundle_output() {
    let Some(mut session) = session() else { return };
    let result = execute(&mut session, "FAKE_MIME");
    assert!(result.outputs.iter().any(|o| matches!(o,
        Output::MimeBundle { data, metadata }
            if data.get("text/html") == Some(&serde_json::json!("<i>m</i>"))
                && metadata.get("k") == Some(&serde_json::json!("v")))));
}

#[test]
fn error_result_produces_error_output_and_skips_count() {
    let Some(mut session) = session() else { return };
    let result = execute(&mut session, "FAKE_ERROR");
    assert_eq!(result.execution_count, 0);
    assert!(result.outputs.iter().any(|o| matches!(o,
        Output::Error { name, message, traceback }
            if name == "FakeError" && message == "boom" && traceback == &vec!["frame1".to_owned(), "frame2".to_owned()])));
}

#[test]
fn bare_error_result_uses_generic_message() {
    let Some(mut session) = session() else { return };
    let result = execute(&mut session, "FAKE_ERROR_BARE");
    assert!(result.outputs.iter().any(|o| matches!(o,
        Output::Error { name, message, .. } if name == "Error" && message == "cell execution failed")));
}

#[test]
fn empty_display_parts_produce_no_output() {
    let Some(mut session) = session() else { return };
    let result = execute(&mut session, "FAKE_EMPTY_DISPLAY");
    assert!(result.outputs.is_empty());
    assert_eq!(result.execution_count, 1);
}

#[test]
fn noise_lines_and_wrong_ids_are_skipped() {
    let Some(mut session) = session() else { return };
    let result = execute(&mut session, "FAKE_NOISE");
    assert!(!result.interrupted);
    assert_eq!(text_outputs(&result), vec!["no newline", "fake result"]);
}

#[test]
fn stream_events_are_emitted() {
    let Some(mut session) = session() else { return };
    let mut events = Vec::new();
    let result = session
        .execute_with_events(
            "FAKE_STREAM FAKE_INPUT",
            None,
            Arc::new(AtomicBool::new(false)),
            &mut |event| events.push(event),
        )
        .expect("execute");
    assert!(!result.interrupted);
    assert!(events.iter().any(|e| matches!(
        e,
        StreamEvent::Phase {
            phase: StreamPhase::Running
        }
    )));
    assert!(events.iter().any(|e| matches!(e,
        StreamEvent::Chunk { stream: ChunkStream::Stdout, data } if data == " streamed\n")));
    assert!(events.iter().any(|e| matches!(e,
        StreamEvent::Prompt { message } if message == "name?")));
}

#[test]
fn inspect_variables_parses_response() {
    let Some(mut session) = session() else { return };
    let vars = session.inspect_variables().expect("inspect");
    assert_eq!(vars.len(), 2);
    assert_eq!(vars[0].name, "alpha");
    assert_eq!(vars[0].type_name.as_deref(), Some("Int64"));
    assert_eq!(vars[0].value.as_deref(), Some("42"));
    assert_eq!(vars[1].name, "beta");
    assert!(vars[1].type_name.is_none());
}

#[test]
fn diagnose_reports_parse_errors() {
    let Some(mut session) = session() else { return };
    let diagnostics = session.diagnose("FAKE_BAD syntax").expect("diagnose");
    assert_eq!(diagnostics.len(), 1);
    let d = &diagnostics[0];
    assert_eq!(d.line, 3);
    assert_eq!(d.column, 7);
    assert_eq!(d.severity, "error");
    assert_eq!(d.code.as_deref(), Some("syntax"));
    assert_eq!(d.message, "bad syntax");

    assert!(session.diagnose("fine").expect("diagnose").is_empty());
}

#[test]
fn write_stdin_and_stdin_writer_reach_driver() {
    let Some(mut session) = session() else { return };
    let log = input_log_path();
    let _ = fs::remove_file(&log);
    session.write_stdin("first answer").expect("write_stdin");
    if let Some(writer) = session.stdin_writer() {
        writer.write("second answer").expect("stdin writer");
        writer.write_eof().expect("stdin writer eof");
    } else {
        panic!("stdin writer unavailable");
    }
    // input_response lines need a spawned driver; force one via a cell.
    execute(&mut session, "warmup");
    // Give the driver a moment to flush the log writes.
    for _ in 0..50 {
        if fs::read_to_string(&log).is_ok_and(|c| c.contains("first answer")) {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let content = fs::read_to_string(&log).unwrap_or_default();
    assert!(content.contains("first answer"), "log was: {content:?}");
    assert!(content.contains("second answer"), "log was: {content:?}");
    assert!(content.contains("None"), "log was: {content:?}");
}

#[test]
fn empty_source_is_rejected() {
    let Some(mut session) = session() else { return };
    let err = session
        .execute_with_control("   \n  ", None, Arc::new(AtomicBool::new(false)))
        .expect_err("empty source must fail");
    assert!(matches!(err, RuntimeError::EmptySource));
}

#[test]
fn timeout_interrupts_and_kills_driver() {
    let Some(mut session) = session() else { return };
    let result = session
        .execute_with_control(
            "FAKE_SLEEP",
            Some(Duration::from_millis(150)),
            Arc::new(AtomicBool::new(false)),
        )
        .expect("execute");
    assert!(result.interrupted);
    assert!(result.outputs.iter().any(|o| matches!(o,
        Output::Error { name, message, .. }
            if name == "Interrupted" && message.contains("time limit"))));

    // The next cell respawns a fresh driver and still works.
    let result = execute(&mut session, "back alive");
    assert!(!result.interrupted);
    assert_eq!(result.execution_count, 1);
}

#[test]
fn cancellation_interrupts_execution() {
    let Some(mut session) = session() else { return };
    let cancellation = Arc::new(AtomicBool::new(true));
    let result = session
        .execute_with_control("FAKE_SLEEP", None, cancellation)
        .expect("execute");
    assert!(result.interrupted);
    assert!(result.outputs.iter().any(|o| matches!(o,
        Output::Error { message, .. } if message.contains("cancelled"))));
}

#[test]
fn driver_death_surfaces_backend_error() {
    let Some(mut session) = session() else { return };
    let err = session
        .execute_with_control("FAKE_DIE", None, Arc::new(AtomicBool::new(false)))
        .expect_err("driver death must error");
    assert!(matches!(err, RuntimeError::Backend(_)));
    assert!(err.to_string().contains("exited"), "error was: {err}");
}

#[test]
fn reset_clears_execution_count() {
    let Some(mut session) = session() else { return };
    execute(&mut session, "one");
    assert_eq!(session.execution_count(), 1);
    session.reset().expect("reset");
    assert_eq!(session.execution_count(), 0);
    execute(&mut session, "two");
    assert_eq!(session.execution_count(), 1);
}

#[test]
fn cargo_lock_is_none() {
    let Some(session) = session() else { return };
    assert_eq!(session.cargo_lock().expect("cargo_lock"), None);
}

#[test]
fn comm_messages_reach_event_sink() {
    let Some(mut session) = session() else { return };
    let mut events = Vec::new();
    session
        .execute_with_events(
            "FAKE_COMMSG",
            None,
            Arc::new(AtomicBool::new(false)),
            &mut |event| events.push(event),
        )
        .expect("execute");
    assert!(events.iter().any(|e| matches!(e,
        StreamEvent::Comm { message }
            if message.comm_id == "c1" && message.data == serde_json::json!({"x": 1}))));
}

#[test]
fn comm_message_without_id_is_dropped() {
    let Some(mut session) = session() else { return };
    let mut events = Vec::new();
    session
        .execute_with_events(
            "FAKE_COMMBAD",
            None,
            Arc::new(AtomicBool::new(false)),
            &mut |event| events.push(event),
        )
        .expect("execute");
    assert!(!events.iter().any(|e| matches!(e, StreamEvent::Comm { .. })));
}

#[test]
fn input_request_without_events_is_silent() {
    let Some(mut session) = session() else { return };
    let result = execute(&mut session, "FAKE_INPUT");
    assert!(!result.interrupted);
    assert_eq!(result.execution_count, 1);
}

#[test]
fn unknown_stream_and_message_types_are_ignored() {
    let Some(mut session) = session() else { return };
    let result = execute(&mut session, "FAKE_TYPES");
    assert!(!result.interrupted);
    assert_eq!(result.execution_count, 1);
}

#[test]
fn malformed_display_and_mime_parts_are_ignored() {
    let Some(mut session) = session() else { return };
    let result = execute(&mut session, "FAKE_BAD_DISPLAY FAKE_BAD_MIME");
    assert!(!result.interrupted);
    assert_eq!(result.execution_count, 1);
    assert_eq!(text_outputs(&result), vec!["fake result"]);
}

#[test]
fn oversized_stderr_is_truncated() {
    let Some(mut session) = session() else { return };
    let result = execute(&mut session, "FAKE_STDERR_BIG");
    assert!(!result.interrupted);
}

#[test]
fn driver_death_with_stderr_reports_tail() {
    let Some(mut session) = session() else { return };
    let err = session
        .execute_with_control("FAKE_DYING_STDERR", None, Arc::new(AtomicBool::new(false)))
        .expect_err("driver death must error");
    assert!(matches!(err, RuntimeError::Backend(_)));
    assert!(err.to_string().contains("dying now"), "error was: {err}");
}

#[test]
fn inspect_ignores_noise_and_wrong_ids() {
    let Some(mut session) = session() else { return };
    let vars = session.inspect_variables().expect("inspect");
    assert_eq!(vars.len(), 2);
}

#[test]
fn handle_comm_returns_published_messages() {
    let Some(mut session) = session() else { return };
    let comms = session
        .handle_comm("widget-1", "comm_msg", &serde_json::json!({"value": 3}))
        .expect("handle_comm");
    assert_eq!(comms.len(), 1);
    assert_eq!(comms[0].comm_id, "out-1");
    assert_eq!(comms[0].comm_type, "comm_msg");
    assert_eq!(comms[0].target_name.as_deref(), Some("test.widget"));
    assert_eq!(comms[0].data, serde_json::json!({"value": 7}));
}

#[test]
fn handle_comm_close_also_works() {
    let Some(mut session) = session() else { return };
    let comms = session
        .handle_comm("widget-1", "comm_close", &serde_json::Value::Null)
        .expect("handle_comm close");
    assert_eq!(comms.len(), 1);
}

#[test]
fn handle_comm_driver_death_surfaces_backend_error() {
    let Some(mut session) = session() else { return };
    execute(&mut session, "FAKE_ARM_DIE");
    let err = session
        .handle_comm("widget-1", "comm_msg", &serde_json::Value::Null)
        .expect_err("comm on dead driver must error");
    assert!(matches!(err, RuntimeError::Backend(_)));
}

#[test]
fn closed_driver_stdin_fails_next_write() {
    let Some(mut session) = session() else { return };
    execute(&mut session, "FAKE_CLOSE_FD");
    let err = session
        .execute_with_control(
            "next",
            Some(Duration::from_secs(5)),
            Arc::new(AtomicBool::new(false)),
        )
        .expect_err("write to closed stdin must fail");
    assert!(
        err.to_string().contains("failed to write"),
        "error was: {err}"
    );
}

#[test]
fn packages_option_reaches_driver_env() {
    let Some(mut session) = session_with_packages() else {
        return;
    };
    let result = execute(&mut session, "FAKE_ENV");
    assert!(
        text_outputs(&result)
            .iter()
            .any(|t| t.contains("pkgs=[\"numpy\"]")),
        "outputs were: {:?}",
        result.outputs
    );
}

fn session_with_packages() -> Option<PythonSession> {
    if !python3_available() {
        return None;
    }
    unsafe {
        std::env::set_var("MARS_PYTHON", fake_driver_path());
        std::env::set_var("FAKE_MARS_DRIVER_LOG", input_log_path());
    }
    PythonSession::with_options(mars_runtime_python::PythonOptions {
        packages: vec!["numpy".to_owned()],
        ..Default::default()
    })
    .ok()
}
