use mars::commands::run_cli;
use mars_core::CellType;
use mars_document::{Cell, Notebook, load, save_atomic};
use serde_json::json;
use std::path::PathBuf;

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("mars-cli-cmd-{}-{}", std::process::id(), name));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

fn write_notebook(path: &PathBuf, notebook: &Notebook) {
    save_atomic(notebook, path).expect("save notebook");
}

fn markdown_only_notebook() -> Notebook {
    let mut notebook = Notebook::new();
    notebook
        .add_cell(Cell::new(CellType::Markdown, "# Title"))
        .expect("cell");
    notebook
}

#[test]
fn rejects_missing_and_unknown_commands() {
    assert!(run_cli(vec![]).is_err());
    assert!(run_cli(args(&["bogus"])).is_err());
    assert!(run_cli(args(&["new"])).is_err());
    assert!(run_cli(args(&["inspect"])).is_err());
    assert!(run_cli(args(&["run"])).is_err());
    assert!(run_cli(args(&["import"])).is_err());
    assert!(run_cli(args(&["export"])).is_err());
}

#[test]
fn new_inspect_and_open_missing_file() {
    let dir = temp_dir("new-inspect");
    let path = dir.join("demo.mars");
    let path_string = path.to_string_lossy().to_string();

    run_cli(args(&["new", &path_string])).expect("new notebook");
    assert!(path.exists());
    let notebook = load(&path).expect("load");
    assert_eq!(notebook.cells.len(), 1);
    assert_eq!(notebook.cells[0].cell_type, CellType::Markdown);

    run_cli(args(&["new", &path_string, "python"])).expect("new with language");
    let notebook = load(&path).expect("load");
    assert_eq!(notebook.metadata.language, "python");

    let other = dir.join("other.mars");
    assert!(run_cli(args(&["new", &other.to_string_lossy(), "   "])).is_err());

    run_cli(args(&["inspect", &path_string])).expect("inspect");

    let missing = dir.join("missing.mars");
    assert!(run_cli(args(&["inspect", &missing.to_string_lossy()])).is_err());
    assert!(run_cli(args(&["open", &missing.to_string_lossy()])).is_err());

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn run_executes_notebook_and_reports() {
    let dir = temp_dir("run");
    let path = dir.join("run.mars");
    write_notebook(&path, &markdown_only_notebook());
    let path_string = path.to_string_lossy().to_string();

    run_cli(args(&[
        "run",
        &path_string,
        "--no-save",
        "--report",
        "--timeout-ms",
        "5000",
    ]))
    .expect("run markdown-only notebook");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn run_rejects_deterministic_without_lockfile() {
    let dir = temp_dir("run-deterministic");
    let path = dir.join("deps.mars");
    let mut notebook = Notebook::new();
    notebook
        .metadata
        .dependencies
        .insert("serde".to_owned(), "1".to_owned());
    notebook
        .add_cell(Cell::new(CellType::Code, "1 + 1"))
        .expect("cell");
    write_notebook(&path, &notebook);

    let error = run_cli(args(&[
        "run",
        &path.to_string_lossy(),
        "--deterministic",
        "--no-save",
    ]))
    .expect_err("deterministic requires cargo_lock");
    assert!(error.contains("cargo_lock"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn import_and_export_round_trip() {
    let dir = temp_dir("io");
    let ipynb = dir.join("input.ipynb");
    std::fs::write(
        &ipynb,
        r##"{
          "cells": [
            {"cell_type": "markdown", "metadata": {}, "source": ["# Imported\n"], "outputs": []},
            {"cell_type": "code", "execution_count": 1, "metadata": {}, "source": "1 + 1", "outputs": [
              {"output_type": "stream", "name": "stdout", "text": ["2\n"]}
            ]}
          ],
          "metadata": {"language_info": {"name": "rust"}},
          "nbformat": 4,
          "nbformat_minor": 5
        }"##,
    )
    .expect("write ipynb");

    run_cli(args(&["import", &ipynb.to_string_lossy()])).expect("import");
    let mars_path = dir.join("input.mars");
    assert!(mars_path.exists());
    let notebook = load(&mars_path).expect("load imported");
    assert_eq!(notebook.cells.len(), 2);

    let explicit = dir.join("explicit.mars");
    run_cli(args(&[
        "import",
        &ipynb.to_string_lossy(),
        &explicit.to_string_lossy(),
    ]))
    .expect("import explicit");
    assert!(explicit.exists());

    for (format, extension) in [
        ("ipynb", "ipynb"),
        ("html", "html"),
        ("md", "md"),
        ("markdown", "md"),
        ("pdf", "pdf"),
    ] {
        let output = dir.join(format!("exported.{extension}"));
        run_cli(args(&[
            "export",
            &mars_path.to_string_lossy(),
            format,
            &output.to_string_lossy(),
        ]))
        .unwrap_or_else(|error| panic!("export {format}: {error}"));
        assert!(output.exists(), "{format} output missing");
    }

    let default_html = dir.join("input.html");
    run_cli(args(&["export", &mars_path.to_string_lossy(), "html"])).expect("default export");
    assert!(default_html.exists());

    let pdf_out = dir.join("sized.pdf");
    run_cli(args(&[
        "export",
        &mars_path.to_string_lossy(),
        "pdf",
        "--page-size=a4",
        &pdf_out.to_string_lossy(),
    ]))
    .expect("export with page size");
    assert!(pdf_out.exists());

    assert!(run_cli(args(&["export", &mars_path.to_string_lossy(), "bogus",])).is_err());
    assert!(
        run_cli(args(&[
            "export",
            &mars_path.to_string_lossy(),
            "pdf",
            "--page-size=bogus",
            &dir.join("bad.pdf").to_string_lossy(),
        ]))
        .is_err()
    );
    assert!(
        run_cli(args(&[
            "export",
            &dir.join("missing.mars").to_string_lossy()
        ]))
        .is_err()
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn run_python_notebook_executes_and_saves_outputs() {
    if std::process::Command::new("python3")
        .arg("--version")
        .output()
        .is_err()
    {
        return;
    }
    let dir = temp_dir("run-python");
    let path = dir.join("py.mars");
    let mut notebook = Notebook::new();
    notebook.metadata.language = "python".to_owned();
    notebook
        .add_cell(Cell::new(CellType::Markdown, "# Py"))
        .expect("md cell");
    notebook
        .add_cell(Cell::new(CellType::Code, "print(40 + 2)"))
        .expect("code cell");
    notebook
        .add_cell(Cell::new(
            CellType::Code,
            "import json\nprint(json.dumps({'a': 1}))",
        ))
        .expect("json cell");
    write_notebook(&path, &notebook);

    run_cli(args(&[
        "run",
        &path.to_string_lossy(),
        "--report",
        "--timeout-ms",
        "30000",
    ]))
    .expect("run python notebook");

    let saved = load(&path).expect("load saved");
    assert!(saved.cells.iter().any(|cell| !cell.outputs.is_empty()));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn run_python_notebook_reports_failure_and_stops() {
    if std::process::Command::new("python3")
        .arg("--version")
        .output()
        .is_err()
    {
        return;
    }
    let dir = temp_dir("run-python-fail");
    let path = dir.join("fail.mars");
    let mut notebook = Notebook::new();
    notebook.metadata.language = "python".to_owned();
    notebook
        .add_cell(Cell::new(CellType::Code, "raise ValueError('boom')"))
        .expect("cell");
    notebook
        .add_cell(Cell::new(CellType::Code, "print('never')"))
        .expect("cell2");
    write_notebook(&path, &notebook);

    let error = run_cli(args(&[
        "run",
        &path.to_string_lossy(),
        "--timeout-ms",
        "30000",
    ]))
    .expect_err("failing cell should error");
    assert!(error.contains("cell 1"), "unexpected error: {error}");

    // With the default save behaviour the failed outputs are persisted.
    let saved = load(&path).expect("load");
    assert!(
        saved.cells[0]
            .outputs
            .iter()
            .any(|output| matches!(output, mars_core::Output::Error { .. }))
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn run_rust_notebook_executes_code_cell() {
    if std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .is_err()
    {
        return;
    }
    let dir = temp_dir("run-rust");
    let path = dir.join("rust.mars");
    let mut notebook = Notebook::new();
    notebook
        .add_cell(Cell::new(CellType::Code, "println!(\"hi from mars\");"))
        .expect("cell");
    write_notebook(&path, &notebook);

    run_cli(args(&[
        "run",
        &path.to_string_lossy(),
        "--no-save",
        "--timeout-ms",
        "120000",
    ]))
    .expect("run rust notebook");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn run_unsupported_language_errors() {
    let dir = temp_dir("run-lang");
    let path = dir.join("lang.mars");
    let mut notebook = Notebook::new();
    notebook.metadata.language = "cobol".to_owned();
    notebook
        .add_cell(Cell::new(CellType::Code, "DISPLAY 'HI'."))
        .expect("cell");
    write_notebook(&path, &notebook);

    assert!(run_cli(args(&["run", &path.to_string_lossy()])).is_err());

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn run_missing_file_errors() {
    let dir = temp_dir("run-missing");
    let missing = dir.join("missing.mars");
    assert!(run_cli(args(&["run", &missing.to_string_lossy()])).is_err());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn kernel_subcommands_dispatch() {
    run_cli(args(&["kernel", "list"])).expect("kernel list");
    run_cli(args(&["kernel", "restart"])).expect("kernel restart");
    assert!(run_cli(args(&["kernel"])).is_err());
    assert!(run_cli(args(&["kernel", "bogus"])).is_err());
}

#[test]
fn mcp_and_tools_subcommands_dispatch() {
    assert!(run_cli(args(&["mcp"])).is_err());
    assert!(run_cli(args(&["mcp", "bogus"])).is_err());
    assert!(run_cli(args(&["mcp", "serve", "--notebook", "/nonexistent/x.mars"])).is_err());
    assert!(run_cli(args(&["tools"])).is_err());
    assert!(
        run_cli(args(&[
            "tools",
            "bogus",
            "--server",
            "definitely-not-a-binary-xyz"
        ]))
        .is_err()
    );
}

#[test]
fn import_preserves_execution_outputs() {
    let dir = temp_dir("import-outputs");
    let ipynb = dir.join("rich.ipynb");
    std::fs::write(
        &ipynb,
        json!({
            "cells": [{
                "cell_type": "code",
                "execution_count": 3,
                "metadata": {},
                "source": "display()",
                "outputs": [
                    {"output_type": "stream", "name": "stderr", "text": ["warn\n"]},
                    {"output_type": "error", "ename": "ValueError", "evalue": "bad", "traceback": ["line1"]},
                    {"output_type": "display_data", "metadata": {}, "data": {"text/html": "<b>x</b>"}}
                ]
            }],
            "metadata": {},
            "nbformat": 4,
            "nbformat_minor": 5
        })
        .to_string(),
    )
    .expect("write ipynb");

    run_cli(args(&["import", &ipynb.to_string_lossy()])).expect("import");
    let notebook = load(dir.join("rich.mars")).expect("load");
    let outputs = &notebook.cells[0].outputs;
    assert_eq!(outputs.len(), 3);
    assert_eq!(notebook.cells[0].execution_count, Some(3));

    let _ = std::fs::remove_dir_all(&dir);
}
