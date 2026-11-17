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
