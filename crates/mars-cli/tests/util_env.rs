//! `open_with_system` spawn-failure path: `open`/`xdg-open` unresolvable when
//! PATH is stripped. Own binary so PATH mutation cannot race other tests.

#![cfg(unix)]

use mars::util::open_with_system;
use std::ffi::OsString;
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

#[test]
fn open_with_system_surfaces_spawn_failure() {
    let _guard = env_lock();
    let _path = PathGuard::stripped();
    assert!(open_with_system("/tmp/definitely-not-opened.mars").is_err());
}
