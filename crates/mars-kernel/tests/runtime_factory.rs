use mars_kernel::{RuntimeBackendKind, create_runtime};
use std::collections::BTreeMap;

fn deps() -> BTreeMap<String, String> {
    BTreeMap::new()
}

#[test]
fn creates_rust_replay_runtime() {
    match create_runtime(
        "rust",
        RuntimeBackendKind::Replay,
        deps(),
        None,
        false,
        false,
    ) {
        Ok(runtime) => assert_eq!(runtime.execution_count(), 0),
        Err(error) => panic!("rust replay runtime: {error}"),
    }
}

#[test]
fn rust_evcxr_backend_selection() {
    let result = create_runtime(
        "rust",
        RuntimeBackendKind::Evcxr,
        deps(),
        Some("lock".to_owned()),
        true,
        true,
    );
    match result {
        Ok(_) => {
            #[cfg(feature = "experimental-evcxr")]
            panic!("evcxr should reject cargo_lock/locked")
        }
        Err(error) => {
            let message = error.to_string();
            #[cfg(not(feature = "experimental-evcxr"))]
            assert!(message.contains("experimental"), "{message}");
            #[cfg(feature = "experimental-evcxr")]
            assert!(message.contains("Evcxr"), "{message}");
        }
    }
}

#[test]
fn creates_available_interpreter_runtimes() {
    for language in ["python", "cpp"] {
        match create_runtime(
            language,
            RuntimeBackendKind::Replay,
            deps(),
            None,
            false,
            false,
        ) {
            Ok(runtime) => assert_eq!(runtime.execution_count(), 0),
            Err(_) => eprintln!("{language} runtime unavailable; skipping"),
        }
    }
}

#[test]
fn unavailable_interpreters_error_or_skip() {
    for language in ["julia", "r"] {
        if let Err(error) = create_runtime(
            language,
            RuntimeBackendKind::Replay,
            deps(),
            None,
            false,
            false,
        ) {
            assert!(!error.to_string().is_empty());
        }
    }
}

#[test]
fn rejects_unknown_language() {
    let error = match create_runtime(
        "cobol",
        RuntimeBackendKind::Replay,
        deps(),
        None,
        false,
        false,
    ) {
        Ok(_) => panic!("unknown language should fail"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("cobol"));
}

#[test]
fn supported_languages_lists_registered_backends() {
    let languages: Vec<&str> = mars_kernel::supported_languages().collect();
    for expected in ["rust", "cpp", "julia", "python", "r"] {
        assert!(languages.contains(&expected), "missing {expected}");
    }
}
