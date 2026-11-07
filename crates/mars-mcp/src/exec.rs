//! `tools/call` handlers for execution and kernel operations.

use crate::handlers::notebook_summary;
use crate::session::MarsSession;
use mars_core::{CellType, ExecutionStatus, Output};
use mars_document::{Cell, Notebook};
use mars_protocol::{KernelRequest, KernelResponse, RequestEnvelope};
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
struct ExecArgs {
    path: String,
    index: Option<usize>,
    source: Option<String>,
    language: Option<String>,
    timeout_ms: Option<u64>,
}

#[derive(Deserialize)]
struct RunArgs {
    path: String,
    timeout_ms: Option<u64>,
}

#[derive(Deserialize)]
struct DiagnoseArgs {
    path: String,
    index: Option<usize>,
    source: Option<String>,
    language: Option<String>,
}

#[derive(Deserialize)]
struct KernelArgs {
    path: Option<String>,
    language: Option<String>,
}

fn parse<T: for<'de> Deserialize<'de>>(args: &Value) -> Result<T, String> {
    serde_json::from_value(args.clone()).map_err(|error| format!("invalid arguments: {error}"))
}

fn language_for(
    session: &MarsSession,
    explicit: Option<String>,
    path: Option<&Path>,
    fallback: Option<&str>,
) -> String {
    explicit
        .or_else(|| fallback.map(str::to_owned))
        .unwrap_or_else(|| {
            path.and_then(|p| session.notebook(p))
                .map(|n| n.metadata.language.clone())
                .unwrap_or_else(|| session.default_language().to_owned())
        })
}

fn response_json(response: &KernelResponse) -> Value {
    serde_json::to_value(response).unwrap_or_else(|_| json!({ "raw": format!("{response:?}") }))
}

fn execution_failed(outputs: &[Output]) -> bool {
    outputs
        .iter()
        .any(|output| matches!(output, Output::Error { .. }))
}

fn status_for(response: &KernelResponse) -> ExecutionStatus {
    match response {
        KernelResponse::Execution { outputs, .. } => {
            if execution_failed(outputs) {
                ExecutionStatus::Failed
            } else {
                ExecutionStatus::Succeeded
            }
        }
        KernelResponse::Error { .. } => ExecutionStatus::Failed,
        _ => ExecutionStatus::Idle,
    }
}

fn apply_execution(cell: &mut Cell, response: &KernelResponse) -> ExecutionStatus {
    let status = status_for(response);
    if let KernelResponse::Execution { outputs, .. } = response {
        cell.outputs = outputs.clone();
    } else if let KernelResponse::Error { message } = response {
        cell.outputs = vec![Output::Error {
            name: "KernelError".to_owned(),
            message: message.clone(),
            traceback: Vec::new(),
        }];
    }
    if status == ExecutionStatus::Succeeded {
        cell.execution_count = Some(cell.execution_count.unwrap_or(0) + 1);
    }
    status
}

pub fn call_exec_tool(
    session: &mut MarsSession,
    name: &str,
    args: &Value,
) -> Option<Result<Value, String>> {
    let result = match name {
        "mars_cell_execute" => parse::<ExecArgs>(args).and_then(|a| cell_execute(session, a)),
        "mars_notebook_execute" => {
            parse::<RunArgs>(args).and_then(|a| notebook_execute(session, a))
        }
        "mars_cell_diagnose" => parse::<DiagnoseArgs>(args).and_then(|a| cell_diagnose(session, a)),
        "mars_kernel_status" => parse::<KernelArgs>(args).and_then(|a| kernel_status(session, a)),
        "mars_kernel_reset" => parse::<KernelArgs>(args).and_then(|a| kernel_reset(session, a)),
        "mars_kernel_variables" => {
            parse::<KernelArgs>(args).and_then(|a| kernel_variables(session, a))
        }
        _ => return None,
    };
    Some(result)
}

fn cell_language_of(cell: &Cell, notebook: &Notebook) -> String {
    cell.metadata
        .get("language")
        .and_then(Value::as_str)
        .map_or_else(|| notebook.metadata.language.clone(), str::to_owned)
}

/// Resolve `(language, source, cell_index)` for execute/diagnose calls.
fn resolve_source(
    session: &MarsSession,
    path: &Path,
    index: Option<usize>,
    source: Option<String>,
    language: Option<String>,
) -> Result<(String, String, Option<usize>), String> {
    match (index, source) {
        (Some(index), _) => {
            let notebook = session
                .notebook(path)
                .ok_or_else(|| "notebook is not open".to_owned())?;
            let cell = notebook.cell(index).map_err(|error| error.to_string())?;
            let resolved = language
                .unwrap_or_else(|| cell_language_of(cell, notebook))
                .to_owned();
            Ok((resolved, cell.source.clone(), Some(index)))
        }
        (None, Some(source)) => {
            let language = language_for(session, language, Some(path), None);
            Ok((language, source, None))
        }
        (None, None) => Err("provide either index or source".to_owned()),
    }
}

fn cell_execute(session: &mut MarsSession, args: ExecArgs) -> Result<Value, String> {
    let path = PathBuf::from(&args.path);
    session.open_notebook(&path)?;
    let (language, source, index) =
        resolve_source(session, &path, args.index, args.source, args.language)?;
    if index.is_some() {
        let notebook = session.notebook(&path).ok_or("notebook unavailable")?;
        if let Some(index) = index {
            let cell = notebook.cell(index).map_err(|error| error.to_string())?;
            if cell.cell_type != CellType::Code {
                return Err(format!("cell {index} is not a code cell"));
            }
        }
    }

    let kernel = session.kernel_for(Some(&path), &language)?;
    let response = kernel
        .handle(RequestEnvelope::new(KernelRequest::Execute {
            source,
            timeout_millis: args.timeout_ms,
            language: Some(language.clone()),
        }))
        .response;
    let status = match index {
        Some(index) => {
            let notebook = session.notebook_mut(&path)?;
            let cell = notebook
                .cell_mut(index)
                .map_err(|error| error.to_string())?;
            apply_execution(cell, &response)
        }
        None => status_for(&response),
    };
    session.save_notebook(&path)?;
    Ok(json!({
        "path": path,
        "index": index,
        "language": language,
        "status": status,
        "response": response_json(&response),
        "summary": session.notebook(&path).map(notebook_summary),
    }))
}

fn notebook_execute(session: &mut MarsSession, args: RunArgs) -> Result<Value, String> {
    let path = PathBuf::from(&args.path);
    session.open_notebook(&path)?;

    // Snapshot cell descriptors first: kernels and notebooks live in disjoint
    // maps, but borrowing cells across kernel calls would conflict.
    let cells: Vec<(usize, String, Option<String>)> = {
        let notebook = session.notebook(&path).ok_or("notebook unavailable")?;
        notebook
            .cells
            .iter()
            .enumerate()
            .filter(|(_, cell)| cell.cell_type == CellType::Code)
            .map(|(index, cell)| {
                (
                    index,
                    cell.source.clone(),
                    cell.metadata
                        .get("language")
                        .and_then(Value::as_str)
                        .map(str::to_owned),
                )
            })
            .collect()
    };
    let notebook_language = session
        .notebook(&path)
        .map(|n| n.metadata.language.clone())
        .unwrap_or_else(|| session.default_language().to_owned());

    let mut results = Vec::new();
    for (index, source, cell_language) in cells {
        let language = cell_language.unwrap_or_else(|| notebook_language.clone());
        let kernel = session.kernel_for(Some(&path), &language)?;
        let response = kernel
            .handle(RequestEnvelope::new(KernelRequest::Execute {
                source,
                timeout_millis: args.timeout_ms,
                language: Some(language.clone()),
            }))
            .response;
        let status = {
            let notebook = session.notebook_mut(&path)?;
            let cell = notebook
                .cell_mut(index)
                .map_err(|error| error.to_string())?;
            apply_execution(cell, &response)
        };
        results.push(json!({
            "index": index,
            "language": language,
            "status": status,
            "response": response_json(&response),
        }));
    }
    session.save_notebook(&path)?;
    Ok(json!({
        "path": path,
        "cells_executed": results.len(),
        "results": results,
        "summary": session.notebook(&path).map(notebook_summary),
    }))
}

fn cell_diagnose(session: &mut MarsSession, args: DiagnoseArgs) -> Result<Value, String> {
    let path = PathBuf::from(&args.path);
    session.open_notebook(&path)?;
    let (language, source, _) =
        resolve_source(session, &path, args.index, args.source, args.language)?;
    let kernel = session.kernel_for(Some(&path), &language)?;
    let response = kernel
        .handle(RequestEnvelope::new(KernelRequest::Diagnose { source }))
        .response;
    Ok(json!({
        "path": path,
        "language": language,
        "response": response_json(&response),
    }))
}

fn kernel_status(session: &mut MarsSession, args: KernelArgs) -> Result<Value, String> {
    let path = args.path.as_ref().map(PathBuf::from);
    let language = language_for(session, args.language.clone(), path.as_deref(), None);
    let _ = session.kernel_for(path.as_deref(), &language)?;
    let status = session.kernel_status(path.as_deref(), &language);
    Ok(json!({ "path": args.path, "language": language, "status": status }))
}

fn kernel_reset(session: &mut MarsSession, args: KernelArgs) -> Result<Value, String> {
    let path = args.path.as_ref().map(PathBuf::from);
    let language = language_for(session, args.language.clone(), path.as_deref(), None);
    if let Some(path) = path.as_deref() {
        session.open_notebook(path)?;
    }
    let kernel = session.kernel_for(path.as_deref(), &language)?;
    let response = kernel
        .handle(RequestEnvelope::new(KernelRequest::Reset))
        .response;
    Ok(json!({ "response": response_json(&response) }))
}

fn kernel_variables(session: &mut MarsSession, args: KernelArgs) -> Result<Value, String> {
    let path = args.path.as_ref().map(PathBuf::from);
    let language = language_for(session, args.language.clone(), path.as_deref(), None);
    if let Some(path) = path.as_deref() {
        session.open_notebook(path)?;
    }
    let kernel = session.kernel_for(path.as_deref(), &language)?;
    let response = kernel
        .handle(RequestEnvelope::new(KernelRequest::Variables))
        .response;
    Ok(json!({ "response": response_json(&response) }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_execution_records_outputs_and_count() {
        let mut cell = Cell::new(CellType::Code, "1 + 1");
        let response = KernelResponse::Execution {
            outputs: vec![Output::Text {
                text: "2".to_owned(),
            }],
            execution_count: 1,
            duration_millis: 1,
        };
        let status = apply_execution(&mut cell, &response);
        assert_eq!(status, ExecutionStatus::Succeeded);
        assert_eq!(cell.execution_count, Some(1));
        assert_eq!(cell.outputs.len(), 1);
    }

    #[test]
    fn error_outputs_mark_failed() {
        let response = KernelResponse::Execution {
            outputs: vec![Output::Error {
                name: "E".to_owned(),
                message: "bad".to_owned(),
                traceback: Vec::new(),
            }],
            execution_count: 1,
            duration_millis: 1,
        };
        assert_eq!(status_for(&response), ExecutionStatus::Failed);
    }
}
