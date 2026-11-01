use mars_core::Output;
use mars_runtime::RuntimeBackend;
use mars_runtime_evcxr::{EvcxrSession, runtime_hook};
use std::{
    collections::BTreeMap,
    sync::{Arc, atomic::AtomicBool},
    time::Duration,
};

fn main() {
    if std::env::var_os("MARS_EVCXR_RUN_INTEGRATION").is_none() {
        println!(
            "Evcxr process integration scenarios are opt-in; set MARS_EVCXR_RUN_INTEGRATION=1 to run them"
        );
        return;
    }
    runtime_hook();
    variables_functions_structs_and_ownership();
    compilation_failure_reset_and_output_channels();
    dependencies_panics_and_process_exit();
    timeout_restarts_the_evaluator();
}

fn variables_functions_structs_and_ownership() {
    let mut session = EvcxrSession::new().unwrap_or_else(|error| fail(error));
    success(&mut session, "let values = vec![1, 2, 3];");
    assert_text(success(&mut session, "values.len()"), "3");
    success(&mut session, "fn twice(value: i32) -> i32 { value * 2 }");
    assert_text(success(&mut session, "twice(21)"), "42");
    success(&mut session, "struct Point { x: i32, y: i32 }");
    assert_text(
        success(
            &mut session,
            "let point = Point { x: 2, y: 5 }; point.x + point.y",
        ),
        "7",
    );
    success(&mut session, "let owned = String::from(\"mars\");");
    assert_text(
        success(
            &mut session,
            "let borrowed_len = (&owned).len(); borrowed_len",
        ),
        "4",
    );
    success(&mut session, "let moved = owned;");
    assert_text(success(&mut session, "moved.len()"), "4");
}

fn compilation_failure_reset_and_output_channels() {
    let mut session = EvcxrSession::new().unwrap_or_else(|error| fail(error));
    success(&mut session, "let retained = 42;");
    let failed = run(&mut session, "let impossible: i32 = \"no\";");
    assert_error(&failed, "CompilationError");
    assert!(failed.outputs.iter().any(|output| matches!(output, Output::Error { message, .. } if message.contains("mismatched types"))));
    assert_text(success(&mut session, "retained"), "42");

    session.reset().unwrap_or_else(|error| fail(error));
    assert_error(&run(&mut session, "retained"), "CompilationError");
}

fn dependencies_panics_and_process_exit() {
    let mut session = EvcxrSession::with_dependencies(BTreeMap::from([(
        "regex".to_owned(),
        "1.12.2".to_owned(),
    )]))
    .unwrap_or_else(|error| fail(error));
    assert_text(
        success(
            &mut session,
            "regex::Regex::new(\"^mars$\").map(|re| re.is_match(\"mars\"))",
        ),
        "true",
    );

    let panic = run(&mut session, "panic!(\"notebook panic\")");
    assert!(
        panic
            .outputs
            .iter()
            .any(|output| matches!(output, Output::Error { .. }))
    );
    success(&mut session, "let after_panic = 7;");

    let exited = run(&mut session, "std::process::exit(23)");
    assert!(
        exited
            .outputs
            .iter()
            .any(|output| matches!(output, Output::Error { .. }))
    );
    assert_text(success(&mut session, "40 + 2"), "42");
}

fn timeout_restarts_the_evaluator() {
    let mut session = EvcxrSession::new().unwrap_or_else(|error| fail(error));
    success(&mut session, "let before_timeout = 9;");
    let result = session
        .execute_with_control(
            "loop { std::thread::sleep(std::time::Duration::from_millis(10)); }",
            Some(Duration::from_millis(100)),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap_or_else(|error| fail(error));
    assert!(result.interrupted);
    assert_error(&result, "Interrupted");
    assert_error(&run(&mut session, "before_timeout"), "CompilationError");
}

fn success(session: &mut EvcxrSession, source: &str) -> mars_runtime::ExecutionResult {
    let result = run(session, source);
    assert!(
        !result
            .outputs
            .iter()
            .any(|output| matches!(output, Output::Error { .. })),
        "cell failed ({source}): {:?}",
        result.outputs
    );
    result
}

fn run(session: &mut EvcxrSession, source: &str) -> mars_runtime::ExecutionResult {
    session
        .execute_with_control(
            source,
            Some(Duration::from_secs(20)),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap_or_else(|error| fail(error))
}

fn assert_text(result: mars_runtime::ExecutionResult, expected: &str) {
    assert!(
        result
            .outputs
            .iter()
            .any(|output| matches!(output, Output::Text { text } if text.contains(expected))),
        "expected text {expected:?}, got {:?}",
        result.outputs
    );
}

fn assert_error(result: &mars_runtime::ExecutionResult, expected_name: &str) {
    assert!(
        result
            .outputs
            .iter()
            .any(|output| matches!(output, Output::Error { name, .. } if name == expected_name)),
        "expected {expected_name}, got {:?}",
        result.outputs
    );
}

fn fail(error: impl std::fmt::Display) -> ! {
    panic!("Evcxr integration check failed: {error}");
}
