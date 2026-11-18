//! `baseline_notebook`/`git_output` error paths that need `git` missing from
//! PATH. Kept in its own binary: PATH mutation must not race with parallel
//! tests in other binaries is process-local anyway, but within this binary the
//! env guard serializes access.

#![cfg(unix)]

use mars::commands::run_cli;
use mars_core::CellType;
use mars_document::{Cell, Notebook, save_atomic};
use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
}

struct PathGuard(Option<OsString>);

impl PathGuard {
    fn stripped() -> Self {
        let old = std::env::var_os("PATH");
        let empty = std::env::temp_dir().join(format!("mars-no-path-{}", std::process::id()));
        std::fs::create_dir_all(&empty).expect("empty path dir");
        unsafe { std::env::set_var("PATH", &empty) };
        Self(old)
    }
}

impl Drop for PathGuard {
    fn drop(&mut self) {
        match &self.0 {
            Some(v) => unsafe { std::env::set_var("PATH", v) },
            None => unsafe { std::env::remove_var("PATH") },
        }
    }
}

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

#[test]
fn diff_git_ref_without_git_on_path_errors() {
    let _guard = env_lock();
    let _path = PathGuard::stripped();

    let dir = std::env::temp_dir().join(format!("mars-cli-nogit-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path: PathBuf = dir.join("demo.mars");
    let mut notebook = Notebook::new();
    notebook
        .add_cell(Cell::new(CellType::Code, "let x = 1;"))
        .expect("cell");
    save_atomic(&notebook, &path).expect("save");

    // `git rev-parse` cannot spawn: the error must surface, not panic.
    let result = run_cli(args(&["notebook", "diff", &path.to_string_lossy(), "HEAD"]));
    assert!(result.is_err(), "expected git spawn failure");
}
