use mars_core::Output;
use mars_runtime::{ExecutionResult, RuntimeBackend, RuntimeError};
use mars_runtime_r::RSession;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

fn session() -> Option<RSession> {
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
    execute(&mut session, "value <- 20");
    let result = execute(&mut session, "value * 2 + 1");
    assert!(!result.interrupted);
    assert_eq!(text_outputs(&result), vec!["[1] 41"]);
    assert_eq!(result.execution_count, 2);
}

#[test]
fn captures_stdout_and_last_expression() {
    let Some(mut session) = session() else {
        return;
    };
    let result = execute(&mut session, "cat(\"hello from mars\\n\")\n21 * 2");
    assert_eq!(text_outputs(&result), vec!["hello from mars\n", "[1] 42"]);
}

#[test]
fn reports_r_errors_without_incrementing_count() {
    let Some(mut session) = session() else {
        return;
    };
    let result = execute(&mut session, "stop(\"boom\")");
    let errors = error_outputs(&result);
    assert_eq!(errors.len(), 1);
    match errors[0] {
        Output::Error { name, message, .. } => {
            assert_eq!(name, "simpleError");
            assert!(message.contains("boom"));
        }
        _ => unreachable!(),
    }
    assert_eq!(result.execution_count, 0);
    assert!(!result.interrupted);
}

#[test]
fn reports_syntax_errors() {
    let Some(mut session) = session() else {
        return;
    };
    let result = execute(&mut session, "x <-");
    let errors = error_outputs(&result);
    assert_eq!(errors.len(), 1);
    assert!(!result.interrupted);
}

#[test]
fn multiline_definitions_and_expressions_work() {
    let Some(mut session) = session() else {
        return;
    };
    let result = execute(
        &mut session,
        "greet <- function(name) paste0(\"hi \", name)\n\ngreet(\"mars\")",
    );
    assert!(
        text_outputs(&result)
            .iter()
            .any(|text| text.contains("hi mars"))
    );
}

#[test]
fn inspect_variables_reports_bindings() {
    let Some(mut session) = session() else {
        return;
    };
    execute(&mut session, "answer <- 42");
    let variables = session.inspect_variables().expect("inspect");
    let found = variables
        .iter()
        .find(|variable| variable.name == "answer")
        .expect("answer variable");
    assert_eq!(found.type_name.as_deref(), Some("numeric"));
    assert!(found.value.as_deref().is_some_and(|v| v.contains("42")));
}

#[test]
fn diagnose_reports_syntax_errors() {
    let Some(mut session) = session() else {
        return;
    };
    let diagnostics = session.diagnose("x <- <- 1").expect("diagnose");
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].severity, "error");
    assert!(session.diagnose("x <- 1").expect("diagnose").is_empty());
}

#[test]
fn reset_clears_state_and_count() {
    let Some(mut session) = session() else {
        return;
    };
    execute(&mut session, "persisted <- \"yes\"");
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
            "Sys.sleep(30)",
            Some(Duration::from_millis(200)),
            Arc::new(AtomicBool::new(false)),
        )
        .expect("execute");
    assert!(result.interrupted);
    // The driver was killed; the next cell runs on a fresh session.
    let next = execute(&mut session, "fresh <- 1\nfresh");
    assert_eq!(text_outputs(&next), vec!["[1] 1"]);
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
fn messages_are_reported_as_stderr_output() {
    let Some(mut session) = session() else {
        return;
    };
    let result = execute(&mut session, "message(\"warn\")");
    assert!(
        result
            .outputs
            .iter()
            .any(|output| matches!(output, Output::Error { name, .. } if name == "stderr"))
    );
}

#[test]
fn plots_are_captured_as_images() {
    let Some(mut session) = session() else {
        return;
    };
    let result = execute(&mut session, "plot(1:10)");
    assert!(
        result
            .outputs
            .iter()
            .any(|output| matches!(output, Output::Image { .. }))
    );
}
