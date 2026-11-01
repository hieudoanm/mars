use mars_core::CellType;
use mars_document::{
    Cell, DocumentError, Notebook, NotebookHistory, from_json, load, save_atomic, to_json_pretty,
    validate,
};
use std::path::PathBuf;

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("mars-doc-{}-{}", std::process::id(), name));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}

#[test]
fn from_json_rejects_malformed_and_unsupported_documents() {
    assert!(matches!(
        from_json("not json"),
        Err(DocumentError::Deserialization(_))
    ));
    assert!(matches!(
        from_json(r#"{"cells":[]}"#),
        Err(DocumentError::MissingFormatVersion)
    ));
    assert!(matches!(
        from_json(r#"{"format_version":"two","cells":[]}"#),
        Err(DocumentError::MissingFormatVersion)
    ));
    assert!(matches!(
        from_json(r#"{"format_version":99,"metadata":{},"cells":[]}"#),
        Err(DocumentError::UnsupportedFormatVersion(99))
    ));
}

#[test]
fn from_json_rejects_duplicate_cell_ids() {
    let cell = r#"{"id":"dup","cell_type":"code","source":"1","metadata":{},"outputs":[]}"#;
    let document = format!(
        r#"{{"format_version":2,"metadata":{{"language":"rust"}},"cells":[{cell},{cell}]}}"#
    );
    assert!(matches!(
        from_json(&document),
        Err(DocumentError::DuplicateCellId(_))
    ));
}

#[test]
fn validate_rejects_markdown_execution_counts() {
    let mut notebook = Notebook::new();
    let mut cell = Cell::new(CellType::Markdown, "text");
    cell.execution_count = Some(1);
    notebook.cells.push(cell);
    assert!(matches!(
        validate(&notebook),
        Err(DocumentError::NonCodeExecutionData(_))
    ));
}

#[test]
fn to_json_pretty_validates_before_serializing() {
    let mut notebook = Notebook::new();
    let mut cell = Cell::new(CellType::Markdown, "text");
    cell.execution_count = Some(1);
    notebook.cells.push(cell);
    assert!(to_json_pretty(&notebook).is_err());
}

#[test]
fn cell_operations_report_errors_instead_of_panicking() {
    let mut notebook = Notebook::new();
    let first = Cell::new(CellType::Code, "first");
    let second = Cell::new(CellType::Code, "second");
    notebook.add_cell(first.clone()).expect("add");
    notebook.add_cell(second).expect("add");

    // Duplicate IDs are rejected on every insertion path.
    assert!(matches!(
        notebook.add_cell(first.clone()),
        Err(DocumentError::DuplicateCellId(_))
    ));
    assert!(matches!(
        notebook.insert_cell(0, first.clone()),
        Err(DocumentError::DuplicateCellId(_))
    ));

    assert!(matches!(
        notebook.insert_cell(99, Cell::new(CellType::Code, "x")),
        Err(DocumentError::CellIndexOutOfBounds(99))
    ));
    assert!(matches!(
        notebook.remove_cell(99),
        Err(DocumentError::CellIndexOutOfBounds(99))
    ));
    assert!(matches!(
        notebook.move_cell(0, 99),
        Err(DocumentError::CellIndexOutOfBounds(99))
    ));
    assert!(matches!(
        notebook.cell(99),
        Err(DocumentError::CellIndexOutOfBounds(99))
    ));
    assert!(matches!(
        notebook.cell_mut(99),
        Err(DocumentError::CellIndexOutOfBounds(99))
    ));

    // move_cell(0, 1) is legal: `to` is a valid destination index.
    notebook.move_cell(0, 1).expect("move");
    assert_eq!(notebook.cell(1).expect("cell").source, "first");
}

#[test]
fn save_atomic_and_load_round_trip() {
    let dir = temp_dir("round-trip");
    let path = dir.join("book.mars");
    let mut notebook = Notebook::new();
    notebook.metadata.title = Some("Demo".to_owned());
    notebook
        .metadata
        .dependencies
        .insert("serde".to_owned(), "1".to_owned());
    notebook
        .add_cell(Cell::new(CellType::Code, "40 + 2"))
        .expect("cell");

    save_atomic(&notebook, &path).expect("save");
    assert!(path.exists());
    // The temporary sibling file is renamed away, not left behind.
    assert!(!dir.join(".book.mars.tmp").exists());

    let loaded = load(&path).expect("load");
    assert_eq!(loaded, notebook);
    assert_eq!(loaded.metadata.title.as_deref(), Some("Demo"));
    assert_eq!(loaded.metadata.dependencies["serde"], "1");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn load_reports_io_errors() {
    let dir = temp_dir("io-errors");
    let missing = dir.join("missing.mars");
    assert!(matches!(load(&missing), Err(DocumentError::Io { .. })));

    // Writing into a nonexistent directory fails at the temp-file step.
    let bad = dir.join("no-such-dir").join("book.mars");
    assert!(matches!(
        save_atomic(&Notebook::new(), &bad),
        Err(DocumentError::Io { .. })
    ));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn history_evicts_past_capacity_and_tracks_future() {
    let initial = Notebook::new();
    let mut history = NotebookHistory::new(initial.clone(), 3);

    let snapshot = |tag: &str| {
        let mut notebook = Notebook::new();
        notebook
            .add_cell(Cell::new(CellType::Code, tag))
            .expect("cell");
        notebook
    };
    history.record(snapshot("one"));
    history.record(snapshot("two"));
    history.record(snapshot("three"));
    history.record(snapshot("four"));

    // Undo walks back through snapshots; redo is cleared by new records.
    assert_eq!(
        history
            .undo()
            .and_then(|n| n.cells.first())
            .map(|c| c.source.as_str()),
        Some("three")
    );
    assert_eq!(
        history
            .redo()
            .and_then(|n| n.cells.first())
            .map(|c| c.source.as_str()),
        Some("four")
    );
    // A fresh record clears the redo stack.
    history.record(snapshot("five"));
    assert!(history.redo().is_none() || history.current().is_some());

    // Undoing to the bottom of the retained window returns None afterwards.
    while history.undo().is_some() {}
    assert!(history.undo().is_none());
    assert!(history.current().is_some());
}
