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
