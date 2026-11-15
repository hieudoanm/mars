use crate::output::print_execution_result;
use mars::options::RunOptions;
use mars_core::{CellType, Output};
use mars_document::{Cell, Notebook, load, save_atomic};
use mars_jupyter::{export_ipynb, import_ipynb};
use mars_kernel::runtime::RuntimeConfig;
use mars_kernel::{create_runtime_with_config, python_options_from_metadata};
use mars_renderer::{render_markdown_notebook, render_notebook, render_pdf_notebook};
use std::fs;
use std::io::BufRead;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

pub fn run_cli(args: Vec<String>) -> Result<(), String> {
    let Some(command) = args.first().map(String::as_str) else {
        return Err(crate::util::usage());
    };

    match command {
        "notebook" => run_notebook_command(&args[1..]),
        "new" | "inspect" | "open" | "run" | "import" | "export" | "diff" => {
            // Backwards compatibility: redirect legacy commands to notebook subcommand
            let mut new_args = vec!["notebook".to_owned()];
            new_args.extend(args);
            run_notebook_command(&new_args[1..])
        }
        "kernel" => match args.get(1).map(String::as_str) {
            Some("list") => {
                println!("mars-kernel: per-process runtime (no background kernels)");
                Ok(())
            }
            Some("restart") => {
                println!("mars-kernel: next execution starts a fresh runtime");
                Ok(())
            }
            Some("serve") => {
                let options = mars::options::parse_serve_options(&args[2..])?;
                if let Some(port) = options.tcp {
                    crate::kernel::serve_kernel_tcp(
                        port,
                        options.tcp_host,
                        options.backend,
                        options.notebook,
                        options.language,
                    )
                } else {
                    crate::kernel::serve_kernel(options.backend, options.notebook, options.language)
                }
            }
            _ => {
                Err("usage: mars kernel list | mars kernel restart | mars kernel serve".to_owned())
            }
        },
        "mcp" => match args.get(1).map(String::as_str) {
            Some("serve") => crate::mcp::serve_mcp(&args[2..]),
            _ => Err("usage: mars mcp serve [--runtime replay|evcxr] [--offline] [--notebook <file.mars>] [--language <lang>]".to_owned()),
        },
        "lab" => crate::lab::run_lab(&args[1..]),
        "tools" => crate::mcp::run_tools(&args[1..]),
        _ => Err(crate::util::usage()),
    }
}

fn run_notebook_command(args: &[String]) -> Result<(), String> {
    let Some(command) = args.first().map(String::as_str) else {
        return Err(crate::util::notebook_usage());
    };

    match command {
        "new" => {
            let path = args.get(1).ok_or_else(crate::util::notebook_usage)?;
            new_notebook(path, &args[2..])
        }
        "diff" => {
            let path = args.get(1).ok_or_else(crate::util::notebook_usage)?;
            diff_notebook(path, args.get(2).map(String::as_str))
        }
        "templates" => {
            println!("{}", crate::templates::list_templates_text());
            Ok(())
        }
        "doctor" => {
            let path = args.get(1).map(String::as_str);
            crate::doctor::run_doctor(path)
        }
        "inspect" => {
            let path = args.get(1).ok_or_else(crate::util::notebook_usage)?;
            let json = args[2..].iter().any(|arg| arg == "--json");
            let notebook = load(path).map_err(|error| error.to_string())?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "path": path,
                        "format_version": notebook.format_version,
                        "language": notebook.metadata.language,
                        "title": notebook.metadata.title,
                        "cell_count": notebook.cells.len(),
                        "code_cells": notebook.cells.iter().filter(|cell| cell.cell_type == CellType::Code).count(),
                        "dependencies": notebook.metadata.dependencies,
                    }))
                    .map_err(|error| error.to_string())?
                );
            } else {
                println!("format_version: {}", notebook.format_version);
                println!("language: {}", notebook.metadata.language);
                println!("cells: {}", notebook.cells.len());
            }
            Ok(())
        }
        "open" => {
            let path = args.get(1).ok_or_else(crate::util::notebook_usage)?;
            load(path).map_err(|error| error.to_string())?;
            crate::util::open_with_system(path)?;
            Ok(())
        }
        "run" => {
            let path = args.get(1).ok_or_else(crate::util::notebook_usage)?;
            let options = mars::options::parse_run_options(&args[2..])?;
            if options.watch {
                watch_notebook(path, &options)
            } else {
                run_notebook(path, &options)
            }
        }
        "import" => {
            let input = args.get(1).ok_or_else(crate::util::notebook_usage)?;
            let output = args.get(2).map_or_else(
                || crate::util::default_mars_path(input),
                |path| Path::new(path).to_path_buf(),
            );
            let content = fs::read_to_string(input).map_err(|error| error.to_string())?;
            let notebook = import_ipynb(&content).map_err(|error| error.to_string())?;
            save_atomic(&notebook, &output).map_err(|error| error.to_string())?;
            println!("Imported {} -> {}", input, output.display());
            Ok(())
        }
        "export" => {
            let input = args.get(1).ok_or_else(crate::util::notebook_usage)?;
            let format = args.get(2).map(String::as_str).unwrap_or("ipynb");
            let mut page_size_str = "a4";
            let mut output: Option<&str> = None;
            for arg in args.get(3..).unwrap_or(&[]) {
                if let Some(size) = arg.strip_prefix("--page-size=") {
                    page_size_str = size;
                } else if output.is_none() {
                    output = Some(arg);
                }
            }
            let output = output.map_or_else(
                || crate::util::default_export_path(input, format),
                |path| Path::new(path).to_path_buf(),
            );
            let notebook = load(input).map_err(|error| error.to_string())?;
            let page_size =
                mars_core::PaperSize::parse(page_size_str).map_err(|error| error.to_string())?;
            let output_bytes = match format {
                "ipynb" => export_ipynb(&notebook)
                    .map_err(|error| error.to_string())?
                    .into_bytes(),
                "html" => render_notebook(&notebook).into_bytes(),
                "slides" => mars_renderer::render_slides_notebook(&notebook).into_bytes(),
                "md" | "markdown" => render_markdown_notebook(&notebook).into_bytes(),
                "pdf" => render_pdf_notebook(&notebook, page_size),
                _ => return Err(format!("unsupported export format: {format}")),
            };
            fs::write(&output, output_bytes).map_err(|error| error.to_string())?;
            println!("Exported {} -> {}", input, output.display());
            Ok(())
        }
        _ => Err(crate::util::notebook_usage()),
    }
}

fn new_notebook(path: &str, args: &[String]) -> Result<(), String> {
    let mut template: Option<String> = None;
    let mut example: Option<String> = None;
    let mut language_flag: Option<String> = None;
    let mut positional: Vec<&str> = Vec::new();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--template" => {
                template = Some(
                    args.get(index + 1)
                        .ok_or("missing value for --template")?
                        .clone(),
                );
                index += 2;
            }
            "--example" => {
                example = Some(
                    args.get(index + 1)
                        .ok_or("missing value for --example")?
                        .clone(),
                );
                index += 2;
            }
            "--language" => {
                language_flag = Some(
                    args.get(index + 1)
                        .ok_or("missing value for --language")?
                        .clone(),
                );
                index += 2;
            }
            flag if flag.starts_with('-') => {
                return Err(format!("unknown new option: {flag}"));
            }
            value => {
                positional.push(value);
                index += 1;
            }
        }
    }
    if template.is_some() && example.is_some() {
        return Err("--template and --example are mutually exclusive".to_owned());
    }
    let positional_language = positional.first().copied();
    if positional.len() > 1 {
        return Err(format!("unexpected argument: {}", positional[1]));
    }
    let language = language_flag.or_else(|| positional_language.map(str::to_owned));
    if let Some(language) = &language
        && language.trim().is_empty()
    {
        return Err("language must not be empty".to_owned());
    }

    let mut notebook = if let Some(name) = &example {
        crate::templates::example_notebook(name)?
    } else if let Some(name) = &template {
        crate::templates::notebook_from_template(name)?
    } else {
        let mut notebook = Notebook::new();
        notebook
            .add_cell(Cell::new(CellType::Markdown, "# New Project Mars notebook"))
            .map_err(|error| error.to_string())?;
        notebook
    };
    // An explicit language flag wins over the template/example default.
    if let Some(language) = language {
        notebook.metadata.language = language;
    }
    save_atomic(&notebook, path).map_err(|error| error.to_string())?;
    println!("Created {path}");
    Ok(())
}

/// `mars notebook diff <file.mars> [other]` — cell-aware diff. `other` is
/// either another `.mars` path or a git ref (default `HEAD`).
fn diff_notebook(path: &str, other: Option<&str>) -> Result<(), String> {
    let current = load(path).map_err(|error| error.to_string())?;
    let (label, baseline) = match other {
        Some(other) if other.ends_with(".mars") || Path::new(other).exists() => {
            let notebook = load(other).map_err(|error| error.to_string())?;
            (other.to_owned(), notebook)
        }
        Some(git_ref) => {
            let notebook = crate::changes::baseline_notebook(path, git_ref)?;
            (format!("{git_ref}:{path}"), notebook)
        }
        None => {
            let notebook = crate::changes::baseline_notebook(path, "HEAD")?;
            (format!("HEAD:{path}"), notebook)
        }
    };
    let changes = crate::changes::diff_cells(&current, &baseline);
    let mut counts = (0usize, 0usize, 0usize);
    for (change, cell) in &changes {
        let kind = match cell.cell_type {
            CellType::Markdown => "markdown",
            CellType::Code => "code",
            CellType::Raw => "raw",
        };
        match change {
            crate::changes::CellChange::Added => {
                counts.0 += 1;
                println!("+ cell {} ({kind}) added", cell.id);
            }
            crate::changes::CellChange::Removed => {
                counts.1 += 1;
                println!("- cell {} ({kind}) removed", cell.id);
            }
            crate::changes::CellChange::Modified => {
                counts.2 += 1;
                println!("~ cell {} ({kind}) modified", cell.id);
                let old = baseline
                    .cells
                    .iter()
                    .find(|old| old.id == cell.id)
                    .map(|old| old.source.as_str())
                    .unwrap_or_default();
                print!("{}", crate::changes::source_diff(old, &cell.source));
            }
            crate::changes::CellChange::Unchanged => {}
        }
    }
    println!(
        "\n{label}: {} added, {} removed, {} modified",
        counts.0, counts.1, counts.2
    );
    Ok(())
}

fn cell_language(notebook: &Notebook, cell: &Cell) -> String {
    cell.metadata
        .get("language")
        .and_then(|value| value.as_str())
        .filter(|language| !language.trim().is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| notebook.metadata.language.clone())
}

/// `--watch`: run the notebook, then re-run whenever the file changes.
/// Polls mtime+size every `watch_ms`; execution errors are reported but do
/// not stop the watch loop.
fn watch_notebook(path: &str, options: &mars::options::RunOptions) -> Result<(), String> {
    use std::time::{Duration, SystemTime};

    fn fingerprint(path: &str) -> Option<(SystemTime, u64)> {
        let metadata = fs::metadata(path).ok()?;
        Some((metadata.modified().ok()?, metadata.len()))
    }

    println!("Watching {path} for changes (Ctrl+C to stop)…");
    loop {
        if let Err(error) = run_notebook(path, options) {
            eprintln!("mars: {error}");
        }
        let mut last = fingerprint(path);
        loop {
            std::thread::sleep(Duration::from_millis(options.watch_ms));
            let current = fingerprint(path);
            if current != last {
                last = current;
                break;
            }
        }
    }
}

fn run_notebook(path: &str, options: &RunOptions) -> Result<(), String> {
    let mut notebook = load(path).map_err(|error| error.to_string())?;
    if options.deterministic
        && !notebook.metadata.dependencies.is_empty()
        && notebook.metadata.cargo_lock.is_none()
    {
        return Err("--deterministic requires metadata.cargo_lock".to_owned());
    }
    // --changed-only / --since: resume execution at the first code cell whose
    // source differs from the git baseline. Earlier cells are skipped because
    // the runtime starts with empty state; cells after the first change still
    // run so downstream state stays consistent.
    let start_index = if options.changed_only {
        let git_ref = options.since.as_deref().unwrap_or("HEAD");
        let baseline = crate::changes::baseline_notebook(path, git_ref)?;
        match crate::changes::first_changed_code_index(&notebook, &baseline) {
            Some(index) => {
                println!(
                    "Changed-only run: skipping {} unchanged cell(s) before index {} (baseline {git_ref})",
                    notebook
                        .cells
                        .iter()
                        .take(index)
                        .filter(|cell| cell.cell_type == CellType::Code)
                        .count(),
                    index + 1
                );
                index
            }
            None => {
                println!("Changed-only run: no code cells changed since {git_ref}; nothing to run");
                if options.save {
                    save_atomic(&notebook, path).map_err(|error| error.to_string())?;
                }
                return Ok(());
            }
        }
    } else {
        0
    };
    let mut sessions: std::collections::HashMap<String, Box<dyn mars_runtime::RuntimeBackend>> =
        std::collections::HashMap::new();
    // Forwards process stdin to whichever language runtime is executing so
    // interactive prompts (`input()`, `mars_prompt`) work under `mars notebook run`.
    // Lines are queued until a writer for the active language exists; stdin
    // EOF is replayed to every writer (present and future) so prompts fail
    // fast with EOF instead of hanging headless runs.
    let stdin_writers = Arc::new(Mutex::new(std::collections::HashMap::<
        String,
        mars_runtime::StdinWriter,
    >::new()));
    let stdin_pending: Arc<Mutex<std::collections::VecDeque<Option<String>>>> =
        Arc::new(Mutex::new(std::collections::VecDeque::new()));
    let stdin_eof = Arc::new(AtomicBool::new(false));
    let active_language = Arc::new(Mutex::new(String::new()));
    let stdin_changed = Arc::new(std::sync::Condvar::new());
    {
        // Reader: process stdin -> pending queue.
        {
            let pending = Arc::clone(&stdin_pending);
            let eof = Arc::clone(&stdin_eof);
            let changed = Arc::clone(&stdin_changed);
            std::thread::spawn(move || {
                let stdin = std::io::stdin();
                let mut lines = stdin.lock().lines();
                loop {
                    match lines.next() {
                        Some(Ok(text)) => {
                            pending
                                .lock()
                                .unwrap_or_else(|poisoned| poisoned.into_inner())
                                .push_back(Some(text));
                        }
                        Some(Err(_)) | None => {
                            pending
                                .lock()
                                .unwrap_or_else(|poisoned| poisoned.into_inner())
                                .push_back(None);
                            eof.store(true, Ordering::Relaxed);
                            break;
                        }
                    }
                    changed.notify_all();
                }
            });
        }
        // Dispatcher: pending queue -> active language's stdin writer.
        enum Action {
            Line(mars_runtime::StdinWriter, String),
            Eof(Vec<mars_runtime::StdinWriter>),
            Wait,
        }
        {
            let writers = Arc::clone(&stdin_writers);
            let pending = Arc::clone(&stdin_pending);
            let active = Arc::clone(&active_language);
            let changed = Arc::clone(&stdin_changed);
            std::thread::spawn(move || {
                loop {
                    let action = {
                        let mut pending = pending
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner());
                        let writers = writers
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner());
                        let language = active
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner())
                            .clone();
                        let writer = writers.get(&language).cloned();
                        match (pending.front(), writer) {
                            (Some(Some(text)), Some(writer)) => {
                                let text = text.clone();
                                pending.pop_front();
                                Action::Line(writer, text)
                            }
                            (Some(None), _) if !writers.is_empty() => {
                                pending.pop_front();
                                Action::Eof(writers.values().cloned().collect())
                            }
                            _ => Action::Wait,
                        }
                    };
                    match action {
                        Action::Line(writer, text) => {
                            // A writer can fail before its driver has spawned;
                            // re-queue so delivery is retried instead of dropped.
                            if writer.write(&text).is_err() {
                                pending
                                    .lock()
                                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                                    .push_front(Some(text));
                            }
                        }
                        Action::Eof(targets) => {
                            let mut failed = false;
                            for writer in &targets {
                                if writer.write_eof().is_err() {
                                    failed = true;
                                }
                            }
                            if failed {
                                pending
                                    .lock()
                                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                                    .push_front(None);
                            }
                        }
                        Action::Wait => {
                            // Poll rather than rely on notify timing: writer
                            // registration happens on the main thread and a
                            // missed wakeup would stall prompt delivery.
                            let pending = pending
                                .lock()
                                .unwrap_or_else(|poisoned| poisoned.into_inner());
                            let _guard = changed
                                .wait_timeout(pending, std::time::Duration::from_millis(200))
                                .unwrap_or_else(|poisoned| poisoned.into_inner());
                        }
                    }
                }
            });
        }
    }
    for index in start_index..notebook.cells.len() {
        if notebook.cells[index].cell_type != CellType::Code {
            continue;
        }
        let language = cell_language(&notebook, &notebook.cells[index]);
        if !sessions.contains_key(&language) {
            let session = create_runtime_with_config(
                &language,
                RuntimeConfig {
                    backend: options.runtime,
                    dependencies: notebook.metadata.dependencies.clone(),
                    cargo_lock: notebook.metadata.cargo_lock.clone(),
                    offline: options.offline,
                    locked: options.locked,
                    python_options: python_options_from_metadata(&notebook.metadata.extra),
                    sql_sources: {
                        let base = Path::new(path)
                            .parent()
                            .map(Path::to_path_buf)
                            .unwrap_or_else(|| Path::new(".").to_path_buf());
                        mars_kernel::sql_sources_from_metadata(&notebook.metadata.extra, &base)
                    },
                },
            )
            .map_err(|error| error.to_string())?;
            if let Some(writer) = session.stdin_writer() {
                let mut guard = stdin_writers
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                guard.insert(language.clone(), writer);
                if stdin_eof.load(Ordering::Relaxed) {
                    if let Some(writer) = guard.get(&language) {
                        let _ = writer.write_eof();
                    }
                }
                drop(guard);
                stdin_changed.notify_all();
            }
            sessions.insert(language.clone(), session);
        }
        *active_language
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = language.clone();
        let session = sessions
            .get_mut(&language)
            .ok_or_else(|| "runtime session unavailable".to_owned())?;
        let source = notebook.cells[index].source.clone();
        println!("Running cell {} ({language})...", index + 1);
        let result = session
            .execute_with_control(&source, options.timeout, Arc::new(AtomicBool::new(false)))
            .map_err(|error| error.to_string())?;
        notebook.cells[index].execution_count = Some(result.execution_count);
        notebook.cells[index].outputs = result.outputs.clone();
        let failed = result
            .outputs
            .iter()
            .any(|output| matches!(output, Output::Error { .. }));
        print_execution_result(index, &result);
        if failed || result.interrupted {
            if options.save {
                save_atomic(&notebook, path).map_err(|error| error.to_string())?;
            }
            return Err(format!("execution failed in cell {}", index + 1));
        }
        if language == "rust"
            && !notebook.metadata.dependencies.is_empty()
            && notebook.metadata.cargo_lock.is_none()
        {
            notebook.metadata.cargo_lock =
                session.cargo_lock().map_err(|error| error.to_string())?;
        }
    }
    if options.save {
        save_atomic(&notebook, path).map_err(|error| error.to_string())?;
    }
    if options.report {
        let code_cells = notebook
            .cells
            .iter()
            .filter(|cell| cell.cell_type == CellType::Code)
            .count();
        if options.json_report {
            let executed = notebook
                .cells
                .iter()
                .skip(start_index)
                .filter(|cell| cell.cell_type == CellType::Code)
                .count();
            println!(
                "{}",
                serde_json::json!({
                    "path": path,
                    "saved": options.save,
                    "code_cells": code_cells,
                    "executed_cells": executed,
                    "skipped_prefix": start_index,
                })
            );
        } else {
            println!(
                "Completed {} code cells{}",
                code_cells,
                if options.save {
                    " and saved results"
                } else {
                    " without saving"
                }
            );
        }
    }
    Ok(())
}
