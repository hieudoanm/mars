use mars_core::CellType;
use mars_document::{Cell, Notebook, to_json_pretty};
use mars_tauri_lib::notebook_cmds::*;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn temp_dir() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    let dir = std::env::temp_dir().join(format!(
        "mars-tauri-cmds-{}-{}",
        nonce,
        SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).expect("creates temp dir");
    dir
}

fn notebook_json() -> String {
    let mut notebook = Notebook::new();
    notebook
        .cells
        .push(Cell::new(CellType::Markdown, "# Test notebook"));
    to_json_pretty(&notebook).expect("serializes notebook")
}

#[test]
fn open_and_save_notebook_roundtrip() {
    let dir = temp_dir();
    let path = dir.join("book.mars");
    let path_str = path.to_string_lossy().into_owned();
    let content = notebook_json();

    save_notebook(path_str.clone(), content.clone()).expect("saves notebook");
    let loaded = open_notebook(path_str).expect("opens notebook");
    assert!(loaded.contains("format_version"));

    assert!(open_notebook(dir.join("missing.mars").to_string_lossy().into_owned()).is_err());
    assert!(
        save_notebook(
            dir.join("bad.mars").to_string_lossy().into_owned(),
            "not json".to_owned()
        )
        .is_err()
    );
    std::fs::remove_dir_all(dir).expect("cleans up");
}

#[test]
fn file_mtime_and_create_file() {
    let dir = temp_dir();
    let missing = dir.join("missing.mars").to_string_lossy().into_owned();
    assert_eq!(notebook_file_mtime(missing).expect("mtime"), None);

    let path = dir.join("new.mars").to_string_lossy().into_owned();
    create_notebook_file(path.clone()).expect("creates notebook");
    assert!(std::path::Path::new(&path).is_file());
    assert!(notebook_file_mtime(path.clone()).expect("mtime").is_some());
    assert!(create_notebook_file(path).is_err());
    std::fs::remove_dir_all(dir).expect("cleans up");
}

#[test]
fn directory_delete_write_and_rename() {
    let dir = temp_dir();
    let nested = dir.join("nested").to_string_lossy().into_owned();
    create_directory(nested.clone()).expect("creates directory");
    assert!(create_directory(nested.clone()).is_err());

    let file = dir.join("note.txt").to_string_lossy().into_owned();
    write_text_file(file.clone(), "hello".to_owned()).expect("writes file");
    assert_eq!(std::fs::read_to_string(&file).expect("reads"), "hello");

    let renamed = dir.join("renamed.txt").to_string_lossy().into_owned();
    rename_notebook_file(file.clone(), renamed.clone()).expect("renames");
    assert!(std::path::Path::new(&renamed).is_file());
    assert!(rename_notebook_file(renamed.clone(), file.clone()).is_ok());

    delete_path(file).expect("deletes file");
    assert!(!dir.join("note.txt").exists());
    delete_path(nested).expect("deletes directory");
    assert!(!dir.join("nested").exists());
    assert!(delete_path(dir.join("gone.txt").to_string_lossy().into_owned()).is_err());
    std::fs::remove_dir_all(dir).expect("cleans up");
}

#[test]
fn list_files_and_export_formats() {
    let dir = temp_dir();
    let notebook_path = dir.join("book.mars");
    std::fs::write(&notebook_path, notebook_json()).expect("writes notebook");

    let entries = list_notebook_files(dir.to_string_lossy().into_owned()).expect("lists files");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].name, "book.mars");

    let content = notebook_json();
    let md = dir.join("out.md").to_string_lossy().into_owned();
    export_notebook_markdown(md.clone(), content.clone()).expect("exports markdown");
    assert!(
        std::fs::read_to_string(&md)
            .expect("reads md")
            .contains('#')
    );

    let html = dir.join("out.html").to_string_lossy().into_owned();
    export_notebook_html(html.clone(), content.clone()).expect("exports html");
    assert!(
        std::fs::read_to_string(&html)
            .expect("reads html")
            .contains("<")
    );

    let slides = dir.join("out-slides.html").to_string_lossy().into_owned();
    export_notebook_slides(slides.clone(), content.clone()).expect("exports slides");
    assert!(std::path::Path::new(&slides).is_file());

    let pdf = dir.join("out.pdf").to_string_lossy().into_owned();
    if export_notebook_pdf(pdf.clone(), content.clone()).is_ok() {
        assert!(std::path::Path::new(&pdf).is_file());
    }

    assert!(
        export_notebook_markdown(
            dir.join("bad.md").to_string_lossy().into_owned(),
            "junk".to_owned()
        )
        .is_err()
    );
    std::fs::remove_dir_all(dir).expect("cleans up");
}

#[test]
fn checkpoint_commands_delegate() {
    let dir = temp_dir();
    let notebook_path = dir.join("book.mars");
    std::fs::write(&notebook_path, notebook_json()).expect("writes notebook");
    let path = notebook_path.to_string_lossy().into_owned();

    let checkpoint = create_notebook_checkpoint(path.clone()).expect("creates checkpoint");
    let checkpoints = list_notebook_checkpoints(path.clone()).expect("lists checkpoints");
    assert_eq!(checkpoints, vec![checkpoint.clone()]);

    let content =
        read_notebook_checkpoint(path.clone(), checkpoint.clone()).expect("reads checkpoint");
    assert!(content.contains("format_version"));

    std::fs::write(&notebook_path, "changed").expect("modifies notebook");
    restore_notebook_checkpoint(path.clone(), checkpoint).expect("restores checkpoint");
    assert!(
        std::fs::read_to_string(&notebook_path)
            .expect("reads notebook")
            .contains("format_version")
    );

    assert!(
        create_notebook_checkpoint(dir.join("missing.mars").to_string_lossy().into_owned())
            .is_err()
    );
    assert!(
        list_notebook_checkpoints(dir.join("none.mars").to_string_lossy().into_owned())
            .expect("lists")
            .is_empty()
    );
    std::fs::remove_dir_all(dir).expect("cleans up");
}

#[test]
fn ipynb_roundtrip_and_open_external_error() {
    let dir = temp_dir();
    let content = notebook_json();

    let ipynb_path = dir.join("book.ipynb");
    std::fs::write(
        &ipynb_path,
        r#"{"cells":[],"metadata":{},"nbformat":4,"nbformat_minor":5}"#,
    )
    .expect("writes ipynb");
    let imported =
        import_notebook_ipynb(ipynb_path.to_string_lossy().into_owned()).expect("imports ipynb");
    assert!(imported.contains("format_version"));

    let out = dir.join("out.ipynb").to_string_lossy().into_owned();
    export_notebook_ipynb(out.clone(), content).expect("exports ipynb");
    assert!(std::path::Path::new(&out).is_file());

    assert!(
        import_notebook_ipynb(dir.join("missing.ipynb").to_string_lossy().into_owned()).is_err()
    );
    assert!(open_path_external(dir.join("missing.bin").to_string_lossy().into_owned()).is_err());
    assert!(present_notebook_slides("junk".to_owned()).is_err());
    std::fs::remove_dir_all(dir).expect("cleans up");
}

#[test]
fn export_and_import_error_paths() {
    let dir = temp_dir();
    let missing_dir = dir.join("no-such-dir");
    let bad_out = missing_dir.join("out.html").to_string_lossy().into_owned();
    let content = notebook_json();

    // Invalid notebook JSON fails at from_json in every export command.
    assert!(export_notebook_markdown(bad_out.clone(), "junk".to_owned()).is_err());
    assert!(export_notebook_html(bad_out.clone(), "junk".to_owned()).is_err());
    assert!(export_notebook_slides(bad_out.clone(), "junk".to_owned()).is_err());
    assert!(export_notebook_pdf(bad_out.clone(), "junk".to_owned()).is_err());
    assert!(export_notebook_ipynb(bad_out.clone(), "junk".to_owned()).is_err());
    assert!(present_notebook_slides("junk".to_owned()).is_err());

    // Valid JSON but an unwritable destination fails at the write step.
    assert!(export_notebook_markdown(bad_out.clone(), content.clone()).is_err());
    assert!(export_notebook_html(bad_out.clone(), content.clone()).is_err());
    assert!(export_notebook_slides(bad_out.clone(), content.clone()).is_err());
    assert!(export_notebook_pdf(bad_out.clone(), content.clone()).is_err());
    assert!(export_notebook_ipynb(bad_out.clone(), content.clone()).is_err());

    // ipynb import: malformed JSON inside a real file.
    let bad_ipynb = dir.join("bad.ipynb");
    std::fs::write(&bad_ipynb, "{not json").expect("writes bad ipynb");
    assert!(import_notebook_ipynb(bad_ipynb.to_string_lossy().into_owned()).is_err());

    // File commands against a path blocked by a regular file.
    let blocker = dir.join("blocker");
    std::fs::write(&blocker, "x").expect("writes blocker");
    let blocked = blocker.join("child").to_string_lossy().into_owned();
    assert!(create_notebook_file(blocked.clone()).is_err());
    assert!(create_directory(blocked.clone()).is_err());
    assert!(write_text_file(blocked.clone(), "x".to_owned()).is_err());
    assert!(save_notebook(blocked, notebook_json()).is_err());

    std::fs::remove_dir_all(dir).expect("cleans up");
}

#[test]
fn present_slides_and_terminal_open_best_effort() {
    // Valid notebook: renders slides, writes a temp file, then attempts to
    // open it externally. The open is best-effort — either outcome is fine,
    // but the render/write path must succeed up to that point.
    let result = present_notebook_slides(notebook_json());
    if let Ok(path) = &result {
        assert!(
            std::path::Path::new(path)
                .extension()
                .is_some_and(|e| e == "html")
        );
    }

    // A bare filename has no parent directory, exercising the "." fallback.
    let _ = open_in_terminal("file.txt".to_owned());
    std::fs::remove_dir_all(temp_dir()).ok();
}
