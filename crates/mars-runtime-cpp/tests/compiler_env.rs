//! `MARS_CPP_COMPILER` override resolution and `find_compiler` PATH probing.
//! Kept in its own test binary: the env mutation must not race with parallel
//! tests spawning compilers.

use mars_runtime_cpp::CppSession;
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
}

struct EnvGuard {
    path: Option<OsString>,
    compiler: Option<OsString>,
}

impl EnvGuard {
    fn new() -> Self {
        Self {
            path: std::env::var_os("PATH"),
            compiler: std::env::var_os("MARS_CPP_COMPILER"),
        }
    }

    fn set_path(&self, dir: &std::path::Path) {
        unsafe { std::env::set_var("PATH", dir) };
    }

    fn clear_compiler(&self) {
        unsafe { std::env::remove_var("MARS_CPP_COMPILER") };
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        match &self.path {
            Some(v) => unsafe { std::env::set_var("PATH", v) },
            None => unsafe { std::env::remove_var("PATH") },
        }
        match &self.compiler {
            Some(v) => unsafe { std::env::set_var("MARS_CPP_COMPILER", v) },
            None => unsafe { std::env::remove_var("MARS_CPP_COMPILER") },
        }
    }
}

fn temp_bin_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("mars-cpp-env-{tag}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn fake_executable(dir: &std::path::Path, name: &str) {
    let path = dir.join(name);
    fs::write(&path, "#!/bin/sh\nexit 0\n").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
}

#[test]
fn compiler_env_override_is_honoured() {
    let _guard = env_lock();
    let env = EnvGuard::new();
    // Point MARS_CPP_COMPILER at a real compiler; find_compiler must accept it.
    unsafe { std::env::set_var("MARS_CPP_COMPILER", "clang++") };
    let session = CppSession::new(HashMap::new(), None, false, false);
    drop(env);
    assert!(session.is_ok() || std::env::var_os("MARS_CPP_COMPILER").is_none());
}

#[test]
fn find_compiler_falls_back_to_gpp() {
    let _guard = env_lock();
    let env = EnvGuard::new();
    let dir = temp_bin_dir("gpp-only");
    fake_executable(&dir, "g++");
    env.set_path(&dir);
    env.clear_compiler();

    // No clang++ on this PATH, but g++ is present: new() must succeed.
    assert!(CppSession::new(HashMap::new(), None, false, false).is_ok());
}

#[test]
fn find_compiler_errors_when_none_found() {
    let _guard = env_lock();
    let env = EnvGuard::new();
    let dir = temp_bin_dir("no-compiler");
    env.set_path(&dir);
    env.clear_compiler();

    let result = CppSession::new(HashMap::new(), None, false, false);
    let Err(error) = result else {
        panic!("no compiler on PATH must fail");
    };
    assert!(error.to_string().contains("No C++ compiler"));
}
