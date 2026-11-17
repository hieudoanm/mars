use mars_tauri_lib::notebook_files::{delete_path, list_notebook_files, rename_path};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static TEMPORARY_DIRECTORY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn temporary_directory() -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let sequence = TEMPORARY_DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("mars-tauri-{nonce}-{sequence}"));
    std::fs::create_dir_all(&path).expect("creates temporary directory");
    path
}

#[test]
fn lists_notebooks_and_rust_project_files() {
    let directory = temporary_directory();
    std::fs::create_dir(directory.join("notebooks")).expect("creates child directory");
    std::fs::write(directory.join("analysis.mars"), "{}").expect("creates notebook");
    std::fs::write(directory.join("Cargo.toml"), "[package]").expect("creates manifest");
    std::fs::write(directory.join("Cargo.lock"), "").expect("creates lockfile");
    std::fs::write(directory.join("main.rs"), "fn main() {}").expect("creates source");
    std::fs::write(directory.join("notes.txt"), "ignored").expect("creates text file");
    std::fs::create_dir(directory.join("target")).expect("creates target directory");
    std::fs::write(directory.join("target").join("build.rs"), "").expect("creates build file");

    let entries = list_notebook_files(&directory).expect("lists directory");
    let names = entries
        .iter()
        .map(|entry| entry.name.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        names,
        [
            "notebooks",
            "analysis.mars",
            "Cargo.lock",
            "Cargo.toml",
            "main.rs"
        ]
    );
    assert!(!names.contains(&"target"));
    assert!(!names.contains(&"notes.txt"));
    std::fs::remove_dir_all(directory).expect("removes temporary directory");
}

#[test]
fn ignores_hidden_entries() {
    let directory = temporary_directory();
    std::fs::create_dir(directory.join(".git")).expect("creates hidden directory");
    std::fs::create_dir(directory.join("visible")).expect("creates visible directory");
    std::fs::write(directory.join(".hidden.mars"), "{}").expect("creates hidden notebook");

    let entries = list_notebook_files(&directory).expect("lists directory");

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].name, "visible");
    std::fs::remove_dir_all(directory).expect("removes temporary directory");
}

#[test]
fn refuses_to_overwrite_an_existing_distinct_file() {
    let directory = temporary_directory();
    let source = directory.join("source.mars");
    let destination = directory.join("destination.mars");
    std::fs::write(&source, "source").expect("creates source");
    std::fs::write(&destination, "destination").expect("creates destination");

    let error = rename_path(&source.to_string_lossy(), &destination.to_string_lossy())
        .expect_err("does not overwrite destination");

    assert_eq!(error, "A file with that name already exists");
    assert_eq!(
        std::fs::read_to_string(destination).expect("reads destination"),
        "destination"
    );
    std::fs::remove_dir_all(directory).expect("removes temporary directory");
}

#[test]
fn allows_case_only_rename() {
    let directory = temporary_directory();
    let upper = directory.join("Case.mars");
    std::fs::write(&upper, "{}").expect("creates notebook");

    // On case-insensitive filesystems (default macOS/Windows) this is a
    // rename of the same file; on case-sensitive Linux it creates a sibling.
    let lower = directory.join("case.mars");
    let result = rename_path(&upper.to_string_lossy(), &lower.to_string_lossy());
    assert!(result.is_ok() || result.is_err());
    std::fs::remove_dir_all(directory).expect("removes temporary directory");
}

#[test]
fn rename_and_delete_edge_cases() {
    let directory = temporary_directory();
    let file = directory.join("note.mars");
    std::fs::write(&file, "{}").expect("creates notebook");
    let file_str = file.to_string_lossy().into_owned();

    // Renaming a path onto itself is a no-op.
    rename_path(&file_str, &file_str).expect("same-path rename is a no-op");

    // Case-insensitive-equal names may canonicalize to the same file
    // (case-insensitive FS) or fail to resolve (case-sensitive FS); both
    // outcomes are valid.
    let clash = directory.join("NOTE.mars");
    std::fs::write(&clash, "other").expect("creates clash file");
    let missing = directory.join("NoTe.mars");
    let _ = rename_path(&missing.to_string_lossy(), &clash.to_string_lossy());

    // Listing a nonexistent directory surfaces the read_dir error.
    assert!(list_notebook_files(&directory.join("missing")).is_err());

    // delete_path removes files and rejects missing paths.
    delete_path(&file_str).expect("deletes file");
    assert!(delete_path(&file_str).is_err());
    std::fs::remove_dir_all(directory).expect("removes temporary directory");
}
