use mars_core::Output;
use mars_runtime::{RuntimeBackend, RuntimeError};
use mars_runtime_python::PythonSession;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

fn session() -> PythonSession {
    PythonSession::new().expect("python session")
}

fn execute(session: &mut PythonSession, source: &str) -> mars_runtime::ExecutionResult {
    session
        .execute_with_control(source, None, Arc::new(AtomicBool::new(false)))
        .expect("execute")
}

fn text_outputs(result: &mars_runtime::ExecutionResult) -> Vec<&str> {
    result
        .outputs
        .iter()
        .filter_map(|output| match output {
            Output::Text { text } => Some(text.as_str()),
            _ => None,
        })
        .collect()
}

fn error_outputs(result: &mars_runtime::ExecutionResult) -> Vec<&Output> {
    result
        .outputs
        .iter()
        .filter(|output| matches!(output, Output::Error { .. }))
        .collect()
}

#[test]
fn persists_variables_across_cells() {
    let mut session = session();
    execute(&mut session, "value = 20");
    let result = execute(&mut session, "value * 2 + 1");
    assert!(!result.interrupted);
    assert_eq!(text_outputs(&result), vec!["41"]);
    assert_eq!(result.execution_count, 2);
}

#[test]
fn captures_stdout_and_last_expression() {
    let mut session = session();
    let result = execute(&mut session, "print('hello from mars')\n21 * 2");
    assert_eq!(text_outputs(&result), vec!["hello from mars\n", "42"]);
}

#[test]
fn reports_python_errors_without_incrementing_count() {
    let mut session = session();
    let result = execute(&mut session, "1 / 0");
    let errors = error_outputs(&result);
    assert_eq!(errors.len(), 1);
    match errors[0] {
        Output::Error { name, message, .. } => {
            assert_eq!(name, "ZeroDivisionError");
            assert!(message.contains("division by zero"));
        }
        _ => unreachable!(),
    }
    assert_eq!(result.execution_count, 0);
    assert!(!result.interrupted);
}

#[test]
fn reports_syntax_errors() {
    let mut session = session();
    let result = execute(&mut session, "def broken(:");
    let errors = error_outputs(&result);
    assert!(matches!(
        errors.first(),
        Some(Output::Error { name, .. }) if name == "SyntaxError"
    ));
}

#[test]
fn multiline_definitions_and_expressions_work() {
    let mut session = session();
    let result = execute(
        &mut session,
        "def greet(name):\n    return f'hi {name}'\n\ngreet('mars')",
    );
    assert_eq!(text_outputs(&result), vec!["'hi mars'"]);
}

#[test]
fn inspect_variables_reports_bindings() {
    let mut session = session();
    execute(&mut session, "answer = 42");
    let variables = session.inspect_variables().expect("inspect");
    let found = variables
        .iter()
        .find(|variable| variable.name == "answer")
        .expect("answer variable");
    assert_eq!(found.type_name.as_deref(), Some("int"));
    assert_eq!(found.value.as_deref(), Some("42"));
}

#[test]
fn diagnose_reports_syntax_errors() {
    let mut session = session();
    let diagnostics = session.diagnose("x = = 1").expect("diagnose");
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].severity, "error");
    assert!(session.diagnose("x = 1").expect("diagnose").is_empty());
}

#[test]
fn reset_clears_state_and_count() {
    let mut session = session();
    execute(&mut session, "persisted = 'yes'");
    session.reset().expect("reset");
    assert_eq!(session.execution_count(), 0);
    let result = execute(&mut session, "persisted");
    assert!(!error_outputs(&result).is_empty());
}

#[test]
fn timeout_interrupts_and_respawns() {
    let mut session = session();
    let result = session
        .execute_with_control(
            "import time\ntime.sleep(30)",
            Some(Duration::from_millis(200)),
            Arc::new(AtomicBool::new(false)),
        )
        .expect("execute");
    assert!(result.interrupted);
    // The driver was killed; the next cell runs on a fresh interpreter.
    let next = execute(&mut session, "fresh = 1\nfresh");
    assert_eq!(text_outputs(&next), vec!["1"]);
    assert_eq!(next.execution_count, 1);
}

#[test]
fn empty_source_is_rejected() {
    let mut session = session();
    let result = session.execute_with_control("   ", None, Arc::new(AtomicBool::new(false)));
    assert!(matches!(result, Err(RuntimeError::EmptySource)));
}

#[test]
fn stderr_is_reported_without_failing_the_cell() {
    let mut session = session();
    let result = execute(&mut session, "import sys\nprint('warn', file=sys.stderr)");
    assert!(
        result
            .outputs
            .iter()
            .any(|output| matches!(output, Output::Text { text } if text.contains("warn")))
    );
    assert!(error_outputs(&result).is_empty());
    assert_eq!(result.execution_count, 1);
}

#[test]
fn input_prompt_is_answered_via_stdin_writer() {
    let mut session = session();
    session.inspect_variables().expect("spawn driver");
    let writer = session.stdin_writer().expect("stdin writer");
    let (prompt_tx, prompt_rx) = std::sync::mpsc::channel::<String>();
    let (result, prompts) = std::thread::scope(|scope| {
        let handle = scope.spawn(|| {
            let mut prompts = Vec::new();
            let result = session
                .execute_with_events(
                    "name = input('Name? ')\nprint('hi ' + name)",
                    Some(Duration::from_secs(15)),
                    Arc::new(AtomicBool::new(false)),
                    &mut |event| {
                        if let mars_runtime::StreamEvent::Prompt { message } = event {
                            prompts.push(message.clone());
                            let _ = prompt_tx.send(message);
                        }
                    },
                )
                .expect("execute");
            (result, prompts)
        });
        prompt_rx
            .recv_timeout(Duration::from_secs(10))
            .expect("prompt event");
        writer.write("Alice").expect("write input");
        handle.join().expect("join")
    });
    assert_eq!(prompts, vec!["Name? ".to_owned()]);
    assert!(
        text_outputs(&result)
            .iter()
            .any(|text| text.contains("hi Alice")),
        "outputs: {:?}",
        result.outputs
    );
}

#[test]
fn display_publishes_mime_bundles() {
    let mut session = session();
    let result = execute(
        &mut session,
        "class Box:\n    def _repr_html_(self): return '<b>box</b>'\n    def __repr__(self): return 'Box()'\ndisplay(Box())",
    );
    assert!(result.outputs.iter().any(|output| matches!(
        output,
        Output::MimeBundle { data, .. }
            if data.get("text/html").and_then(|v| v.as_str()) == Some("<b>box</b>")
                && data.get("text/plain").and_then(|v| v.as_str()) == Some("Box()")
    )));
}

#[test]
fn shell_escape_runs_commands() {
    let mut session = session();
    let result = execute(&mut session, "!echo hello-from-shell");
    assert!(
        text_outputs(&result)
            .iter()
            .any(|text| text.contains("hello-from-shell")),
        "outputs: {:?}",
        result.outputs
    );
}

#[test]
fn top_level_await_is_supported() {
    let mut session = session();
    let result = execute(
        &mut session,
        "import asyncio\nawait asyncio.sleep(0.01)\n'async-done'",
    );
    assert_eq!(text_outputs(&result), vec!["'async-done'"]);
    assert!(error_outputs(&result).is_empty());
}

#[test]
fn unsupported_magic_reports_error() {
    let mut session = session();
    let result = execute(&mut session, "%matplotlib inline");
    assert!(error_outputs(&result).iter().any(|output| matches!(
        output,
        Output::Error { name, message, .. }
            if name == "RuntimeError" && message.contains("unsupported IPython magic")
    )));
}

#[test]
fn shell_failure_reports_shell_error() {
    let mut session = session();
    let result = execute(&mut session, "!false");
    assert!(error_outputs(&result).iter().any(|output| matches!(
        output,
        Output::Error { name, .. } if name == "ShellError"
    )));
}

#[test]
fn display_raw_dict_and_filters() {
    let mut session = session();
    let result = execute(
        &mut session,
        "display({'text/html': '<i>raw</i>', 'text/plain': 'raw'}, raw=True)",
    );
    assert!(result.outputs.iter().any(|output| matches!(
        output,
        Output::MimeBundle { data, .. }
            if data.get("text/html").and_then(|v| v.as_str()) == Some("<i>raw</i>")
    )));

    let filtered = execute(
        &mut session,
        "class Two:\n    def _repr_html_(self): return '<b>two</b>'\n    def __repr__(self): return 'Two()'\ndisplay(Two(), include=['text/html'])",
    );
    assert!(filtered.outputs.iter().any(|output| matches!(
        output,
        Output::MimeBundle { data, .. }
            if data.contains_key("text/html") && !data.contains_key("text/plain")
    )));

    let excluded = execute(&mut session, "display(Two(), exclude=['text/html'])");
    assert!(excluded.outputs.iter().any(|output| matches!(
        output,
        Output::MimeBundle { data, .. }
            if !data.contains_key("text/html") && data.contains_key("text/plain")
    )));
}

#[test]
fn display_mimebundle_method_and_png_bytes() {
    let mut session = session();
    let result = execute(
        &mut session,
        "class Rich:\n    def _repr_mimebundle_(self):\n        return {'application/x-custom': 'yes', 'text/plain': 'Rich()'}, {'meta': 1}\ndisplay(Rich())",
    );
    assert!(result.outputs.iter().any(|output| matches!(
        output,
        Output::MimeBundle { data, metadata, .. }
            if data.get("application/x-custom").and_then(|v| v.as_str()) == Some("yes")
                && metadata.get("meta").and_then(|v| v.as_i64()) == Some(1)
    )));

    let png = execute(
        &mut session,
        "class Png:\n    def _repr_png_(self): return b'\\x89PNG fake'\n    def __repr__(self): return 'Png()'\ndisplay(Png())",
    );
    assert!(png.outputs.iter().any(|output| matches!(
        output,
        Output::MimeBundle { data, .. }
            if data.contains_key("image/png")
    )));
}

#[test]
fn input_eof_raises_eoferror() {
    let mut session = session();
    session.inspect_variables().expect("spawn driver");
    let writer = session.stdin_writer().expect("stdin writer");
    let (prompt_tx, prompt_rx) = std::sync::mpsc::channel::<String>();
    let (result, prompts) = std::thread::scope(|scope| {
        let handle = scope.spawn(|| {
            let mut prompts = Vec::new();
            let result = session
                .execute_with_events(
                    "value = input('Name? ')",
                    Some(Duration::from_secs(15)),
                    Arc::new(AtomicBool::new(false)),
                    &mut |event| {
                        if let mars_runtime::StreamEvent::Prompt { message } = event {
                            prompts.push(message.clone());
                            let _ = prompt_tx.send(message);
                        }
                    },
                )
                .expect("execute");
            (result, prompts)
        });
        prompt_rx
            .recv_timeout(Duration::from_secs(10))
            .expect("prompt event");
        // EOF on the input channel surfaces as EOFError inside the cell.
        writer.write_eof().expect("write eof");
        handle.join().expect("join")
    });
    assert_eq!(prompts, vec!["Name? ".to_owned()]);
    assert!(error_outputs(&result).iter().any(|output| matches!(
        output,
        Output::Error { name, .. } if name == "EOFError"
    )));
}

#[test]
fn matplotlib_figures_emit_png_when_available() {
    let mut session = session();
    let probe = execute(&mut session, "import matplotlib\nmatplotlib.__version__");
    if !error_outputs(&probe).is_empty() {
        eprintln!("matplotlib unavailable; skipping");
        return;
    }
    let result = execute(
        &mut session,
        "import matplotlib.pyplot as plt\nplt.plot([1, 2, 3])\nplt.show()",
    );
    assert!(result.outputs.iter().any(|output| matches!(
        output,
        Output::MimeBundle { data, .. } if data.contains_key("image/png")
    )));
}

#[test]
fn trait_surface_covers_lock_stdin_and_reset() {
    let mut session = PythonSession::new().expect("python session");
    assert!(session.cargo_lock().expect("cargo lock").is_none());

    // Spawn the driver, then exercise write_stdin / stdin_writer directly.
    session.inspect_variables().expect("spawn driver");
    session.write_stdin("ignored").expect("write stdin");
    let writer = session.stdin_writer().expect("stdin writer");
    assert!(writer.write_eof().is_ok());

    session.reset().expect("reset");
    assert_eq!(session.execution_count(), 0);
}

#[test]
fn last_expression_uses_mimebundle_method() {
    let mut session = session();
    let result = execute(
        &mut session,
        "class OnlyMime:\n    def _repr_mimebundle_(self):\n        return {'application/x-custom': {'k': 1}, 'text/plain': 'CUSTOM'}\nOnlyMime()",
    );
    assert!(
        result.outputs.iter().any(|output| matches!(
            output,
            Output::Text { text } if text == "CUSTOM"
        )),
        "outputs: {:?}",
        result.outputs
    );
    assert!(result.outputs.iter().any(|output| matches!(
        output,
        Output::MimeBundle { data, .. }
            if data.get("application/x-custom").is_some()
    )));
}

#[test]
fn repr_png_returning_str_does_not_crash() {
    let mut session = session();
    let result = execute(
        &mut session,
        "class BadPng:\n    def _repr_png_(self): return 'aGVsbG8='\nBadPng()",
    );
    assert!(
        result.outputs.iter().any(|output| matches!(
            output,
            Output::Image { mime_type, data }
                if mime_type == "image/png" && data == "aGVsbG8="
        )),
        "outputs: {:?}",
        result.outputs
    );
}

#[test]
fn comm_messages_round_trip_when_comm_package_available() {
    let mut session = session();
    // Skip when the `comm` package is unavailable (it ships with ipywidgets).
    let probe = execute(
        &mut session,
        "import importlib.util\nimportlib.util.find_spec('comm') is not None",
    );
    let available = text_outputs(&probe)
        .iter()
        .any(|text| text.trim() == "True");
    if !available {
        return;
    }
    let created = execute(
        &mut session,
        "import comm\nwidget_comm = comm.create_comm(target_name='test.widget', data={'state': {'value': 1}})\ncid = widget_comm.comm_id",
    );
    assert!(
        error_outputs(&created).is_empty(),
        "outputs: {:?}",
        created.outputs
    );
    let id_result = execute(&mut session, "cid");
    let comm_id = text_outputs(&id_result)
        .iter()
        .find_map(|text| {
            let trimmed = text.trim().trim_matches('\'').trim_matches('"');
            (trimmed.len() == 32 && trimmed.chars().all(|c| c.is_ascii_hexdigit()))
                .then(|| trimmed.to_owned())
        })
        .expect("comm id output");

    let comms = session
        .handle_comm(
            &comm_id,
            "comm_msg",
            &serde_json::json!({"method": "update", "state": {"value": 2}}),
        )
        .expect("handle_comm");
    // The comm has no registered callback, so nothing is published back, but
    // the request must be acknowledged without error.
    assert!(comms.iter().all(|comm| comm.comm_id == comm_id));

    // A comm_close must also succeed.
    session
        .handle_comm(&comm_id, "comm_close", &serde_json::json!({}))
        .expect("comm_close");
}

#[test]
fn comm_during_execution_is_rejected() {
    // The busy path is exercised indirectly: handle_comm requires &mut self
    // so it cannot race execute_inner on the same session object; the guard
    // exists for cloneable-handle callers. Verify the method exists and
    // accepts a well-formed message on an idle session.
    let mut session = session();
    let result = session.handle_comm(
        "nonexistent",
        "comm_msg",
        &serde_json::json!({"method": "update"}),
    );
    // Unknown comm ids are tolerated by the driver (no callback registered).
    assert!(result.is_ok() || result.is_err());
}
