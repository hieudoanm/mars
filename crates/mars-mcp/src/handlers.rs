//! `tools/call` handlers for notebook and cell operations.

use crate::session::MarsSession;
use mars_core::CellType;
use mars_document::{Cell, Notebook, NotebookMetadata, save_atomic};
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::PathBuf;

#[derive(Deserialize)]
struct NewArgs {
    path: String,
    language: Option<String>,
}

#[derive(Deserialize)]
struct PathArgs {
    path: String,
}

#[derive(Deserialize)]
struct ReadArgs {
    path: String,
    cell_index: Option<usize>,
    #[serde(default = "default_true")]
    include_outputs: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Deserialize)]
struct AddArgs {
    path: String,
    cell_type: String,
    source: Option<String>,
    index: Option<usize>,
    language: Option<String>,
}

#[derive(Deserialize)]
struct UpdateArgs {
    path: String,
    index: usize,
    source: Option<String>,
    cell_type: Option<String>,
}

#[derive(Deserialize)]
struct IndexArgs {
    path: String,
    index: usize,
}

#[derive(Deserialize)]
struct MoveArgs {
    path: String,
    from: usize,
    to: usize,
}

fn parse<T: for<'de> Deserialize<'de>>(args: &Value) -> Result<T, String> {
    serde_json::from_value(args.clone()).map_err(|error| format!("invalid arguments: {error}"))
}

#[doc(hidden)]
pub fn parse_cell_type(value: &str) -> Result<CellType, String> {
    match value {
        "code" => Ok(CellType::Code),
        "markdown" => Ok(CellType::Markdown),
        "raw" => Ok(CellType::Raw),
        other => Err(format!("unknown cell type: {other}")),
    }
}

fn cell_language(cell: &Cell, notebook: &Notebook) -> String {
    cell.metadata
        .get("language")
        .and_then(Value::as_str)
        .map_or_else(|| notebook.metadata.language.clone(), str::to_owned)
}

pub fn notebook_summary(notebook: &Notebook) -> Value {
    json!({
        "format_version": notebook.format_version,
        "metadata": notebook.metadata,
        "cell_count": notebook.cells.len(),
        "cells": notebook.cells.iter().enumerate().map(|(index, cell)| json!({
            "index": index,
            "id": cell.id.to_string(),
            "cell_type": cell.cell_type,
            "language": cell_language(cell, notebook),
            "source_lines": cell.source.lines().count(),
            "has_outputs": !cell.outputs.is_empty(),
        })).collect::<Vec<_>>(),
    })
}

pub fn call_document_tool(
    session: &mut MarsSession,
    name: &str,
    args: &Value,
) -> Option<Result<Value, String>> {
    let result = match name {
        "mars_notebook_new" => parse::<NewArgs>(args).and_then(|a| new_notebook(session, a)),
        "mars_notebook_open" => parse::<PathArgs>(args).and_then(|a| open_notebook(session, a)),
        "mars_notebook_read" => parse::<ReadArgs>(args).and_then(|a| read_notebook(session, a)),
        "mars_notebook_save" => parse::<PathArgs>(args).and_then(|a| save_notebook(session, a)),
        "mars_notebook_inspect" => {
            parse::<PathArgs>(args).and_then(|a| inspect_notebook(session, a))
        }
        "mars_cell_add" => parse::<AddArgs>(args).and_then(|a| add_cell(session, a)),
        "mars_cell_update" => parse::<UpdateArgs>(args).and_then(|a| update_cell(session, a)),
        "mars_cell_delete" => parse::<IndexArgs>(args).and_then(|a| delete_cell(session, a)),
        "mars_cell_move" => parse::<MoveArgs>(args).and_then(|a| move_cell(session, a)),
        _ => return None,
    };
    Some(result)
}

fn new_notebook(session: &mut MarsSession, args: NewArgs) -> Result<Value, String> {
    let path = PathBuf::from(&args.path);
    if path.exists() {
        return Err(format!("file already exists: {}", path.display()));
    }
    let metadata = NotebookMetadata {
        language: args.language.unwrap_or_else(|| "rust".to_owned()),
        ..Default::default()
    };
    let mut notebook = Notebook::new();
    notebook.metadata = metadata;
    let cell = Cell::new(CellType::Markdown, "# New Project Mars notebook");
    notebook.add_cell(cell).map_err(|error| error.to_string())?;
    save_atomic(&notebook, &path).map_err(|error| error.to_string())?;
    session.create_notebook(&path, notebook.clone());
    Ok(json!({ "path": path, "summary": notebook_summary(&notebook) }))
}

fn open_notebook(session: &mut MarsSession, args: PathArgs) -> Result<Value, String> {
    let path = PathBuf::from(&args.path);
    let notebook = session.open_notebook(&path)?;
    Ok(json!({ "path": path, "summary": notebook_summary(notebook) }))
}

fn read_notebook(session: &mut MarsSession, args: ReadArgs) -> Result<Value, String> {
    let path = PathBuf::from(&args.path);
    let notebook = session.open_notebook(&path)?;
    match args.cell_index {
        Some(index) => {
            let cell = notebook.cell(index).map_err(|error| error.to_string())?;
            let mut value = serde_json::to_value(cell).map_err(|e| e.to_string())?;
            if !args.include_outputs {
                if let Some(object) = value.as_object_mut() {
                    object.insert("outputs".to_owned(), json!([]));
                }
            }
            value["language"] = json!(cell_language(cell, notebook));
            Ok(json!({ "path": path, "cell": value }))
        }
        None => {
            let mut value = serde_json::to_value(notebook).map_err(|e| e.to_string())?;
            if !args.include_outputs {
                if let Some(cells) = value.get_mut("cells").and_then(Value::as_array_mut) {
                    for cell in cells {
                        if let Some(object) = cell.as_object_mut() {
                            object.insert("outputs".to_owned(), json!([]));
                        }
                    }
                }
            }
            Ok(json!({ "path": path, "notebook": value }))
        }
    }
}

fn save_notebook(session: &mut MarsSession, args: PathArgs) -> Result<Value, String> {
    let path = PathBuf::from(&args.path);
    let saved = session.save_notebook(&path)?;
    Ok(json!({ "path": path, "saved": saved }))
}

fn inspect_notebook(session: &mut MarsSession, args: PathArgs) -> Result<Value, String> {
    let path = PathBuf::from(&args.path);
    let notebook = session.open_notebook(&path)?;
    Ok(json!({
        "path": path,
        "format_version": notebook.format_version,
        "language": notebook.metadata.language,
        "title": notebook.metadata.title,
        "dependencies": notebook.metadata.dependencies,
        "cell_count": notebook.cells.len(),
        "cells": notebook.cells.iter().map(|cell| json!({
            "cell_type": cell.cell_type,
            "source_lines": cell.source.lines().count(),
        })).collect::<Vec<_>>(),
    }))
}

fn add_cell(session: &mut MarsSession, args: AddArgs) -> Result<Value, String> {
    let path = PathBuf::from(&args.path);
    let cell_type = parse_cell_type(&args.cell_type)?;
    let mut cell = Cell::new(cell_type, args.source.unwrap_or_default());
    if let Some(language) = args.language {
        cell.metadata.insert("language".to_owned(), json!(language));
    }
    let notebook = session.notebook_mut(&path)?;
    let index = args.index.unwrap_or(notebook.cells.len());
    if index > notebook.cells.len() {
        return Err(format!("index out of range: {index}"));
    }
    notebook
        .insert_cell(index, cell)
        .map_err(|e| e.to_string())?;
    let count = notebook.cells.len();
    session.save_notebook(&path)?;
    Ok(json!({ "path": path, "index": index, "cell_count": count }))
}

fn update_cell(session: &mut MarsSession, args: UpdateArgs) -> Result<Value, String> {
    let path = PathBuf::from(&args.path);
    if args.source.is_none() && args.cell_type.is_none() {
        return Err("nothing to update: provide source and/or cell_type".to_owned());
    }
    let notebook = session.notebook_mut(&path)?;
    let cell = notebook
        .cell_mut(args.index)
        .map_err(|error| error.to_string())?;
    if let Some(source) = args.source {
        cell.source = source;
    }
    if let Some(cell_type) = args.cell_type {
        cell.cell_type = parse_cell_type(&cell_type)?;
    }
    let index = args.index;
    session.save_notebook(&path)?;
    Ok(json!({ "path": path, "index": index }))
}

fn delete_cell(session: &mut MarsSession, args: IndexArgs) -> Result<Value, String> {
    let path = PathBuf::from(&args.path);
    let notebook = session.notebook_mut(&path)?;
    let removed = notebook
        .remove_cell(args.index)
        .map_err(|e| e.to_string())?;
    let removed_id = removed.id.to_string();
    let count = notebook.cells.len();
    session.save_notebook(&path)?;
    Ok(json!({ "path": path, "removed_id": removed_id, "cell_count": count }))
}

fn move_cell(session: &mut MarsSession, args: MoveArgs) -> Result<Value, String> {
    let path = PathBuf::from(&args.path);
    let notebook = session.notebook_mut(&path)?;
    notebook
        .move_cell(args.from, args.to)
        .map_err(|e| e.to_string())?;
    let count = notebook.cells.len();
    session.save_notebook(&path)?;
    Ok(json!({ "path": path, "cell_count": count }))
}
