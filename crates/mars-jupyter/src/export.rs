use crate::conversion::table_to_html;
use crate::error::JupyterError;
use mars_core::{CellType, Output};
use mars_document::{Cell, Notebook};
use serde::Serialize;
use serde_json::{Map, Value};

#[derive(Debug, Serialize)]
pub struct ExportedNotebook {
    pub cells: Vec<ExportedCell>,
    pub metadata: Map<String, Value>,
    pub nbformat: u32,
    pub nbformat_minor: u32,
}

#[derive(Debug, Serialize)]
pub struct ExportedCell {
    pub cell_type: String,
    pub source: String,
    pub metadata: Map<String, Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Map<String, Value>>,
    pub outputs: Vec<ExportedOutput>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "output_type")]
pub enum ExportedOutput {
    #[serde(rename = "stream")]
    Stream { name: String, text: String },
    #[serde(rename = "error")]
    Error {
        ename: String,
        evalue: String,
        traceback: Vec<String>,
    },
    #[serde(rename = "display_data")]
    DisplayData {
        data: Map<String, Value>,
        metadata: Map<String, Value>,
    },
}

/// Maps a Project Mars language id to a Jupyter kernelspec object, when a
/// conventional kernel name exists.
fn kernelspec_for_language(language: &str) -> Option<Value> {
    let (name, display_name) = match language {
        "python" => ("python3", "Python 3"),
        "r" => ("ir", "R"),
        "rust" => ("rust", "Rust"),
        "julia" => ("julia-1.10", "Julia 1.10"),
        _ => return None,
    };
    let mut spec = Map::new();
    spec.insert(
        "display_name".to_owned(),
        Value::String(display_name.to_owned()),
    );
    spec.insert("language".to_owned(), Value::String(language.to_owned()));
    spec.insert("name".to_owned(), Value::String(name.to_owned()));
    Some(Value::Object(spec))
}

pub fn export_ipynb(notebook: &Notebook) -> Result<String, JupyterError> {
    mars_document::validate(notebook).map_err(JupyterError::Document)?;
    let mut metadata = notebook
        .metadata
        .extra
        .get("jupyter")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    if !metadata.contains_key("kernelspec") {
        if let Some(spec) = kernelspec_for_language(&notebook.metadata.language) {
            metadata.insert("kernelspec".to_owned(), spec);
        }
    }
    if !metadata.contains_key("language_info") {
        let mut language_info = Map::new();
        language_info.insert(
            "name".to_owned(),
            Value::String(notebook.metadata.language.clone()),
        );
        metadata.insert("language_info".to_owned(), Value::Object(language_info));
    }
    let cells = notebook
        .cells
        .iter()
        .map(export_cell)
        .collect::<Result<Vec<_>, JupyterError>>()?;
    let exported = ExportedNotebook {
        cells,
        metadata,
        nbformat: 4,
        nbformat_minor: 5,
    };
    serde_json::to_string_pretty(&exported).map_err(JupyterError::Serialization)
}

pub fn export_cell(cell: &Cell) -> Result<ExportedCell, JupyterError> {
    let outputs = cell
        .outputs
        .iter()
        .map(export_output)
        .collect::<Result<Vec<_>, JupyterError>>()?;
    Ok(ExportedCell {
        cell_type: match cell.cell_type {
            CellType::Code => "code",
            CellType::Markdown => "markdown",
            CellType::Raw => "raw",
        }
        .to_owned(),
        source: cell.source.clone(),
        metadata: cell.metadata.clone(),
        execution_count: cell.execution_count,
        attachments: jupyter_attachments(cell),
        outputs,
    })
}

/// Maps `metadata.attachments` (`name` → `data:` URL) onto the Jupyter
/// cell-level `attachments` field (`name` → `{mime: base64}`).
fn jupyter_attachments(cell: &Cell) -> Option<Map<String, Value>> {
    let attachments = cell.metadata.get("attachments")?.as_object()?;
    let mut mapped = Map::new();
    for (name, value) in attachments {
        let Some(data_url) = value.as_str() else {
            continue;
        };
        let Some((mime, payload)) = data_url.split_once(',') else {
            continue;
        };
        let mime = mime.strip_prefix("data:").unwrap_or(mime);
        let mime = mime.split(';').next().unwrap_or(mime);
        let mut entry = Map::new();
        entry.insert(mime.to_owned(), Value::String(payload.to_owned()));
        mapped.insert(name.clone(), Value::Object(entry));
    }
    if mapped.is_empty() {
        None
    } else {
        Some(mapped)
    }
}

pub fn export_output(output: &Output) -> Result<ExportedOutput, JupyterError> {
    match output {
        Output::Text { text } => Ok(ExportedOutput::Stream {
            name: "stdout".to_owned(),
            text: text.clone(),
        }),
        Output::Error {
            name,
            message,
            traceback,
        } => Ok(ExportedOutput::Error {
            ename: name.clone(),
            evalue: message.clone(),
            traceback: traceback.clone(),
        }),
        Output::Html { html } => Ok(ExportedOutput::DisplayData {
            data: Map::from_iter([("text/html".to_owned(), Value::String(html.clone()))]),
            metadata: Map::new(),
        }),
        Output::Image { mime_type, data } => Ok(ExportedOutput::DisplayData {
            data: Map::from_iter([(mime_type.clone(), Value::String(data.clone()))]),
            metadata: Map::new(),
        }),
        Output::Table { columns, rows } => Ok(ExportedOutput::DisplayData {
            data: Map::from_iter([(
                "text/html".to_owned(),
                Value::String(table_to_html(columns, rows)),
            )]),
            metadata: Map::new(),
        }),
        Output::Latex { latex } => Ok(ExportedOutput::DisplayData {
            data: Map::from_iter([("text/latex".to_owned(), Value::String(latex.clone()))]),
            metadata: Map::new(),
        }),
        Output::Chart { spec } => Ok(ExportedOutput::DisplayData {
            data: Map::from_iter([("application/vnd.vega-lite+json".to_owned(), spec.clone())]),
            metadata: Map::new(),
        }),
        Output::MimeBundle { data, metadata } => Ok(ExportedOutput::DisplayData {
            data: data.clone(),
            metadata: metadata.clone(),
        }),
    }
}
