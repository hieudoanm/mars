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

#[test]
fn creates_sql_runtime_with_sources() {
    use mars_kernel::create_runtime_with_config;
    use mars_kernel::runtime::RuntimeConfig;
    use mars_runtime::SqlSource;
    let mut deps = deps();
    deps.insert("ignored".to_owned(), "value".to_owned());
    let config = RuntimeConfig {
        backend: RuntimeBackendKind::Replay,
        dependencies: deps,
        cargo_lock: Some("Cargo.lock".to_owned()),
        offline: true,
        locked: true,
        python_options: Default::default(),
        sql_sources: vec![SqlSource {
            name: "pets".to_owned(),
            path: std::path::PathBuf::from("/tmp/pets.csv"),
        }],
    };
    let runtime = create_runtime_with_config("sql", config).expect("sql runtime");
    assert_eq!(runtime.execution_count(), 0);

    // Unknown language through the config entry point hits the registry miss.
    let result = create_runtime_with_config(
        "cobol",
        RuntimeConfig {
            backend: RuntimeBackendKind::Replay,
            dependencies: BTreeMap::new(),
            cargo_lock: None,
            offline: false,
            locked: false,
            python_options: Default::default(),
            sql_sources: Vec::new(),
        },
    );
    match result {
        Ok(_) => panic!("unknown language should fail"),
        Err(error) => assert!(error.to_string().contains("cobol")),
    }
}

#[test]
fn python_options_parse_from_metadata() {
    use mars_kernel::python_options_from_metadata;
    use serde_json::json;

    // Missing section → defaults.
    let options = python_options_from_metadata(&serde_json::Map::new());
    assert!(options.interpreter.is_none());
    assert!(options.packages.is_empty());

    // Well-formed section.
    let mut extra = serde_json::Map::new();
    extra.insert(
        "python".to_owned(),
        json!({ "interpreter": "python3.12", "version": "3.12", "packages": ["numpy"] }),
    );
    let options = python_options_from_metadata(&extra);
    assert_eq!(options.interpreter.as_deref(), Some("python3.12"));
    assert_eq!(options.version.as_deref(), Some("3.12"));
    assert_eq!(options.packages, vec!["numpy".to_owned()]);

    // Malformed section falls back to defaults.
    let mut extra = serde_json::Map::new();
    extra.insert("python".to_owned(), json!("not-an-object"));
    let options = python_options_from_metadata(&extra);
    assert!(options.interpreter.is_none());
}

#[test]
fn sql_sources_parse_from_metadata() {
    use mars_kernel::sql_sources_from_metadata;
    use serde_json::json;
    let base = std::path::Path::new("/notebooks/demo");

    // No sql section → empty.
    assert!(sql_sources_from_metadata(&serde_json::Map::new(), base).is_empty());

    let mut extra = serde_json::Map::new();
    extra.insert(
        "sql".to_owned(),
        json!({ "tables": {
            "pets": "data/pets.csv",
            "abs": "/tmp/abs.csv",
            "bad": 42,
        }}),
    );
    let sources = sql_sources_from_metadata(&extra, base);
    assert_eq!(sources.len(), 2);
    let pets = sources.iter().find(|s| s.name == "pets").expect("pets");
    assert_eq!(pets.path, base.join("data/pets.csv"));
    let abs = sources.iter().find(|s| s.name == "abs").expect("abs");
    assert_eq!(abs.path, std::path::PathBuf::from("/tmp/abs.csv"));
}
