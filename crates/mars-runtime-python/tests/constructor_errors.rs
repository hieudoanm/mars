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
}
