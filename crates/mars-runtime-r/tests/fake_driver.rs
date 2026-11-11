//! Tests that drive `RSession` against a fake interpreter implementing the
//! newline-delimited JSON driver protocol, so session/protocol logic is
//! covered without a real R installation.

#![cfg(unix)]

use mars_core::Output;
use mars_runtime::{
    ChunkStream, ExecutionResult, RuntimeBackend, RuntimeError, StreamEvent, StreamPhase,
};
use mars_runtime_r::RSession;
use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

const FAKE_DRIVER: &str = r#"#!/usr/bin/env python3
import json, os, sys, time

def send(msg):
    sys.stdout.write(json.dumps(msg) + "\n")
    sys.stdout.flush()

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
        continue
    if rtype == "inspect":
        send({"type": "inspection", "id": rid, "variables": [
            {"name": "alpha", "type": "numeric", "value": "42"},
            {"name": "beta"},
            {"type": "numeric"},
        ]})
        continue
    if rtype == "check":
        src = req.get("source", "")
        if "FAKE_BAD" in src:
            send({"type": "check_result", "id": rid, "error": {
                "name": "parse", "lineno": 2, "offset": 5, "message": "unexpected symbol"}})
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
        send({"type": "display", "id": rid, "data": [["text/plain", "[1] fake result"]]})
    if "FAKE_ERROR_BARE" in src:
        send({"type": "result", "id": rid})
    elif "FAKE_ERROR" in src:
        send({"type": "result", "id": rid, "ok": False, "error": {
            "name": "FakeError", "message": "boom",
            "traceback": ["frame1"]}})
    else:
        send({"type": "result", "id": rid, "ok": True})
"#;

fn python3_available() -> bool {
    std::process::Command::new("python3")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn fake_driver_path() -> std::path::PathBuf {
    use std::sync::OnceLock;
    static PATH: OnceLock<std::path::PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let dir = std::env::temp_dir().join("mars-fake-r-driver");
        std::fs::create_dir_all(&dir).expect("create fake driver dir");
        let path = dir.join("fake_r.py");
        std::fs::write(&path, FAKE_DRIVER).expect("write fake driver");
        let mut perms = std::fs::metadata(&path)
            .expect("driver metadata")
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&path, perms).expect("chmod fake driver");
        path
    })
    .clone()
}

fn session() -> Option<RSession> {
    if !python3_available() {
        return None;
    }
    // SAFETY: all tests in this binary use the same fake interpreter path, so
    // concurrent `set_var` calls write identical values.
    unsafe {
        std::env::set_var("MARS_RSCRIPT", fake_driver_path());
    }
    RSession::new().ok()
}

fn execute(session: &mut RSession, source: &str) -> ExecutionResult {
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
    assert_eq!(text_outputs(&result), vec!["[1] fake result"]);
}

#[test]
fn captures_stdout_and_display() {
    let Some(mut session) = session() else { return };
    let result = execute(&mut session, "FAKE_STREAM");
    let texts = text_outputs(&result);
    assert!(texts.contains(&" streamed\n"));
    assert!(texts.contains(&"[1] fake result"));
}

#[test]
fn stderr_stream_becomes_error_output() {
    let Some(mut session) = session() else { return };
    let result = execute(&mut session, "FAKE_STDERR");
    assert!(result.outputs.iter().any(|o| matches!(o,
        Output::Error { name, message, .. } if name == "stderr" && message.contains("warn text"))));
    assert!(!result.interrupted);
    assert_eq!(result.execution_count, 1);
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
fn error_result_produces_error_output_and_skips_count() {
    let Some(mut session) = session() else { return };
    let result = execute(&mut session, "FAKE_ERROR");
    assert_eq!(result.execution_count, 0);
    assert!(result.outputs.iter().any(|o| matches!(o,
        Output::Error { name, message, traceback }
            if name == "FakeError" && message == "boom" && traceback == &vec!["frame1".to_owned()])));
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
    assert_eq!(text_outputs(&result), vec!["no newline", "[1] fake result"]);
}

#[test]
fn stream_events_are_emitted() {
    let Some(mut session) = session() else { return };
    let mut events = Vec::new();
    let result = session
        .execute_with_events(
            "FAKE_STREAM",
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
}

#[test]
fn inspect_variables_parses_response() {
    let Some(mut session) = session() else { return };
    let vars = session.inspect_variables().expect("inspect");
    assert_eq!(vars.len(), 2);
    assert_eq!(vars[0].name, "alpha");
    assert_eq!(vars[0].type_name.as_deref(), Some("numeric"));
    assert_eq!(vars[0].value.as_deref(), Some("42"));
    assert_eq!(vars[1].name, "beta");
}

#[test]
fn diagnose_reports_parse_errors() {
    let Some(mut session) = session() else { return };
    let diagnostics = session.diagnose("FAKE_BAD syntax").expect("diagnose");
    assert_eq!(diagnostics.len(), 1);
    let d = &diagnostics[0];
    assert_eq!(d.line, 2);
    assert_eq!(d.column, 5);
    assert_eq!(d.severity, "error");
    assert_eq!(d.code.as_deref(), Some("parse"));
    assert_eq!(d.message, "unexpected symbol");

    assert!(session.diagnose("fine").expect("diagnose").is_empty());
}

#[test]
fn write_stdin_is_unsupported() {
    let Some(mut session) = session() else { return };
    let err = session
        .write_stdin("answer")
        .expect_err("R stdin unsupported");
    assert!(matches!(err, RuntimeError::Backend(_)));
    assert!(err.to_string().contains("not supported"));
    assert!(session.stdin_writer().is_none());
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
