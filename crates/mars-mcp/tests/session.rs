use mars_core::CellType;
use mars_document::{Cell, Notebook};
use mars_kernel::RuntimeBackendKind;
use mars_mcp::MarsSession;
use std::path::PathBuf;

fn temp_dir(name: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("mars-mcp-session-{}-{}", std::process::id(), name));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}

#[test]
fn create_open_save_round_trip() {
    let dir = temp_dir("round-trip");
    let path = dir.join("session.mars");
    let mut session = MarsSession::new(RuntimeBackendKind::Replay, false, None);

    let mut notebook = Notebook::new();
    notebook
        .add_cell(Cell::new(CellType::Code, "1 + 1"))
        .expect("cell");
    session.create_notebook(&path, notebook);
    assert!(session.is_dirty(&path));
    assert!(session.save_notebook(&path).expect("save"));
    assert!(!session.is_dirty(&path));
    // Saving a clean notebook is a no-op.
    assert!(!session.save_notebook(&path).expect("save again"));

    let mut reopened_session = MarsSession::new(RuntimeBackendKind::Replay, false, None);
    let reopened = reopened_session.open_notebook(&path).expect("open");
    assert_eq!(reopened.cells.len(), 1);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn open_notebook_errors_for_missing_files() {
    let dir = temp_dir("missing");
    let mut session = MarsSession::new(RuntimeBackendKind::Replay, true, None);
    let missing = dir.join("missing.mars");
    assert!(session.open_notebook(&missing).is_err());
    assert!(session.notebook_mut(&missing).is_err());
    assert!(session.save_notebook(&missing).is_err());
    assert!(!session.is_dirty(&missing));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn notebook_mut_marks_dirty_and_mutations_persist() {
    let dir = temp_dir("dirty");
    let path = dir.join("dirty.mars");
    let mut notebook = Notebook::new();
    notebook
        .add_cell(Cell::new(CellType::Code, "1"))
        .expect("cell");
    mars_document::save_atomic(&notebook, &path).expect("save");

    let mut session = MarsSession::new(RuntimeBackendKind::Replay, true, None);
    session.open_notebook(&path).expect("open");
    assert!(!session.is_dirty(&path));

    {
        let notebook = session.notebook_mut(&path).expect("mutable");
        notebook.cells[0].source = "2 + 2".to_owned();
    }
    assert!(session.is_dirty(&path));
    assert!(session.save_notebook(&path).expect("save"));
    assert!(!session.is_dirty(&path));

    let reloaded = mars_document::load(&path).expect("reload");
    assert_eq!(reloaded.cells[0].source, "2 + 2");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn default_language_and_cached_notebook_access() {
    let mut session = MarsSession::new(RuntimeBackendKind::Replay, true, Some("python".to_owned()));
    assert_eq!(session.default_language(), "python");
    let default = MarsSession::new(RuntimeBackendKind::Replay, true, None);
    assert_eq!(default.default_language(), "rust");

    let dir = temp_dir("cache");
    let path = dir.join("cache.mars");
    let mut notebook = Notebook::new();
    notebook.metadata.language = "julia".to_owned();
    mars_document::save_atomic(&notebook, &path).expect("save");

    session.open_notebook(&path).expect("open");
    assert_eq!(
        session.notebook(&path).expect("notebook").metadata.language,
        "julia"
    );
    assert_eq!(session.notebook(&dir.join("other.mars")), None);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn kernel_lifecycle_is_lazy_and_scoped() {
    let mut session = MarsSession::new(RuntimeBackendKind::Replay, true, None);
    // No kernel exists yet: status is read-only and reports nothing.
    assert_eq!(session.kernel_status(None, "rust"), None);

    // Unknown languages fail at creation time.
    assert!(session.kernel_for(None, "cobol").is_err());

    // Creating a kernel makes status available without executing anything.
    session.kernel_for(None, "rust").expect("kernel");
    assert!(session.kernel_status(None, "rust").is_some());
    // A different language still has no kernel.
    assert_eq!(session.kernel_status(None, "python"), None);
}
