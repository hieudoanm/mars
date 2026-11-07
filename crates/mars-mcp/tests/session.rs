use mars_core::CellType;
use mars_document::{Cell, Notebook};
use mars_kernel::RuntimeBackendKind;
use mars_mcp::MarsSession;

#[test]
fn create_open_save_round_trip() {
    let dir = std::env::temp_dir().join(format!("mars-mcp-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
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

    let mut reopened_session = MarsSession::new(RuntimeBackendKind::Replay, false, None);
    let reopened = reopened_session.open_notebook(&path).expect("open");
    assert_eq!(reopened.cells.len(), 1);

    let _ = std::fs::remove_dir_all(&dir);
}
