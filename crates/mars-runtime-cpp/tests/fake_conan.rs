//! Tests that put a fake `conan` executable on PATH so the dependency
//! install branches of `execute_with_control` run without a real Conan.
//! Kept in its own binary: PATH mutation must not race with parallel tests.

#![cfg(unix)]

use mars_runtime::RuntimeBackend;
use mars_runtime_cpp::CppSession;
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    ENV_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
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

    fn set_path(&self, dir: &Path) {
        let mut paths = vec![dir.to_path_buf()];
        if let Some(old) = &self.path {
            paths.extend(std::env::split_paths(old));
        }
        unsafe {
            std::env::set_var("PATH", std::env::join_paths(paths).unwrap());
        }
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

fn fake_bin(name: &str, script: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("mars-cpp-fakebin-{}", name));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    fs::write(&path, script).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    dir
}

fn deps_session(offline: bool, locked: bool, lock: Option<String>) -> Option<CppSession> {
    let mut deps = HashMap::new();
    deps.insert("fmt".to_owned(), "11.0.0".to_owned());
    match CppSession::new(deps, lock, offline, locked) {
        Ok(session) => Some(session),
        Err(_) => {
            eprintln!("no C++ compiler available; skipping");
            None
        }
    }
}

fn execute(
    session: &mut CppSession,
) -> Result<mars_runtime::ExecutionResult, mars_runtime::RuntimeError> {
    session.execute_with_control(
        "int x = 1;",
        Some(Duration::from_secs(120)),
        Arc::new(AtomicBool::new(false)),
    )
}

#[test]
fn successful_conan_install_saves_lockfile() {
    let _guard = env_lock();
    let env = EnvGuard::new();
    let bin = fake_bin(
        "conan",
        "#!/bin/sh\necho fake-conan-lock-v1 > conan.lock\nexit 0\n",
    );
    env.set_path(&bin);

    let Some(mut session) = deps_session(false, false, None) else {
        return;
    };
    // The fake conan succeeds, so the lockfile it drops is read back into the
    // session. Compilation then fails on the Conan profile args (no real
    // Conan toolchain), which is fine — the lock path already ran.
    let _ = execute(&mut session);
    let lock = session.cargo_lock().expect("cargo_lock");
    assert!(
        lock.as_deref()
            .is_some_and(|l| l.contains("fake-conan-lock-v1")),
        "expected saved conan lock, got {lock:?}"
    );
}

#[test]
fn failing_conan_install_surfaces_stderr() {
    let _guard = env_lock();
    let env = EnvGuard::new();
    let bin = fake_bin("conan", "#!/bin/sh\necho 'conan exploded' >&2\nexit 3\n");
    env.set_path(&bin);

    let Some(mut session) = deps_session(false, false, None) else {
        return;
    };
    let error = execute(&mut session).expect_err("failing conan must error");
    let msg = error.to_string();
    assert!(msg.contains("Conan install failed"), "{msg}");
    assert!(msg.contains("conan exploded"), "{msg}");
}

#[test]
fn conan_lock_read_fails_when_lock_is_directory() {
    let _guard = env_lock();
    let env = EnvGuard::new();
    // Fake conan leaves a *directory* named conan.lock: exists() passes but
    // read_to_string fails, covering the lock-read error path.
    let bin = fake_bin("conan", "#!/bin/sh\nmkdir conan.lock\nexit 0\n");
    env.set_path(&bin);

    let Some(mut session) = deps_session(false, false, None) else {
        return;
    };
    let error = execute(&mut session).expect_err("directory lock must fail");
    assert!(error.to_string().contains("Failed to read lock"), "{error}");
}

#[test]
fn compile_spawn_fails_with_broken_compiler() {
    let _guard = env_lock();
    let _env = EnvGuard::new();
    // Executable bit is set so which() accepts it, but the interpreter in the
    // shebang does not exist, so spawn() fails inside compile_command.output().
    let bin = fake_bin("cc-bad", "#!/nonexistent-mars-interp-xyz\nexit 0\n");
    unsafe { std::env::set_var("MARS_CPP_COMPILER", bin.join("cc-bad")) };

    let mut session = match CppSession::new(HashMap::new(), None, false, false) {
        Ok(session) => session,
        Err(_) => return,
    };
    let error = session
        .execute_with_control(
            "int x = 1;",
            Some(Duration::from_secs(60)),
            Arc::new(AtomicBool::new(false)),
        )
        .expect_err("broken compiler must fail");
    assert!(error.to_string().contains("Compilation failed"), "{error}");
}

#[test]
fn exec_spawn_fails_when_compiler_produces_no_binary() {
    let _guard = env_lock();
    let _env = EnvGuard::new();
    // Fake compiler exits 0 without writing `main`, so status.success() passes
    // and the exec spawn of the missing binary fails.
    let bin = fake_bin("cc-noop", "#!/bin/sh\nexit 0\n");
    unsafe { std::env::set_var("MARS_CPP_COMPILER", bin.join("cc-noop")) };

    let mut session = match CppSession::new(HashMap::new(), None, false, false) {
        Ok(session) => session,
        Err(_) => return,
    };
    let error = session
        .execute_with_control(
            "int x = 1;",
            Some(Duration::from_secs(60)),
            Arc::new(AtomicBool::new(false)),
        )
        .expect_err("missing binary must fail");
    assert!(error.to_string().contains("Execution failed"), "{error}");
}

#[test]
fn kill_failure_surfaces_when_path_lacks_kill() {
    let _guard = env_lock();
    let _env = EnvGuard::new();
    // PATH holds only `sleep`/`chmod` symlinks — no `kill` — so the timeout's
    // process-tree kill fails to spawn. A fake compiler (absolute path via
    // MARS_CPP_COMPILER) writes a shell-script `main` that sleeps 5s and
    // self-terminates, so nothing real is needed from PATH. posix_spawnp
    // falls through PATH on exec failure, so a broken `kill` shim cannot be
    // used — the binary must be genuinely absent.
    let bin = std::env::temp_dir().join("mars-cpp-fakebin-nokill");
    let _ = fs::remove_dir_all(&bin);
    fs::create_dir_all(&bin).unwrap();
    for tool in ["sleep", "chmod"] {
        std::os::unix::fs::symlink(format!("/bin/{tool}"), bin.join(tool)).unwrap();
    }
    let compiler = fake_bin(
        "cc-sleep",
        "#!/bin/sh\nout=\"\"\nprev=\"\"\nfor a in \"$@\"; do\n  if [ \"$prev\" = \"-o\" ]; then out=\"$a\"; fi\n  prev=\"$a\"\ndone\nprintf '#!/bin/sh\\nsleep 5\\n' > \"$out\"\nchmod +x \"$out\"\nexit 0\n",
    );
    unsafe {
        std::env::set_var("PATH", &bin);
        std::env::set_var("MARS_CPP_COMPILER", compiler.join("cc-sleep"));
    }

    let mut session = match CppSession::new(HashMap::new(), None, false, false) {
        Ok(session) => session,
        Err(_) => return,
    };
    let error = session
        .execute_with_control(
            "int x = 1;",
            Some(Duration::from_millis(300)),
            Arc::new(AtomicBool::new(false)),
        )
        .expect_err("kill failure must surface");
    assert!(error.to_string().contains("kill process tree"), "{error}");
}
