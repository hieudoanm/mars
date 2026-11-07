use crate::output::print_execution_result;
use mars::options::RunOptions;
use mars_core::{CellType, Output};
use mars_document::{Cell, Notebook, load, save_atomic};
use mars_jupyter::{export_ipynb, import_ipynb};
use mars_kernel::create_runtime;
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
        "new" => {
            let path = args.get(1).ok_or_else(crate::util::usage)?;
            let language = args.get(2).map(String::as_str).unwrap_or("rust");
            if language.trim().is_empty() {
                return Err("language must not be empty".to_owned());
            }
            let mut notebook = Notebook::new();
            notebook.metadata.language = language.to_owned();
            notebook
                .add_cell(Cell::new(CellType::Markdown, "# New Mars notebook"))
                .map_err(|error| error.to_string())?;
            save_atomic(&notebook, path).map_err(|error| error.to_string())?;
            println!("Created {path}");
            Ok(())
        }
        "inspect" => {
            let path = args.get(1).ok_or_else(crate::util::usage)?;
            let notebook = load(path).map_err(|error| error.to_string())?;
            println!("format_version: {}", notebook.format_version);
            println!("language: {}", notebook.metadata.language);
            println!("cells: {}", notebook.cells.len());
            Ok(())
        }
        "open" => {
            let path = args.get(1).ok_or_else(crate::util::usage)?;
            load(path).map_err(|error| error.to_string())?;
            crate::util::open_with_system(path)?;
            Ok(())
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
        "tools" => crate::mcp::run_tools(&args[1..]),
        "run" => {
            let path = args.get(1).ok_or_else(crate::util::usage)?;
            let options = mars::options::parse_run_options(&args[2..])?;
            run_notebook(path, &options)
        }
        "import" => {
            let input = args.get(1).ok_or_else(crate::util::usage)?;
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
            let input = args.get(1).ok_or_else(crate::util::usage)?;
            let format = args.get(2).map(String::as_str).unwrap_or("ipynb");
            let mut page_size_str = "a4";
            let mut output: Option<&str> = None;
            for arg in &args[3..] {
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
                "md" | "markdown" => render_markdown_notebook(&notebook).into_bytes(),
                "pdf" => render_pdf_notebook(&notebook, page_size),
                _ => return Err(format!("unsupported export format: {format}")),
            };
            fs::write(&output, output_bytes).map_err(|error| error.to_string())?;
            println!("Exported {} -> {}", input, output.display());
            Ok(())
        }
        _ => Err(crate::util::usage()),
    }
}

fn cell_language(notebook: &Notebook, cell: &Cell) -> String {
    cell.metadata
        .get("language")
        .and_then(|value| value.as_str())
        .filter(|language| !language.trim().is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| notebook.metadata.language.clone())
}

fn run_notebook(path: &str, options: &RunOptions) -> Result<(), String> {
    let mut notebook = load(path).map_err(|error| error.to_string())?;
    if options.deterministic
        && !notebook.metadata.dependencies.is_empty()
        && notebook.metadata.cargo_lock.is_none()
    {
        return Err("--deterministic requires metadata.cargo_lock".to_owned());
    }
    let mut sessions: std::collections::HashMap<String, Box<dyn mars_runtime::RuntimeBackend>> =
        std::collections::HashMap::new();
    // Forwards process stdin to whichever language runtime is executing so
    // interactive prompts (`input()`, `mars_prompt`) work under `mars run`.
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
    for index in 0..notebook.cells.len() {
        if notebook.cells[index].cell_type != CellType::Code {
            continue;
        }
        let language = cell_language(&notebook, &notebook.cells[index]);
        if !sessions.contains_key(&language) {
            let session = create_runtime(
                &language,
                options.runtime,
                notebook.metadata.dependencies.clone(),
                notebook.metadata.cargo_lock.clone(),
                options.offline,
                options.locked,
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
        println!(
            "Completed {} code cells{}",
            notebook
                .cells
                .iter()
                .filter(|cell| cell.cell_type == CellType::Code)
                .count(),
            if options.save {
                " and saved results"
            } else {
                " without saving"
            }
        );
    }
    Ok(())
}
