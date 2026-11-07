use mars_runtime::RuntimeBackend;
use mars_runtime_cpp::CppSession;
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;

fn session() -> Option<CppSession> {
    CppSession::new(HashMap::new(), None, false, false).ok()
}

#[test]
fn executes_cpp_cells_with_persistent_state() {
    let Some(mut session) = session() else {
        eprintln!("no C++ compiler available; skipping");
        return;
    };

    let first = session
        .execute_with_control(
            "int answer = 40 + 2;",
            Some(Duration::from_secs(120)),
            Arc::new(AtomicBool::new(false)),
        )
        .expect("first cell");
    assert_eq!(first.execution_count, 1);
    assert!(!first.interrupted);

    // Replay semantics: earlier cells are recompiled into every run, so the
    // second cell can reference bindings from the first.
    let second = session
        .execute_with_control(
            "std::cout << \"answer=\" << answer << std::endl;",
            Some(Duration::from_secs(120)),
            Arc::new(AtomicBool::new(false)),
        )
        .expect("second cell");
    assert_eq!(second.execution_count, 2);
    assert!(second
        .outputs
        .iter()
        .any(|output| matches!(output, mars_core::Output::Text { .. })));

    assert_eq!(session.execution_count(), 2);
    assert!(session.cargo_lock().expect("lock").is_none());
}

#[test]
fn reports_compilation_errors() {
    let Some(mut session) = session() else {
        return;
    };
    let result = session.execute_with_control(
        "this is not valid c++ !!!",
        Some(Duration::from_secs(120)),
        Arc::new(AtomicBool::new(false)),
    );
    assert!(result.is_err());
}

#[test]
fn reset_clears_state() {
    let Some(mut session) = session() else {
        return;
    };
    session
        .execute_with_control(
            "int x = 1;",
            Some(Duration::from_secs(120)),
            Arc::new(AtomicBool::new(false)),
        )
        .expect("cell");
    assert_eq!(session.execution_count(), 1);
    session.reset().expect("reset");
    assert_eq!(session.execution_count(), 0);
}

#[test]
fn captures_stderr_output() {
    let Some(mut session) = session() else {
        return;
    };
    let result = session
        .execute_with_control(
            "std::cerr << \"warn here\" << std::endl;",
            Some(Duration::from_secs(120)),
            Arc::new(AtomicBool::new(false)),
        )
        .expect("cell");
    assert!(result.outputs.iter().any(|output| matches!(
        output,
        mars_core::Output::Text { text } if text.contains("warn here")
    )));
}

#[test]
fn timeout_kills_long_running_program() {
    let Some(mut session) = session() else {
        return;
    };
    let result = session.execute_with_control(
        "while (true) {}",
        Some(Duration::from_secs(5)),
        Arc::new(AtomicBool::new(false)),
    );
    let error = result.expect_err("infinite loop must time out");
    assert!(error.to_string().contains("timeout"), "{error}");
}

#[test]
fn cancellation_interrupts_execution() {
    let Some(mut session) = session() else {
        return;
    };
    let cancellation = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&cancellation);
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(500));
        flag.store(true, std::sync::atomic::Ordering::Relaxed);
    });
    let result = session.execute_with_control(
        "while (true) {}",
        Some(Duration::from_secs(120)),
        cancellation,
    );
    let error = result.expect_err("cancelled execution must fail");
    assert!(error.to_string().contains("interrupt"), "{error}");
}

#[test]
fn locked_without_lockfile_errors_before_conan() {
    let mut dependencies = HashMap::new();
    dependencies.insert("nlohmann_json".to_owned(), "3.11.2".to_owned());
    let Ok(mut session) = CppSession::new(dependencies, None, false, true) else {
        eprintln!("no C++ compiler available; skipping");
        return;
    };
    let error = session
        .execute_with_control(
            "int x = 1;",
            Some(Duration::from_secs(60)),
            Arc::new(AtomicBool::new(false)),
        )
        .expect_err("locked without conan_lock must fail");
    assert!(error.to_string().contains("conan_lock"));
}
