//! Constructor and pre-spawn error paths that do not need a real Julia.
//! Single test: env mutations must not race with parallel tests.

use mars_runtime::RuntimeBackend;
use mars_runtime_julia::JuliaSession;

#[test]
fn constructor_and_pre_spawn_errors() {
    // Lazy spawn: MARS_JULIA need not point at a real interpreter for `new`.
    unsafe { std::env::set_var("MARS_JULIA", "/bin/true") };

    // Point TMPDIR at a regular file so create_dir_all fails.
    let blocker = std::env::temp_dir().join(format!("mars-blocker-{}", std::process::id()));
    std::fs::write(&blocker, b"x").expect("blocker");
    unsafe { std::env::set_var("TMPDIR", &blocker) };
    let result = JuliaSession::new();
    unsafe { std::env::remove_var("TMPDIR") };
    let _ = std::fs::remove_file(&blocker);
    assert!(result.is_err());

    // Pre-spawn stdin writer reports the missing driver stdin.
    let session = JuliaSession::new().expect("lazy session");
    unsafe { std::env::remove_var("MARS_JULIA") };
    let writer = session.stdin_writer().expect("writer");
    let error = writer.write("answer").expect_err("no driver stdin yet");
    assert!(error.to_string().contains("stdin"), "unexpected: {error}");
}
