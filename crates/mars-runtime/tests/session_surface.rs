use mars_runtime::RustSession;
use std::collections::BTreeMap;

#[test]
fn constructors_create_and_clean_workdirs() {
    let session = RustSession::new().expect("new");
    assert_eq!(session.execution_count(), 0);
    assert!(session.cargo_lock().expect("lock").is_none());
    drop(session);

    let deps = BTreeMap::from([("serde".to_owned(), "1".to_owned())]);
    let session = RustSession::with_dependencies(deps).expect("with deps");
    drop(session);

    let session =
        RustSession::with_configuration(BTreeMap::new(), Some("lock".to_owned()), true, true)
            .expect("with config");
    // cargo_lock() reads Cargo.lock from the session workdir, which only
    // exists after a dependency build; a fresh session has none.
    assert!(session.cargo_lock().expect("lock").is_none());
    drop(session);
}

#[test]
fn invalid_dependencies_are_rejected() {
    let deps = BTreeMap::from([("bad name!".to_owned(), "1".to_owned())]);
    assert!(RustSession::with_dependencies(deps).is_err());
}

#[test]
fn reset_clears_state() {
    let mut session = RustSession::new().expect("new");
    session.reset();
    assert_eq!(session.execution_count(), 0);
}

#[test]
fn execute_reports_compile_errors() {
    let mut session = RustSession::new().expect("new");
    let result = session.execute("let value: = 1;");
    assert!(result.is_ok());
    assert!(
        result
            .expect("result")
            .outputs
            .iter()
            .any(|output| matches!(
                output,
                mars_core::Output::Error { name, .. } if name == "CompilationError"
            ))
    );
    assert_eq!(session.execution_count(), 0);
}
