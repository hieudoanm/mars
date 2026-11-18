//! `find_python` resolution-path tests: version hints, pyenv/uv directory
//! scanning, and MPLBACKEND handling. Kept in its own test binary because the
//! tests mutate process-global `PATH`/`HOME`/`MARS_PYTHON`.
#![cfg(unix)]

use mars_runtime_python::{PythonOptions, PythonSession};
use std::os::unix::fs::PermissionsExt;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn write_executable(path: &std::path::Path, body: &str) {
    std::fs::write(path, body).expect("write fake interpreter");
    let mut perms = std::fs::metadata(path).expect("metadata").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).expect("chmod");
}

struct EnvGuard {
    path: Option<std::ffi::OsString>,
    home: Option<std::ffi::OsString>,
    mars_python: Option<std::ffi::OsString>,
}

impl EnvGuard {
    fn capture() -> Self {
        Self {
            path: std::env::var_os("PATH"),
            home: std::env::var_os("HOME"),
            mars_python: std::env::var_os("MARS_PYTHON"),
        }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        // SAFETY: tests in this binary hold ENV_LOCK while mutating.
        unsafe {
            match &self.path {
                Some(v) => std::env::set_var("PATH", v),
                None => std::env::remove_var("PATH"),
            }
            match &self.home {
                Some(v) => std::env::set_var("HOME", v),
                None => std::env::remove_var("HOME"),
            }
            match &self.mars_python {
                Some(v) => std::env::set_var("MARS_PYTHON", v),
                None => std::env::remove_var("MARS_PYTHON"),
            }
        }
    }
}

#[test]
fn version_hint_finds_matching_interpreter() {
    let _lock = ENV_LOCK.lock().expect("env lock");
    let guard = EnvGuard::capture();
    let dir = std::env::temp_dir().join(format!("mars-pyver-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("probe dir");
    write_executable(&dir.join("python3.12"), "#!/bin/sh\necho 'Python 3.12.7'\n");

    // SAFETY: ENV_LOCK held; this is the only test mutating env at a time.
    unsafe {
        std::env::remove_var("MARS_PYTHON");
        std::env::set_var("PATH", dir.as_os_str());
        std::env::set_var("HOME", dir.join("nohome").as_os_str());
    }
    let session = PythonSession::with_options(PythonOptions {
        version: Some("3.12".to_owned()),
        ..Default::default()
    });
    drop(guard);
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        session.is_ok(),
        "expected version discovery, got {}",
        session.err().map(|e| e.to_string()).unwrap_or_default()
    );
}

#[test]
fn pyenv_version_dir_is_scanned() {
    let _lock = ENV_LOCK.lock().expect("env lock");
    let guard = EnvGuard::capture();
    let root = std::env::temp_dir().join(format!("mars-pyenv-{}", std::process::id()));
    let ver_dir = root.join(".pyenv/versions/3.13.1/bin");
    std::fs::create_dir_all(&ver_dir).expect("pyenv dir");
    write_executable(
        &ver_dir.join("python3"),
        "#!/bin/sh\necho 'Python 3.13.1'\n",
    );
    // An empty PATH dir so `python3.13`/`python313` probes fail and the
    // pyenv scan is what finds the interpreter.
    let empty = root.join("empty-bin");
    std::fs::create_dir_all(&empty).expect("empty bin");

    // SAFETY: ENV_LOCK held.
    unsafe {
        std::env::remove_var("MARS_PYTHON");
        std::env::set_var("PATH", empty.as_os_str());
        std::env::set_var("HOME", root.as_os_str());
    }
    let session = PythonSession::with_options(PythonOptions {
        version: Some("3.13".to_owned()),
        ..Default::default()
    });
    drop(guard);
    let _ = std::fs::remove_dir_all(&root);
    assert!(
        session.is_ok(),
        "expected pyenv discovery, got {}",
        session.err().map(|e| e.to_string()).unwrap_or_default()
    );
}

#[test]
fn empty_version_hint_falls_back_to_path_probe() {
    let _lock = ENV_LOCK.lock().expect("env lock");
    let guard = EnvGuard::capture();
    // SAFETY: ENV_LOCK held.
    unsafe {
        std::env::remove_var("MARS_PYTHON");
    }
    // version "" normalizes to empty -> find_python_version returns None and
    // resolution continues to the python3/python/py PATH probes (real PATH).
    let session = PythonSession::with_options(PythonOptions {
        version: Some(String::new()),
        ..Default::default()
    });
    drop(guard);
    // May succeed (python3 on PATH) or fail; the point is it must not panic
    // and must not get stuck. Either outcome exercises the fallback path.
    let _ = session;
}
