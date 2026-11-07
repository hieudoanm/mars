use crate::options::RunOptions;
use crate::output::print_execution_result;
use mars_core::{CellType, Output};
use mars_document::{Cell, Notebook, load, save_atomic};
use mars_jupyter::{export_ipynb, import_ipynb};
use mars_kernel::create_runtime;
use mars_renderer::{render_markdown_notebook, render_notebook, render_pdf_notebook};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

pub fn run_cli(args: Vec<String>) -> Result<(), String> {
    let Some(command) = args.first().map(String::as_str) else {
        return Err(crate::util::usage());
    };

    match command {
        "new" => {
            let path = args.get(1).ok_or_else(crate::util::usage)?;
            let mut notebook = Notebook::new();
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
                let options = crate::options::parse_serve_options(&args[2..])?;
                if let Some(port) = options.tcp {
                    crate::kernel::serve_kernel_tcp(
                        port,
                        options.tcp_host,
                        options.backend,
                        options.notebook,
                    )
                } else {
                    crate::kernel::serve_kernel(options.backend, options.notebook)
                }
            }
            _ => {
                Err("usage: mars kernel list | mars kernel restart | mars kernel serve".to_owned())
            }
        },
        "run" => {
            let path = args.get(1).ok_or_else(crate::util::usage)?;
            let options = crate::options::parse_run_options(&args[2..])?;
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

fn run_notebook(path: &str, options: &RunOptions) -> Result<(), String> {
    let mut notebook = load(path).map_err(|error| error.to_string())?;
    if options.deterministic
        && !notebook.metadata.dependencies.is_empty()
        && notebook.metadata.cargo_lock.is_none()
    {
        return Err("--deterministic requires metadata.cargo_lock".to_owned());
    }
    let mut session = create_runtime(
        options.runtime,
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
        if !notebook.metadata.dependencies.is_empty() && notebook.metadata.cargo_lock.is_none() {
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
