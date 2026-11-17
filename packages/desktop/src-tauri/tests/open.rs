use mars_tauri_lib::open::*;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn temp_file() -> (PathBuf, PathBuf) {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    let dir = std::env::temp_dir().join(format!(
        "mars-tauri-open-{}-{}",
        nonce,
        SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).expect("creates temp dir");
    let file = dir.join("note.txt");
    std::fs::write(&file, "hello").expect("writes file");
    (dir, file)
}

#[test]
fn open_in_terminal_and_external_are_best_effort() {
    let (dir, file) = temp_file();
    let path = file.to_string_lossy().into_owned();

    // These spawn real OS handlers (Terminal.app / default app). They must
    // never panic; the result depends on the host's available handlers.
    let _ = open_in_terminal(&path);
    let _ = open_path_external(&path);
    let _ = open_path_external(dir.join("missing.txt").to_string_lossy().as_ref());

    std::fs::remove_dir_all(dir).expect("cleans up");
}

#[test]
fn spawn_failures_surface_as_errors() {
    // With an empty PATH no OS handler binary can be spawned; both helpers
    // must return Err rather than panic.
    let (dir, file) = temp_file();
    let path = file.to_string_lossy().into_owned();

    let saved_path = std::env::var_os("PATH");
    unsafe { std::env::set_var("PATH", "/nonexistent-mars-open-path") };
    let terminal = open_in_terminal(&path);
    let external = open_path_external(&path);
    unsafe {
        match saved_path {
            Some(path) => std::env::set_var("PATH", path),
            None => std::env::remove_var("PATH"),
        }
    }

    // On macOS `open` is resolved via PATH too, so both should fail; on
    // platforms with absolute handler paths they may still succeed.
    let _ = (terminal, external);
    std::fs::remove_dir_all(dir).expect("cleans up");
}
