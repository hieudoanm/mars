use mars_core::Output;
use mars_runtime::{ExecutionResult, RuntimeBackend, RuntimeError};
use mars_runtime_julia::JuliaSession;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

fn session() -> Option<JuliaSession> {
    JuliaSession::new().ok()
}

fn execute(session: &mut JuliaSession, source: &str) -> ExecutionResult {
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

fn error_outputs(result: &ExecutionResult) -> Vec<&Output> {
    result
        .outputs
        .iter()
        .filter(|output| matches!(output, Output::Error { .. }))
        .collect()
}

#[test]
fn persists_variables_across_cells() {
    let Some(mut session) = session() else {
        return;
    };
    execute(&mut session, "value = 20");
    let result = execute(&mut session, "value * 2 + 1");
    assert!(!result.interrupted);
    assert_eq!(text_outputs(&result), vec!["41"]);
    assert_eq!(result.execution_count, 2);
}

#[test]
fn captures_stdout_and_last_expression() {
    let Some(mut session) = session() else {
        return;
    };
    let result = execute(&mut session, "println(\"hello from mars\")\n21 * 2");
    assert_eq!(text_outputs(&result), vec!["hello from mars\n", "42"]);
}

#[test]
fn reports_julia_errors_without_incrementing_count() {
    let Some(mut session) = session() else {
        return;
    };
    // `1 / 0` is `Inf` in Julia; integer division is what raises DivideError.
    let result = execute(&mut session, "1 ÷ 0");
    let errors = error_outputs(&result);
    assert_eq!(errors.len(), 1);
    match errors[0] {
        Output::Error { name, message, .. } => {
            assert!(
                name.contains("DivideError")
                    || message.contains("division")
                    || message.contains("divide")
            );
        }
        _ => unreachable!(),
    }
    assert_eq!(result.execution_count, 0);
    assert!(!result.interrupted);
}

#[test]
fn reports_parse_errors() {
    let Some(mut session) = session() else {
        return;
    };
    let result = execute(&mut session, "x = = 1");
    assert!(!error_outputs(&result).is_empty());
    assert_eq!(result.execution_count, 0);
}

#[test]
fn assignments_do_not_display() {
    let Some(mut session) = session() else {
        return;
    };
    let result = execute(&mut session, "quiet = 41\nquiet");
    assert_eq!(text_outputs(&result), vec!["41"]);
}

#[test]
fn function_definitions_and_calls_work() {
    let Some(mut session) = session() else {
        return;
    };
    let result = execute(
        &mut session,
        "greet(name) = \"hi \" * name\n\ngreet(\"mars\")",
    );
    // String values render as a mime bundle; html is preferred over text/plain.
    assert!(result.outputs.iter().any(|output| match output {
        Output::Html { html } => html.contains("hi mars"),
        Output::Text { text } => text.contains("hi mars"),
        _ => false,
    }));
}

#[test]
fn inspect_variables_reports_bindings() {
    let Some(mut session) = session() else {
        return;
    };
    execute(&mut session, "answer = 42");
    let variables = session.inspect_variables().expect("inspect");
    let found = variables
        .iter()
        .find(|variable| variable.name == "answer")
        .expect("answer variable");
    assert!(found.type_name.as_deref() == Some("Int64"));
    assert_eq!(found.value.as_deref(), Some("42"));
}

#[test]
fn diagnose_reports_parse_errors() {
    let Some(mut session) = session() else {
        return;
    };
    let diagnostics = session.diagnose("x = = 1").expect("diagnose");
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].severity, "error");
    assert!(session.diagnose("x = 1").expect("diagnose").is_empty());
}

#[test]
fn reset_clears_state_and_count() {
    let Some(mut session) = session() else {
        return;
    };
    execute(&mut session, "persisted = \"yes\"");
    session.reset().expect("reset");
    assert_eq!(session.execution_count(), 0);
    let result = execute(&mut session, "persisted");
    assert!(!error_outputs(&result).is_empty());
}

#[test]
fn timeout_interrupts_and_respawns() {
    let Some(mut session) = session() else {
        return;
    };
    let result = session
        .execute_with_control(
            "sleep(30)",
            Some(Duration::from_millis(500)),
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
    let Some(mut session) = session() else {
        return;
    };
    let result = session.execute_with_control("   ", None, Arc::new(AtomicBool::new(false)));
    assert!(matches!(result, Err(RuntimeError::EmptySource)));
}

#[test]
fn stderr_is_reported_without_failing_the_cell() {
    let Some(mut session) = session() else {
        return;
    };
    let result = execute(&mut session, "println(stderr, \"warn\")");
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
fn readline_prompt_is_answered_via_stdin_writer() {
    let Some(mut session) = session() else {
        return;
    };
    session.inspect_variables().expect("spawn driver");
    let writer = session.stdin_writer().expect("stdin writer");
    let (prompt_tx, prompt_rx) = std::sync::mpsc::channel::<String>();
    let (result, prompts) = std::thread::scope(|scope| {
        let handle = scope.spawn(|| {
            let mut prompts = Vec::new();
            let result = session
                .execute_with_events(
                    "name = readline(\"Name? \")\nprintln(\"hi \", name)",
                    Some(Duration::from_secs(30)),
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
            .recv_timeout(Duration::from_secs(20))
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
