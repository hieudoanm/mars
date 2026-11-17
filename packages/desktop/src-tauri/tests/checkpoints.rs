use mars_tauri_lib::checkpoints::{
    create_checkpoint, list_checkpoints, read_checkpoint, restore_checkpoint,
};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn temp_notebook(contents: &str) -> (PathBuf, PathBuf) {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    let dir = std::env::temp_dir().join(format!(
        "mars-tauri-checkpoints-{}-{}",
        nonce,
        SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).expect("creates temp dir");
    let notebook = dir.join("analysis.mars");
    std::fs::write(&notebook, contents).expect("writes notebook");
    (dir, notebook)
}

#[test]
fn creates_lists_reads_and_restores_checkpoints() {
    let (dir, notebook) = temp_notebook("v1");
    let path = notebook.to_string_lossy().into_owned();

    let first = create_checkpoint(&path).expect("creates first checkpoint");
    assert!(first.ends_with(".mars"));
    assert!(std::path::Path::new(&first).is_file());

    std::fs::write(&notebook, "v2").expect("updates notebook");
    let second = create_checkpoint(&path).expect("creates second checkpoint");
    assert_ne!(first, second);

    let mut checkpoints = list_checkpoints(&path).expect("lists checkpoints");
    checkpoints.sort();
    assert_eq!(checkpoints.len(), 2);

    assert_eq!(
        read_checkpoint(&path, &first).expect("reads checkpoint"),
        "v1"
    );
    assert_eq!(
        read_checkpoint(&path, &second).expect("reads checkpoint"),
        "v2"
    );

    restore_checkpoint(&path, &first).expect("restores checkpoint");
    assert_eq!(
        std::fs::read_to_string(&notebook).expect("reads notebook"),
        "v1"
    );
    std::fs::remove_dir_all(dir).expect("cleans up");
}

#[test]
fn list_checkpoints_returns_empty_without_directory() {
    let (dir, notebook) = temp_notebook("v1");
    let path = notebook.to_string_lossy().into_owned();
    assert!(
        list_checkpoints(&path)
            .expect("lists checkpoints")
            .is_empty()
    );
    std::fs::remove_dir_all(dir).expect("cleans up");
}

#[test]
fn rejects_missing_notebook_and_traversal() {
    let (dir, notebook) = temp_notebook("v1");
    let path = notebook.to_string_lossy().into_owned();

    assert!(create_checkpoint(&dir.join("missing.mars").to_string_lossy()).is_err());
    assert!(read_checkpoint(&path, "/etc/passwd").is_err());
    assert!(restore_checkpoint(&path, "/etc/passwd").is_err());
    std::fs::remove_dir_all(dir).expect("cleans up");
}

#[test]
fn rejects_invalid_paths_and_failed_restore() {
    // "/" has no file name component.
    assert!(list_checkpoints("/").is_err());
    assert!(create_checkpoint("/").is_err());

    let (dir, notebook) = temp_notebook("v1");
    let path = notebook.to_string_lossy().into_owned();
    let checkpoint = create_checkpoint(&path).expect("creates checkpoint");

    // Replace the notebook with a directory so the restore copy fails.
    std::fs::remove_file(&notebook).expect("removes notebook");
    std::fs::create_dir(&notebook).expect("creates directory at notebook path");
    assert!(restore_checkpoint(&path, &checkpoint).is_err());

    std::fs::remove_dir_all(dir).expect("cleans up");
}

#[cfg(unix)]
#[test]
fn io_errors_under_unwritable_checkpoint_dir() {
    use std::os::unix::fs::PermissionsExt;

    let (dir, notebook) = temp_notebook("v1");
    let path = notebook.to_string_lossy().into_owned();
    let checkpoint_dir = dir.join("analysis.mars.checkpoints");
    let checkpoint = create_checkpoint(&path).expect("creates checkpoint");

    // Inaccessible checkpoint dir: listing, reading, and creating all fail.
    std::fs::set_permissions(&checkpoint_dir, std::fs::Permissions::from_mode(0o000))
        .expect("makes checkpoint dir inaccessible");
    assert!(list_checkpoints(&path).is_err());
    assert!(read_checkpoint(&path, &checkpoint).is_err());
    assert!(create_checkpoint(&path).is_err());

    std::fs::set_permissions(&checkpoint_dir, std::fs::Permissions::from_mode(0o700))
        .expect("restores permissions");

    // An unreadable checkpoint file fails at read_to_string.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&checkpoint, std::fs::Permissions::from_mode(0o000))
            .expect("makes checkpoint unreadable");
        assert!(read_checkpoint(&path, &checkpoint).is_err());
        std::fs::set_permissions(&checkpoint, std::fs::Permissions::from_mode(0o600))
            .expect("restores checkpoint permissions");
    }

    std::fs::remove_dir_all(dir).expect("cleans up");
}
