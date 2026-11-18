//! `find_rscript` PATH-probe success path: a `Rscript` executable on PATH is
//! discovered when `MARS_RSCRIPT` is unset. Kept in its own test binary so
//! the process-global `PATH`/`MARS_RSCRIPT` mutations cannot race siblings.
#![cfg(unix)]

use mars_runtime_r::RSession;
use std::os::unix::fs::PermissionsExt;

#[test]
fn finds_rscript_on_path() {
    let dir = std::env::temp_dir().join(format!("mars-r-path-probe-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("probe dir");
    let script = dir.join("Rscript");
    std::fs::write(&script, "#!/bin/sh\necho 'Rscript version 4.3.0'\n").expect("write probe");
    let mut perms = std::fs::metadata(&script).expect("metadata").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&script, perms).expect("chmod");

    let old_path = std::env::var_os("PATH").unwrap_or_default();
    // SAFETY: this is the only test in this binary.
    unsafe {
        std::env::remove_var("MARS_RSCRIPT");
        std::env::set_var(
            "PATH",
            format!("{}:{}", dir.display(), old_path.to_string_lossy()),
        );
    }
    let result = RSession::new();
    // SAFETY: same as above.
    unsafe { std::env::set_var("PATH", old_path) };
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        result.is_ok(),
        "expected PATH discovery, got {}",
        result.err().map(|e| e.to_string()).unwrap_or_default()
    );
}

#[test]
fn missing_rscript_errors() {
    let dir = std::env::temp_dir().join(format!("mars-r-empty-path-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("empty path dir");

    let old_path = std::env::var_os("PATH").unwrap_or_default();
    // SAFETY: this is the only test in this binary.
    unsafe {
        std::env::remove_var("MARS_RSCRIPT");
        std::env::set_var("PATH", dir.as_os_str());
    }
    let result = RSession::new();
    // SAFETY: same as above.
    unsafe { std::env::set_var("PATH", old_path) };
    let _ = std::fs::remove_dir_all(&dir);
    let err = result.err().expect("expected missing-interpreter error");
    assert!(
        err.to_string().contains("no Rscript interpreter found"),
        "unexpected error: {err}"
    );
}
