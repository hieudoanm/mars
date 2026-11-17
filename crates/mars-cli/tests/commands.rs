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

#[test]
fn run_mixed_language_notebook_uses_per_cell_language() {
    if std::process::Command::new("python3")
        .arg("--version")
        .output()
        .is_err()
    {
        return;
    }
    let dir = temp_dir("run-mixed");
    let path = dir.join("mixed.mars");
    let mut notebook = Notebook::new();
    notebook.metadata.language = "rust".to_owned();

    // Rust cell (default language) — exercises the Rust replay session.
    let mut rust_cell = Cell::new(CellType::Code, "let mixed_rust = 1;");
    rust_cell.metadata.clear();
    notebook.add_cell(rust_cell).expect("rust cell");

    // Python cell via per-cell language metadata — exercises the second
    // entry in the per-language session map.
    let mut python_cell = Cell::new(CellType::Code, "mixed_py = 6 * 7\nmixed_py");
    python_cell
        .metadata
        .insert("language".to_owned(), json!("python"));
    notebook.add_cell(python_cell).expect("python cell");

    // Another Rust cell reuses the first session.
    notebook
        .add_cell(Cell::new(CellType::Code, "let _ = mixed_rust;"))
        .expect("rust cell 2");
    write_notebook(&path, &notebook);

    run_cli(args(&[
        "run",
        &path.to_string_lossy(),
        "--no-save",
        "--report",
        "--timeout-ms",
        "120000",
    ]))
    .expect("run mixed-language notebook");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn command_error_paths() {
    let dir = temp_dir("cmd-errors");

    // `new` rejects an empty language before writing.
    assert!(run_cli(args(&["new", &dir.join("x.mars").to_string_lossy(), ""])).is_err());

    // `kernel serve` surfaces option-parse errors without blocking.
    assert!(run_cli(args(&["kernel", "serve", "--definitely-bogus"])).is_err());

    // `run` on a missing notebook errors cleanly.
    assert!(run_cli(args(&["run", &dir.join("missing.mars").to_string_lossy()])).is_err());

    // `import` rejects non-ipynb content.
    let bad_ipynb = dir.join("bad.ipynb");
    std::fs::write(&bad_ipynb, b"not json at all").expect("write");
    assert!(
        run_cli(args(&[
            "import",
            &bad_ipynb.to_string_lossy(),
            &dir.join("out.mars").to_string_lossy(),
        ]))
        .is_err()
    );

    // `inspect` on a missing file errors.
    assert!(
        run_cli(args(&[
            "inspect",
            &dir.join("missing.mars").to_string_lossy()
        ]))
        .is_err()
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn open_loads_and_hands_off_to_system() {
    let dir = temp_dir("open");
    let path = dir.join("open.mars");
    write_notebook(&path, &markdown_only_notebook());
    let result = run_cli(args(&["open", &path.to_string_lossy()]));
    // `open`/`xdg-open` may be absent on minimal CI images; only assert the
    // command reached the system-opener stage.
    if let Err(error) = result {
        assert!(error.contains("failed to open"), "unexpected: {error}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn run_forwards_stdin_to_python_input_prompt() {
    if std::process::Command::new("python3")
        .arg("--version")
        .output()
        .is_err()
    {
        return;
    }
    let dir = temp_dir("run-stdin");
    let path = dir.join("prompt.mars");
    let mut notebook = Notebook::new();
    notebook.metadata.language = "python".to_owned();
    notebook
        .add_cell(Cell::new(
            CellType::Code,
            "name = input('Name? ')\nprint('hello', name)",
        ))
        .expect("cell");
    write_notebook(&path, &notebook);

    // Drive the real binary so the process-stdin pump in run_notebook runs.
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_mars"))
        .args([
            "run",
            &path.to_string_lossy(),
            "--no-save",
            "--timeout-ms",
            "30000",
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("spawn mars");
    use std::io::Write;
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(b"Alice\n")
        .expect("write stdin");
    drop(child.stdin.take());
    let output = child.wait_with_output().expect("wait");
    assert!(output.status.success(), "stderr suppressed; status failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("hello Alice"), "stdout: {stdout}");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn run_prints_rich_outputs_from_python_cells() {
    if std::process::Command::new("python3")
        .arg("--version")
        .output()
        .is_err()
    {
        return;
    }
    let dir = temp_dir("run-rich");
    let path = dir.join("rich.mars");
    let mut notebook = Notebook::new();
    notebook.metadata.language = "python".to_owned();
    notebook
        .add_cell(Cell::new(
            CellType::Code,
            "display({\"text/html\": \"<b>bold</b>\", \"image/png\": \"aGk=\"})",
        ))
        .expect("cell");
    write_notebook(&path, &notebook);

    let result = run_cli(args(&[
        "run",
        &path.to_string_lossy(),
        "--no-save",
        "--timeout-ms",
        "30000",
    ]));
    assert!(result.is_ok(), "run failed: {:?}", result.err());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn export_supports_markdown_and_html_formats() {
    let dir = temp_dir("export-formats");
    let path = dir.join("book.mars");
    write_notebook(&path, &markdown_only_notebook());

    for format in ["md", "html"] {
        let out = dir.join(format!("book.{format}"));
        run_cli(args(&[
            "export",
            &path.to_string_lossy(),
            format,
            &out.to_string_lossy(),
        ]))
        .unwrap_or_else(|error| panic!("export {format}: {error}"));
        assert!(out.exists(), "{format} output missing");
    }
    assert!(
        run_cli(args(&[
            "export",
            &path.to_string_lossy(),
            "bogus",
            &dir.join("book.bogus").to_string_lossy(),
        ]))
        .is_err()
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn run_report_flag_prints_summary() {
    let dir = temp_dir("run-report");
    let path = dir.join("report.mars");
    write_notebook(&path, &markdown_only_notebook());
    run_cli(args(&[
        "run",
        &path.to_string_lossy(),
        "--no-save",
        "--report",
    ]))
    .expect("run with report");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn tools_list_and_call_against_real_mcp_server() {
    let server = format!("{} mcp serve", env!("CARGO_BIN_EXE_mars"));
    run_cli(args(&["tools", "list", "--server", &server])).expect("tools list");

    let dir = temp_dir("tools-call");
    let nb = dir.join("tool.mars");
    let call_args = json!({ "path": nb.to_string_lossy(), "language": "rust" });
    run_cli(args(&[
        "tools",
        "call",
        "mars_notebook_new",
        &call_args.to_string(),
        "--server",
        &server,
    ]))
    .expect("tools call");
    assert!(nb.exists(), "notebook not created by tool call");

    // Error surfaces: unknown tool reports isError -> Err.
    assert!(
        run_cli(args(&[
            "tools",
            "call",
            "mars_notebook_open",
            "{\"path\":\"/nonexistent/nope.mars\"}",
            "--server",
            &server,
        ]))
        .is_err()
    );
    // Bad JSON arguments and extra positionals are rejected.
    assert!(
        run_cli(args(&[
            "tools",
            "call",
            "mars_notebook_new",
            "not-json",
            "--server",
            &server,
        ]))
        .is_err()
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn new_with_template_and_example() {
    let dir = temp_dir("new-template");

    let template_path = dir.join("py.mars");
    run_cli(args(&[
        "notebook",
        "new",
        &template_path.to_string_lossy(),
        "--template",
        "python",
    ]))
    .expect("new --template python");
    let notebook = load(&template_path).expect("load template");
    assert_eq!(notebook.metadata.language, "python");
    assert!(
        notebook
            .cells
            .iter()
            .any(|cell| cell.cell_type == CellType::Code)
    );

    // Positional language still works and combines with --language flag.
    let flag_path = dir.join("flag.mars");
    run_cli(args(&[
        "notebook",
        "new",
        &flag_path.to_string_lossy(),
        "--language",
        "julia",
    ]))
    .expect("new --language julia");
    assert_eq!(load(&flag_path).expect("load").metadata.language, "julia");

    // Unknown template and conflicting flags are rejected.
    assert!(
        run_cli(args(&[
            "notebook",
            "new",
            &dir.join("bad.mars").to_string_lossy(),
            "--template",
            "nope",
        ]))
        .is_err()
    );
    assert!(
        run_cli(args(&[
            "notebook",
            "new",
            &dir.join("conflict.mars").to_string_lossy(),
            "--template",
            "rust",
            "--example",
            "rust/01-getting-started/getting-started",
        ]))
        .is_err()
    );

    // Examples resolve from the repo checkout (tests run inside the repo).
    let example_path = dir.join("example.mars");
    run_cli(args(&[
        "notebook",
        "new",
        &example_path.to_string_lossy(),
        "--example",
        "rust/01-getting-started/getting-started",
    ]))
    .expect("new --example");
    let example = load(&example_path).expect("load example");
    assert!(!example.cells.is_empty());

    assert!(run_cli(args(&["notebook", "templates"])).is_ok());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn inspect_json_flag_is_accepted() {
    let dir = temp_dir("inspect-json");
    let path = dir.join("demo.mars");
    write_notebook(&path, &markdown_only_notebook());
    run_cli(args(&[
        "notebook",
        "inspect",
        &path.to_string_lossy(),
        "--json",
    ]))
    .expect("inspect --json");
    run_cli(args(&["notebook", "inspect", &path.to_string_lossy()])).expect("inspect");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn doctor_runs_and_reports() {
    run_cli(args(&["notebook", "doctor"])).expect("doctor without notebook");
    let dir = temp_dir("doctor");
    let path = dir.join("demo.mars");
    write_notebook(&path, &markdown_only_notebook());
    // Markdown-only notebook needs no external runtime -> Ok.
    run_cli(args(&["notebook", "doctor", &path.to_string_lossy()])).expect("doctor notebook");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn templates_subcommand_lists_templates() {
    run_cli(args(&["notebook", "templates"])).expect("notebook templates");
}

#[test]
fn export_supports_slides_pdf_and_markdown_aliases() {
    let dir = temp_dir("export-formats-2");
    let path = dir.join("book.mars");
    write_notebook(&path, &markdown_only_notebook());
    let p = path.to_string_lossy().to_string();

    for (format, ext) in [("slides", "html"), ("pdf", "pdf"), ("markdown", "md")] {
        let out = dir.join(format!("out.{ext}"));
        run_cli(args(&["export", &p, format, &out.to_string_lossy()]))
            .unwrap_or_else(|e| panic!("export {format}: {e}"));
        assert!(out.exists(), "{format} output missing");
    }

    // --page-size flag parsing, including an invalid size.
    let pdf = dir.join("sized.pdf");
    run_cli(args(&[
        "export",
        &p,
        "pdf",
        "--page-size=a4",
        &pdf.to_string_lossy(),
    ]))
    .expect("export pdf with page size");
    assert!(pdf.exists());
    assert!(
        run_cli(args(&[
            "export",
            &p,
            "pdf",
            "--page-size=bogus",
            &dir.join("bad.pdf").to_string_lossy(),
        ]))
        .is_err()
    );
}

#[test]
fn run_option_parse_errors() {
    let dir = temp_dir("run-opts");
    let path = dir.join("book.mars");
    write_notebook(&path, &markdown_only_notebook());
    let p = path.to_string_lossy().to_string();

    for extra in [
        vec!["--watch-ms", "0"],
        vec!["--watch-ms", "abc"],
        vec!["--watch-ms"],
        vec!["--timeout-ms", "0"],
        vec!["--timeout-ms", "abc"],
        vec!["--timeout-ms"],
        vec!["--runtime", "bogus"],
        vec!["--runtime"],
        vec!["--since"],
        vec!["--definitely-bogus"],
    ] {
        let mut full = vec!["run", p.as_str()];
        full.extend(extra.iter().copied());
        assert!(
            run_cli(args(&full)).is_err(),
            "expected error for {extra:?}"
        );
    }
}

#[test]
fn json_report_flag_is_accepted() {
    let dir = temp_dir("json-report");
    let path = dir.join("book.mars");
    write_notebook(&path, &markdown_only_notebook());
    run_cli(args(&[
        "run",
        &path.to_string_lossy(),
        "--json-report",
        "--no-save",
    ]))
    .expect("run --json-report");
}

#[test]
fn serve_and_mcp_usage_errors() {
    assert!(run_cli(args(&["kernel"])).is_err());
    assert!(run_cli(args(&["kernel", "bogus"])).is_err());
    assert!(run_cli(args(&["kernel", "serve", "--runtime", "bogus"])).is_err());
    assert!(run_cli(args(&["kernel", "serve", "--tcp", "notaport"])).is_err());
    assert!(run_cli(args(&["kernel", "serve", "--bogus"])).is_err());
    assert!(run_cli(args(&["mcp"])).is_err());
    assert!(run_cli(args(&["mcp", "bogus"])).is_err());
    assert!(run_cli(args(&["mcp", "serve", "--bogus"])).is_err());
    assert!(run_cli(args(&["tools"])).is_err());
}

#[test]
fn run_accepts_timeout_offline_locked_and_evcxr_rejection() {
    let dir = temp_dir("run-flags");
    let path = dir.join("code.mars");
    let mut notebook = Notebook::new();
    notebook
        .add_cell(Cell::new(CellType::Code, "let answer = 42;"))
        .expect("cell");
    write_notebook(&path, &notebook);
    let p = path.to_string_lossy().to_string();

    run_cli(args(&[
        "run",
        &p,
        "--timeout-ms",
        "30000",
        "--offline",
        "--locked",
        "--no-save",
    ]))
    .expect("run with flags");

    // The evcxr backend is feature-gated; without it the run must fail
    // honestly rather than silently fall back.
    assert!(run_cli(args(&["run", &p, "--runtime", "evcxr", "--no-save"])).is_err());
}

#[test]
fn doctor_accepts_a_notebook_path() {
    let dir = temp_dir("doctor-nb");
    let path = dir.join("book.mars");
    write_notebook(&path, &markdown_only_notebook());
    // Doctor with a notebook checks only the languages it uses; rust is
    // always available so this succeeds, and a bogus path errors.
    run_cli(args(&["notebook", "doctor", &path.to_string_lossy()])).expect("doctor notebook");
    let missing = dir.join("missing.mars");
    assert!(run_cli(args(&["notebook", "doctor", &missing.to_string_lossy()])).is_err());
}

#[test]
fn export_and_import_use_default_output_paths() {
    let dir = temp_dir("default-paths");
    let mars = dir.join("book.mars");
    write_notebook(&mars, &markdown_only_notebook());

    // export without an explicit output writes next to the input.
    run_cli(args(&["export", &mars.to_string_lossy(), "html"])).expect("default export");
    assert!(dir.join("book.html").exists());

    // import without an output derives a .mars path next to the .ipynb.
    let ipynb = dir.join("nb.ipynb");
    std::fs::write(
        &ipynb,
        r#"{"nbformat":4,"nbformat_minor":5,"metadata":{},"cells":[]}"#,
    )
    .expect("write ipynb");
    run_cli(args(&["import", &ipynb.to_string_lossy()])).expect("default import");
    assert!(dir.join("nb.mars").exists());
}

#[test]
fn mcp_serve_and_tools_error_paths() {
    // Arg-parse failures return before serve_mcp blocks on stdin.
    assert!(run_cli(args(&["mcp", "serve", "--runtime", "bogus"])).is_err());
    assert!(run_cli(args(&["mcp", "serve", "--notebook"])).is_err());
    assert!(run_cli(args(&["mcp", "serve", "--language"])).is_err());
    assert!(run_cli(args(&["tools", "bogus"])).is_err());
    // A --server command that cannot spawn surfaces as an error.
    assert!(
        run_cli(args(&[
            "tools",
            "list",
            "--server",
            "definitely-not-a-real-binary-xyz",
        ]))
        .is_err()
    );
}

#[test]
fn new_example_and_doctor_unknown_language() {
    let dir = temp_dir("example-doctor");
    let path = dir.join("ex.mars");
    // Unknown example name errors honestly.
    assert!(
        run_cli(args(&[
            "new",
            &path.to_string_lossy(),
            "--example",
            "definitely-not-an-example",
        ]))
        .is_err()
    );

    // Doctor on a notebook whose language has no probe reports success
    // (no external runtime required).
    let nb = dir.join("odd.mars");
    let mut notebook = Notebook::new();
    notebook.metadata.language = "cobol".to_owned();
    write_notebook(&nb, &notebook);
    run_cli(args(&["notebook", "doctor", &nb.to_string_lossy()])).expect("doctor cobol");
}

#[test]
fn watch_mode_runs_and_detects_changes() {
    let dir = temp_dir("watch");
    let path = dir.join("watched.mars");
    // Markdown-only notebook: each watch iteration completes quickly.
    let mut notebook = Notebook::new();
    notebook.cells.push(Cell {
        cell_type: CellType::Markdown,
        source: "# watched".to_owned(),
        metadata: serde_json::Map::new(),
        outputs: Vec::new(),
        execution_count: None,
        id: mars_core::CellId::new(),
    });
    write_notebook(&path, &notebook);

    // --watch never returns; run it on a detached thread. The first
    // fingerprint pass plus one detected file change are enough coverage.
    let watch_path = path.clone();
    let handle = std::thread::spawn(move || {
        run_cli(args(&[
            "notebook",
            "run",
            &watch_path.to_string_lossy(),
            "--watch",
            "--watch-ms",
            "20",
            "--no-save",
        ]))
    });
    std::thread::sleep(std::time::Duration::from_millis(800));

    // Touch the file (mtime+size change) so the watch loop iterates again.
    let mut touched = Notebook::new();
    touched.cells.push(Cell {
        cell_type: CellType::Markdown,
        source: "# watched v2".to_owned(),
        metadata: serde_json::Map::new(),
        outputs: Vec::new(),
        execution_count: None,
        id: mars_core::CellId::new(),
    });
    write_notebook(&path, &touched);
    std::thread::sleep(std::time::Duration::from_millis(800));
    drop(handle); // detached: the watch loop keeps running until process exit
}
