use mars_core::{CellType, Output};
use mars_document::{Cell, Notebook};
use mars_jupyter::{export_ipynb, import_ipynb};
use mars_kernel::Kernel;
use mars_protocol::RequestEnvelope;
use mars_renderer::render_notebook;
use mars_runtime::RustSession;
use std::{
    env, fs,
    io::{self, BufRead, Write},
    path::Path,
    process::{Command, ExitCode},
    time::Duration,
};

fn main() -> ExitCode {
    match run(env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("mars: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    let Some(command) = args.first().map(String::as_str) else {
        return Err(usage());
    };

    match command {
        "new" => {
            let path = args.get(1).ok_or_else(usage)?;
            let mut notebook = Notebook::new();
            notebook
                .add_cell(Cell::new(CellType::Markdown, "# New Mars notebook"))
                .map_err(|error| error.to_string())?;
            notebook
                .save_atomic(path)
                .map_err(|error| error.to_string())?;
            println!("Created {path}");
            Ok(())
        }
        "inspect" => {
            let path = args.get(1).ok_or_else(usage)?;
            let notebook = Notebook::load(path).map_err(|error| error.to_string())?;
            println!("format_version: {}", notebook.format_version);
            println!("language: {}", notebook.metadata.language);
            println!("cells: {}", notebook.cells.len());
            Ok(())
        }
        "open" => {
            let path = args.get(1).ok_or_else(usage)?;
            Notebook::load(path).map_err(|error| error.to_string())?;
            open_with_system(path)?;
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
            Some("serve") => serve_kernel(),
            _ => {
                Err("usage: mars kernel list | mars kernel restart | mars kernel serve".to_owned())
            }
        },
        "run" => {
            let path = args.get(1).ok_or_else(usage)?;
            let options = parse_run_options(&args[2..])?;
            let mut notebook = Notebook::load(path).map_err(|error| error.to_string())?;
            if options.deterministic
                && !notebook.metadata.dependencies.is_empty()
                && notebook.metadata.cargo_lock.is_none()
            {
                return Err("--deterministic requires metadata.cargo_lock".to_owned());
            }
            let mut session = RustSession::with_configuration(
                notebook.metadata.dependencies.clone(),
                notebook.metadata.cargo_lock.clone(),
                options.offline,
                options.locked,
            )
            .map_err(|error| error.to_string())?;
            for index in 0..notebook.cells.len() {
                if notebook.cells[index].cell_type != CellType::Code {
                    continue;
                }
                let source = notebook.cells[index].source.clone();
                println!("Running cell {}...", index + 1);
                let result = session
                    .execute_with_timeout(&source, options.timeout)
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
                        notebook
                            .save_atomic(path)
                            .map_err(|error| error.to_string())?;
                    }
                    return Err(format!("execution failed in cell {}", index + 1));
                }
                if !notebook.metadata.dependencies.is_empty()
                    && notebook.metadata.cargo_lock.is_none()
                {
                    notebook.metadata.cargo_lock =
                        session.cargo_lock().map_err(|error| error.to_string())?;
                }
            }
            if options.save {
                notebook
                    .save_atomic(path)
                    .map_err(|error| error.to_string())?;
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
        "import" => {
            let input = args.get(1).ok_or_else(usage)?;
            let output = args.get(2).map_or_else(
                || default_mars_path(input),
                |path| Path::new(path).to_path_buf(),
            );
            let content = fs::read_to_string(input).map_err(|error| error.to_string())?;
            let notebook = import_ipynb(&content).map_err(|error| error.to_string())?;
            notebook
                .save_atomic(&output)
                .map_err(|error| error.to_string())?;
            println!("Imported {} -> {}", input, output.display());
            Ok(())
        }
        "export" => {
            let input = args.get(1).ok_or_else(usage)?;
            let format = args.get(2).map(String::as_str).unwrap_or("ipynb");
            let output = args.get(3).map_or_else(
                || default_export_path(input, format),
                |path| Path::new(path).to_path_buf(),
            );
            let notebook = Notebook::load(input).map_err(|error| error.to_string())?;
            let content = match format {
                "ipynb" => export_ipynb(&notebook).map_err(|error| error.to_string())?,
                "html" => render_notebook(&notebook),
                _ => return Err(format!("unsupported export format: {format}")),
            };
            fs::write(&output, format!("{content}\n")).map_err(|error| error.to_string())?;
            println!("Exported {} -> {}", input, output.display());
            Ok(())
        }
        _ => Err(usage()),
    }
}

fn usage() -> String {
    "usage: mars new <file.mars> | mars open <file.mars> | mars inspect <file.mars> | mars run <file.mars> [--timeout-ms <milliseconds>] [--offline] [--locked] [--deterministic] [--no-save] [--report] | mars import <file.ipynb> [file.mars] | mars export <file.mars> [ipynb|html] [output] | mars kernel [list|restart|serve]".to_owned()
}

struct RunOptions {
    timeout: Option<Duration>,
    offline: bool,
    locked: bool,
    deterministic: bool,
    save: bool,
    report: bool,
}

fn parse_run_options(args: &[String]) -> Result<RunOptions, String> {
    let mut options = RunOptions {
        timeout: None,
        offline: false,
        locked: false,
        deterministic: false,
        save: true,
        report: false,
    };
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--offline" => options.offline = true,
            "--locked" => options.locked = true,
            "--deterministic" => {
                options.deterministic = true;
                options.locked = true;
            }
            "--no-save" => options.save = false,
            "--report" => options.report = true,
            "--timeout-ms" => {
                let value = args.get(index + 1).ok_or("missing timeout value")?;
                let milliseconds = value
                    .parse::<u64>()
                    .map_err(|_| "timeout must be an integer")?;
                if milliseconds == 0 {
                    return Err("timeout must be greater than zero".to_owned());
                }
                options.timeout = Some(Duration::from_millis(milliseconds));
                index += 1;
            }
            other => return Err(format!("unknown run option: {other}")),
        }
        index += 1;
    }
    Ok(options)
}

fn open_with_system(path: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let mut command = Command::new("open");
    #[cfg(target_os = "linux")]
    let mut command = Command::new("xdg-open");
    #[cfg(target_os = "windows")]
    let mut command = {
        let mut command = Command::new("cmd");
        command.arg("/C").arg("start");
        command
    };
    command
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("failed to open {path}: {error}"))
}

fn serve_kernel() -> Result<(), String> {
    let stdin = io::stdin();
    let mut stdout = io::BufWriter::new(io::stdout().lock());
    let mut kernel = Kernel::new().map_err(|error| error.to_string())?;
    for line in stdin.lock().lines() {
        let line = line.map_err(|error| error.to_string())?;
        if line.trim().is_empty() {
            continue;
        }
        let request: RequestEnvelope =
            serde_json::from_str(&line).map_err(|error| format!("invalid request: {error}"))?;
        let response = kernel.handle(request);
        serde_json::to_writer(&mut stdout, &response).map_err(|error| error.to_string())?;
        stdout.write_all(b"\n").map_err(|error| error.to_string())?;
        stdout.flush().map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn default_mars_path(input: &str) -> std::path::PathBuf {
    Path::new(input).with_extension("mars")
}

fn default_ipynb_path(input: &str) -> std::path::PathBuf {
    Path::new(input).with_extension("ipynb")
}

fn default_export_path(input: &str, format: &str) -> std::path::PathBuf {
    match format {
        "html" => Path::new(input).with_extension("html"),
        _ => default_ipynb_path(input),
    }
}

fn print_execution_result(index: usize, result: &mars_runtime::ExecutionResult) {
    let failed = result
        .outputs
        .iter()
        .any(|output| matches!(output, Output::Error { .. }));
    let outcome = if result.interrupted {
        "interrupted"
    } else if failed {
        "failed"
    } else {
        "succeeded"
    };
    println!(
        "Cell {} {} in {} ms (execution {}, {} output{})",
        index + 1,
        outcome,
        result.duration.as_millis(),
        result.execution_count,
        result.outputs.len(),
        if result.outputs.len() == 1 { "" } else { "s" }
    );
    for output in &result.outputs {
        match output {
            Output::Text { text } => {
                println!("  stdout:");
                print_indented(text);
            }
            Output::Error { name, message, .. } => {
                eprintln!("  {name}:");
                print_indented_error(message);
            }
            Output::Html { .. } => println!("  html output"),
            Output::Image { mime_type, .. } => println!("  image output ({mime_type})"),
            Output::Table { columns, rows } => println!(
                "  table output ({} columns, {} rows)",
                columns.len(),
                rows.len()
            ),
            Output::Latex { .. } => println!("  LaTeX output"),
            Output::Chart { .. } => println!("  chart output"),
            Output::MimeBundle { data, .. } => {
                println!("  MIME bundle output ({} types)", data.len())
            }
        }
    }
}

fn print_indented(text: &str) {
    for line in text.lines() {
        println!("    {line}");
    }
}

fn print_indented_error(text: &str) {
    for line in text.lines() {
        eprintln!("    {line}");
    }
}
