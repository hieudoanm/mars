//! Constructor and pre-spawn error paths that do not need a real Python.
//! Single test: env mutations must not race with parallel tests.

use mars_runtime::RuntimeBackend;
use mars_runtime_python::PythonSession;

#[test]
fn constructor_and_pre_spawn_errors() {
    unsafe { std::env::set_var("MARS_PYTHON", "/bin/true") };

    let blocker = std::env::temp_dir().join(format!("mars-blocker-{}", std::process::id()));
    std::fs::write(&blocker, b"x").expect("blocker");
    unsafe { std::env::set_var("TMPDIR", &blocker) };
    let result = PythonSession::new();
    unsafe { std::env::remove_var("TMPDIR") };
    let _ = std::fs::remove_file(&blocker);
    assert!(result.is_err());

    let session = PythonSession::new().expect("lazy session");
    unsafe { std::env::remove_var("MARS_PYTHON") };
    let writer = session.stdin_writer().expect("writer");
    let error = writer.write("answer").expect_err("no driver stdin yet");
    assert!(error.to_string().contains("stdin"), "unexpected: {error}");

    // Interpreter resolution branches. MARS_PYTHON is unset at this point, so
    // explicit interpreter, version discovery, and PATH probing all run.
    use mars_runtime_python::PythonOptions;
    let options = |interpreter: Option<&str>, version: Option<&str>| PythonOptions {
        interpreter: interpreter.map(str::to_owned),
        version: version.map(str::to_owned),
        packages: Vec::new(),
    };
    // A bogus explicit interpreter fails the probe and is an error even
    // though PATH probing would succeed.
    assert!(PythonSession::with_options(options(Some("/nonexistent/mars-py"), None)).is_err());
    let python3 = std::process::Command::new("python3")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|status| status.success());
    if python3 {
        assert!(PythonSession::with_options(options(Some("python3"), None)).is_ok());
        // Version hints probe `python<version>` executables, then scan
        // pyenv/uv directories before falling back to PATH probing.
        assert!(PythonSession::with_options(options(None, Some("3"))).is_ok());
        assert!(PythonSession::with_options(options(None, Some("0.0.0-no-such-version"))).is_ok());
        // Package lists are accepted at construction; the driver installs
        // them non-fatally at spawn time.
        let mut with_packages = options(None, None);
        with_packages.packages = vec!["definitely-not-installed-pkg".to_owned()];
        assert!(PythonSession::with_options(with_packages).is_ok());
    }
}
