use mars_core::CellType;
use mars_document::{Cell, Notebook};
use mars_kernel::RuntimeBackendKind;
use mars_mcp::handlers::{call_document_tool, notebook_summary, parse_cell_type};
use mars_mcp::{MarsSession, tools};
use serde_json::{Value, json};
use std::path::PathBuf;

#[test]
fn parses_cell_type() {
    assert_eq!(parse_cell_type("code").expect("code"), CellType::Code);
    assert!(parse_cell_type("bogus").is_err());
}

#[test]
fn tools_registered() {
    assert!(
        tools::definitions()
            .iter()
            .any(|t| t.name == "mars_cell_add")
    );
}

fn temp_notebook_path(name: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("mars-mcp-handlers-{}-{}", std::process::id(), name));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir.join("notebook.mars")
}

fn call(session: &mut MarsSession, name: &str, args: Value) -> Result<Value, String> {
    call_document_tool(session, name, &args).expect("document tool")
}

#[test]
fn notebook_summary_includes_cell_details() {
    let mut notebook = Notebook::new();
    notebook.metadata.language = "python".to_owned();
    let mut cell = Cell::new(CellType::Code, "print('hi')\nprint('bye')");
    cell.metadata.insert("language".to_owned(), json!("python"));
    cell.outputs.push(mars_core::Output::Text {
        text: "hi".to_owned(),
    });
    notebook.add_cell(cell).expect("cell");

    let summary = notebook_summary(&notebook);
    assert_eq!(summary["cell_count"], 1);
    assert_eq!(summary["cells"][0]["cell_type"], "code");
    assert_eq!(summary["cells"][0]["language"], "python");
    assert_eq!(summary["cells"][0]["source_lines"], 2);
    assert_eq!(summary["cells"][0]["has_outputs"], json!(true));
}

#[test]
fn document_tools_manage_notebook_cells() {
    let path = temp_notebook_path("document-tools");
    let path_string = path.to_string_lossy().to_string();
    let mut session = MarsSession::new(RuntimeBackendKind::Replay, true, None);

    let created = call(
        &mut session,
        "mars_notebook_new",
        json!({ "path": path_string, "language": "rust" }),
    )
    .expect("new notebook");
    assert_eq!(created["summary"]["cell_count"], 1);

    let added = call(
        &mut session,
        "mars_cell_add",
        json!({
            "path": path_string,
            "cell_type": "code",
            "source": "let x = 1;",
            "index": 0,
            "language": "rust"
        }),
    )
    .expect("add cell");
    assert_eq!(added["cell_count"], 2);

    call(
        &mut session,
        "mars_cell_add",
        json!({
            "path": path_string,
            "cell_type": "markdown",
            "source": "# Title"
        }),
    )
    .expect("add markdown");

    let read = call(
        &mut session,
        "mars_notebook_read",
        json!({ "path": path_string, "include_outputs": false }),
    )
    .expect("read notebook");
    assert_eq!(
        read["notebook"]["cells"].as_array().expect("cells").len(),
        3
    );
    assert_eq!(read["notebook"]["cells"][0]["outputs"], json!([]));

    let cell = call(
        &mut session,
        "mars_notebook_read",
        json!({ "path": path_string, "cell_index": 0, "include_outputs": true }),
    )
    .expect("read cell");
    assert_eq!(cell["cell"]["language"], "rust");

    let inspected = call(
        &mut session,
        "mars_notebook_inspect",
        json!({ "path": path_string }),
    )
    .expect("inspect");
    assert_eq!(inspected["cell_count"], 3);
    assert_eq!(inspected["language"], "rust");

    call(
        &mut session,
        "mars_cell_update",
        json!({
            "path": path_string,
            "index": 0,
            "source": "let y = 2;"
        }),
    )
    .expect("update cell");

    let moved = call(
        &mut session,
        "mars_cell_move",
        json!({ "path": path_string, "from": 2, "to": 0 }),
    )
    .expect("move cell");
    assert_eq!(moved["cell_count"], 3);

    let deleted = call(
        &mut session,
        "mars_cell_delete",
        json!({ "path": path_string, "index": 1 }),
    )
    .expect("delete cell");
    assert_eq!(deleted["cell_count"], 2);

    let saved = call(
        &mut session,
        "mars_notebook_save",
        json!({ "path": path_string }),
    )
    .expect("save");
    assert_eq!(saved["saved"], json!(false));
    assert!(path.exists());

    let _ = std::fs::remove_dir_all(path.parent().expect("parent"));
}

#[test]
fn document_tools_report_errors() {
    let path = temp_notebook_path("document-errors");
    let path_string = path.to_string_lossy().to_string();
    let mut session = MarsSession::new(RuntimeBackendKind::Replay, true, None);

    assert!(call_document_tool(&mut session, "mars_unknown", &json!({})).is_none());
    assert!(
        call(
            &mut session,
            "mars_notebook_open",
            json!({ "path": path_string })
        )
        .is_err()
    );
    assert!(
        call(
            &mut session,
            "mars_notebook_new",
            json!({ "path": path_string, "language": "rust" })
        )
        .is_ok()
    );
    assert!(
        call(
            &mut session,
            "mars_notebook_new",
            json!({ "path": path_string })
        )
        .is_err()
    );
    assert!(
        call(
            &mut session,
            "mars_cell_add",
            json!({ "path": path_string, "cell_type": "bogus" })
        )
        .is_err()
    );
    assert!(
        call(
            &mut session,
            "mars_cell_update",
            json!({ "path": path_string, "index": 0 })
        )
        .is_err()
    );
    assert!(
        call(
            &mut session,
            "mars_cell_delete",
            json!({ "path": path_string, "index": 99 })
        )
        .is_err()
    );
    assert!(
        call(
            &mut session,
            "mars_cell_move",
            json!({ "path": path_string, "from": 0, "to": 99 })
        )
        .is_err()
    );

    let _ = std::fs::remove_dir_all(path.parent().expect("parent"));
}

#[test]
fn read_defaults_to_include_outputs_and_strips_on_request() {
    let path = temp_notebook_path("read-defaults");
    let path_string = path.to_string_lossy().to_string();
    let mut session = MarsSession::new(RuntimeBackendKind::Replay, true, None);

    call(
        &mut session,
        "mars_notebook_new",
        json!({ "path": path_string, "language": "rust" }),
    )
    .expect("new notebook");
    call(
        &mut session,
        "mars_cell_add",
        json!({
            "path": path_string,
            "cell_type": "code",
            "source": "1 + 1",
            "index": 1
        }),
    )
    .expect("add cell");

    // Omitting include_outputs defaults to true.
    let read = call(
        &mut session,
        "mars_notebook_read",
        json!({ "path": path_string }),
    )
    .expect("read notebook");
    assert!(read["notebook"]["cells"].is_array());

    // Single-cell read with include_outputs=false strips outputs.
    let cell = call(
        &mut session,
        "mars_notebook_read",
        json!({ "path": path_string, "cell_index": 1, "include_outputs": false }),
    )
    .expect("read cell");
    assert_eq!(cell["cell"]["outputs"], json!([]));
    assert!(cell["cell"]["language"].is_string());

    // Add at an out-of-range index errors; update with cell_type works.
    assert!(
        call(
            &mut session,
            "mars_cell_add",
            json!({ "path": path_string, "cell_type": "code", "index": 99 })
        )
        .is_err()
    );
    call(
        &mut session,
        "mars_cell_update",
        json!({ "path": path_string, "index": 1, "cell_type": "markdown" }),
    )
    .expect("update cell type");

    let _ = std::fs::remove_dir_all(path.parent().expect("parent"));
}

#[test]
fn document_tools_cover_update_move_read_inspect_save() {
    let path = temp_notebook_path("doc-more");
    let path_string = path.to_string_lossy().to_string();
    let mut session = MarsSession::new(RuntimeBackendKind::Replay, true, None);

    call(
        &mut session,
        "mars_notebook_new",
        json!({ "path": path_string, "language": "rust" }),
    )
    .expect("new");
    call(
        &mut session,
        "mars_cell_add",
        json!({ "path": path_string, "cell_type": "markdown", "source": "# Hi" }),
    )
    .expect("add md");
    call(
        &mut session,
        "mars_cell_add",
        json!({
            "path": path_string,
            "cell_type": "code",
            "source": "let x = 1;",
            "index": 0,
            "language": "rust",
        }),
    )
    .expect("add code at index");

    // Update source and cell type; move cells; read a single cell.
    call(
        &mut session,
        "mars_cell_update",
        json!({ "path": path_string, "index": 0, "source": "let y = 2;" }),
    )
    .expect("update source");
    call(
        &mut session,
        "mars_cell_update",
        json!({ "path": path_string, "index": 1, "cell_type": "raw" }),
    )
    .expect("update type");
    call(
        &mut session,
        "mars_cell_move",
        json!({ "path": path_string, "from": 0, "to": 1 }),
    )
    .expect("move");
    let single = call(
        &mut session,
        "mars_notebook_read",
        json!({ "path": path_string, "cell_index": 0, "include_outputs": false }),
    )
    .expect("read cell");
    assert!(single.is_object());

    // Inspect + save round out the document tools.
    call(
        &mut session,
        "mars_notebook_inspect",
        json!({ "path": path_string }),
    )
    .expect("inspect");
    call(
        &mut session,
        "mars_notebook_save",
        json!({ "path": path_string }),
    )
    .expect("save");

    let _ = std::fs::remove_dir_all(path.parent().expect("parent"));
}

#[test]
fn document_tools_reject_malformed_arguments() {
    let mut session = MarsSession::new(RuntimeBackendKind::Replay, true, None);
    // Every parse::<Args> error closure fires on type-mismatched input.
    for (tool, args) in [
        ("mars_notebook_new", json!({ "path": 1 })),
        ("mars_notebook_open", json!({})),
        (
            "mars_notebook_read",
            json!({ "path": "x", "cell_index": "zero" }),
        ),
        ("mars_notebook_save", json!({ "path": null })),
        ("mars_notebook_inspect", json!({ "path": 7 })),
        ("mars_cell_add", json!({ "path": "x", "cell_type": 3 })),
        ("mars_cell_update", json!({ "path": "x" })),
        ("mars_cell_delete", json!({ "path": "x", "index": "i" })),
        (
            "mars_cell_move",
            json!({ "path": "x", "from": "a", "to": 1 }),
        ),
    ] {
        assert!(
            call_document_tool(&mut session, tool, &args)
                .expect("dispatch")
                .is_err(),
            "{tool} should reject {args}"
        );
    }
}

#[test]
fn document_tools_cover_edge_branches() {
    let dir = std::env::temp_dir().join(format!(
        "mars-mcp-edge-{}-{}",
        std::process::id(),
        "handlers"
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("edge.mars");
    let path_string = path.to_string_lossy().to_string();
    let mut session = MarsSession::new(RuntimeBackendKind::Replay, true, None);

    // New without explicit language uses the rust default.
    let created = call(
        &mut session,
        "mars_notebook_new",
        json!({ "path": path_string }),
    )
    .expect("new notebook");
    assert_eq!(created["summary"]["metadata"]["language"], "rust");

    // Save to an unwritable location surfaces the atomic-save error.
    let bad = dir.join("missing-dir").join("deep").join("nb.mars");
    let bad_string = bad.to_string_lossy().to_string();
    let mut other = Notebook::new();
    other
        .add_cell(Cell::new(CellType::Code, "let z = 1;"))
        .expect("cell");
    session.create_notebook(&bad, other);
    assert!(
        call(
            &mut session,
            "mars_notebook_save",
            json!({ "path": bad_string })
        )
        .is_err()
    );

    // Read paths: include_outputs=false on both whole notebook and single cell,
    // plus out-of-range cell index.
    call(
        &mut session,
        "mars_cell_add",
        json!({ "path": path_string, "cell_type": "code", "source": "1 + 1" }),
    )
    .expect("add");
    let whole = call(
        &mut session,
        "mars_notebook_read",
        json!({ "path": path_string, "include_outputs": false }),
    )
    .expect("read");
    assert!(whole["notebook"]["cells"].is_array());
    let single = call(
        &mut session,
        "mars_notebook_read",
        json!({ "path": path_string, "cell_index": 1, "include_outputs": false }),
    )
    .expect("read cell");
    assert_eq!(single["cell"]["outputs"], json!([]));
    assert!(
        call(
            &mut session,
            "mars_notebook_read",
            json!({ "path": path_string, "cell_index": 99 })
        )
        .is_err()
    );

    // Update/delete/move out-of-range indexes hit the cell accessor errors.
    assert!(
        call(
            &mut session,
            "mars_cell_update",
            json!({ "path": path_string, "index": 42, "source": "x" })
        )
        .is_err()
    );
    assert!(
        call(
            &mut session,
            "mars_cell_delete",
            json!({ "path": path_string, "index": 42 })
        )
        .is_err()
    );
    assert!(
        call(
            &mut session,
            "mars_cell_move",
            json!({ "path": path_string, "from": 0, "to": 42 })
        )
        .is_err()
    );
    // Update with an unknown cell type and add with out-of-range index.
    assert!(
        call(
            &mut session,
            "mars_cell_update",
            json!({ "path": path_string, "index": 0, "cell_type": "bogus" })
        )
        .is_err()
    );
    assert!(
        call(
            &mut session,
            "mars_cell_add",
            json!({ "path": path_string, "cell_type": "raw", "index": 99 })
        )
        .is_err()
    );
    let _ = dir;
}
