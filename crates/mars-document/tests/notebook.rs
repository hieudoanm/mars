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
