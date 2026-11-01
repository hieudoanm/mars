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
