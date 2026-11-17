use mars_core::{CellType, Output};
use mars_document::{
    CURRENT_FORMAT_VERSION, Cell, DocumentError, Notebook, NotebookHistory, from_json,
    to_json_pretty, validate,
};

#[test]
fn round_trips_notebook_json() {
    let mut notebook = Notebook::new();
    notebook
        .add_cell(Cell::new(CellType::Markdown, "# Hello"))
        .expect("unique ID");
    notebook
        .add_cell(Cell::new(CellType::Code, "1 + 1"))
        .expect("unique ID");

    let json = to_json_pretty(&notebook).expect("serializes");
    let restored = from_json(&json).expect("deserializes");

    assert_eq!(notebook, restored);
}

#[test]
fn supports_cell_operations() {
    let mut notebook = Notebook::new();
    notebook
        .add_cell(Cell::new(CellType::Code, "first"))
        .expect("unique ID");
    notebook
        .insert_cell(0, Cell::new(CellType::Markdown, "intro"))
        .expect("valid index");
    notebook.move_cell(0, 1).expect("valid indexes");

    assert_eq!(notebook.cell(0).expect("cell exists").source, "first");
    assert_eq!(
        notebook.remove_cell(1).expect("cell exists").source,
        "intro"
    );
}

#[test]
fn migrates_format_version_one() {
    let notebook = from_json(r#"{"format_version":1,"metadata":{"language":"rust"},"cells":[]}"#)
        .expect("migrates");
    assert_eq!(notebook.format_version, CURRENT_FORMAT_VERSION);
}

#[test]
fn supports_undo_and_redo_history() {
    let initial = Notebook::new();
    let mut history = NotebookHistory::new(initial.clone(), 4);
    let mut changed = initial;
    changed
        .add_cell(Cell::new(CellType::Code, "1 + 1"))
        .expect("adds");
    history.record(changed.clone());
    assert_eq!(history.current(), Some(&changed));
    assert!(history.undo().is_some());
    assert!(history.redo().is_some());
}

#[test]
fn rejects_execution_data_on_markdown() {
    let mut notebook = Notebook::new();
    let mut cell = Cell::new(CellType::Markdown, "text");
    cell.outputs.push(Output::Text {
        text: "invalid".to_owned(),
    });
    notebook.cells.push(cell);

    assert!(matches!(
        validate(&notebook),
        Err(DocumentError::NonCodeExecutionData(_))
    ));
}

#[test]
fn cell_operations_reject_invalid_input() {
    let mut notebook = Notebook::new();
    notebook
        .add_cell(Cell::new(CellType::Code, "first"))
        .expect("add");
    let duplicate = notebook.cell(0).expect("cell").clone();

    assert!(matches!(
        notebook.add_cell(duplicate.clone()),
        Err(DocumentError::DuplicateCellId(_))
    ));
    assert!(matches!(
        notebook.insert_cell(99, Cell::new(CellType::Code, "x")),
        Err(DocumentError::CellIndexOutOfBounds(99))
    ));
    assert!(matches!(
        notebook.insert_cell(0, duplicate),
        Err(DocumentError::DuplicateCellId(_))
    ));
    assert!(matches!(
        notebook.remove_cell(42),
        Err(DocumentError::CellIndexOutOfBounds(42))
    ));
    assert!(matches!(
        notebook.move_cell(0, 9),
        Err(DocumentError::CellIndexOutOfBounds(_))
    ));
    assert!(matches!(
        notebook.cell(7),
        Err(DocumentError::CellIndexOutOfBounds(7))
    ));
    assert!(matches!(
        notebook.cell_mut(7),
        Err(DocumentError::CellIndexOutOfBounds(7))
    ));

    notebook
        .cell_mut(0)
        .expect("cell mut")
        .source
        .push_str(" // edited");
    assert!(notebook.cell(0).expect("cell").source.ends_with("edited"));
}

#[test]
fn history_evicts_past_and_clears_redo_on_record() {
    let initial = Notebook::new();
    let mut history = NotebookHistory::new(initial.clone(), 2);
    assert_eq!(history.current(), Some(&initial));
    // Undo at the root is a no-op.
    assert!(history.undo().is_none());

    for source in ["a", "b", "c"] {
        let mut notebook = history.current().expect("current").clone();
        notebook
            .add_cell(Cell::new(CellType::Code, source))
            .expect("add");
        history.record(notebook);
    }
    // Capacity 2 keeps only the two most recent snapshots.
    assert_eq!(history.current().expect("current").cells.len(), 3);
    history.undo().expect("undo");
    assert_eq!(history.current().expect("current").cells.len(), 2);
    // A new record discards the redo branch.
    let mut notebook = history.current().expect("current").clone();
    notebook
        .add_cell(Cell::new(CellType::Code, "d"))
        .expect("add");
    history.record(notebook);
    assert!(history.redo().is_none());
}
