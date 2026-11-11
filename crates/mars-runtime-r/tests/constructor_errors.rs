//! Constructor error paths that do not need a real Rscript.

use mars_runtime_r::RSession;

#[test]
fn new_surfaces_io_errors() {
    unsafe { std::env::set_var("MARS_RSCRIPT", "/bin/true") };
    let blocker = std::env::temp_dir().join(format!("mars-blocker-{}", std::process::id()));
    std::fs::write(&blocker, b"x").expect("blocker");
    unsafe { std::env::set_var("TMPDIR", &blocker) };
    let result = RSession::new();
    unsafe {
        std::env::remove_var("TMPDIR");
        std::env::remove_var("MARS_RSCRIPT");
    }
    let _ = std::fs::remove_file(&blocker);
    assert!(result.is_err());
}
