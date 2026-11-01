use crate::conversion::convert_output;
use crate::error::JupyterError;
use mars_core::{CellType, PaperSize};
use mars_document::{CURRENT_FORMAT_VERSION, Cell, Notebook, NotebookMetadata};
use serde::Deserialize;
use serde_json::{Map, Value};
use std::collections::BTreeMap;

#[derive(Debug, Deserialize)]
pub struct JupyterNotebook {
    #[serde(default)]
    pub cells: Vec<JupyterCell>,
    #[serde(default)]
    pub metadata: Map<String, Value>,
    #[serde(default = "default_nbformat")]
    pub nbformat: u32,
    #[serde(default = "default_nbformat_minor")]
    pub nbformat_minor: u32,
}

fn default_nbformat() -> u32 {
    4
}

fn default_nbformat_minor() -> u32 {
    5
}

#[derive(Debug, Deserialize)]
pub struct JupyterCell {
    pub cell_type: String,
    #[serde(default)]
    pub source: Source,
    #[serde(default)]
    pub metadata: Map<String, Value>,
    #[serde(default)]
    pub execution_count: Option<u64>,
    #[serde(default)]
    pub outputs: Vec<JupyterOutput>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(untagged)]
pub enum Source {
    #[default]
    Empty,
    Text(String),
    Lines(Vec<String>),
}

impl Source {
    pub fn into_string(self) -> String {
        match self {
            Self::Empty => String::new(),
            Self::Text(source) => source,
            Self::Lines(lines) => lines.concat(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "output_type")]
pub enum JupyterOutput {
    #[serde(rename = "stream")]
    Stream {
        #[serde(default)]
        text: Source,
        #[serde(default, rename = "name")]
        _name: Option<String>,
    },
    #[serde(rename = "error")]
    Error {
        #[serde(default)]
        ename: String,
        #[serde(default)]
        evalue: String,
        #[serde(default)]
        traceback: Vec<String>,
    },
    #[serde(rename = "display_data")]
    DisplayData {
        #[serde(default)]
        data: Map<String, Value>,
        #[serde(default)]
        metadata: Map<String, Value>,
    },
    #[serde(rename = "execute_result")]
    ExecuteResult {
        #[serde(default)]
        data: Map<String, Value>,
        #[serde(default)]
        metadata: Map<String, Value>,
    },
}

pub fn metadata_string(metadata: &Map<String, Value>, key: &str) -> Option<String> {
    metadata.get(key).and_then(Value::as_str).map(str::to_owned)
}

/// Normalizes a Jupyter language/kernel name to a Mars language id.
fn normalize_language(name: &str) -> &str {
    match name {
        "python" | "python3" | "python2" => "python",
        "rust" => "rust",
        "julia" => "julia",
        "r" | "R" => "r",
        "javascript" | "js" | "nodejs" => "javascript",
        "typescript" | "ts" => "typescript",
        other => other,
    }
}

pub fn language_from_metadata(metadata: &Map<String, Value>) -> String {
    if let Some(name) = metadata
        .get("language_info")
        .and_then(Value::as_object)
        .and_then(|language_info| language_info.get("name"))
        .and_then(Value::as_str)
    {
        return normalize_language(name).to_owned();
    }
    if let Some(spec) = metadata.get("kernelspec").and_then(Value::as_object) {
        if let Some(language) = spec.get("language").and_then(Value::as_str) {
            return normalize_language(language).to_owned();
        }
        if let Some(name) = spec.get("name").and_then(Value::as_str) {
            return normalize_language(name).to_owned();
        }
    }
    "rust".to_owned()
}

pub fn import_ipynb(input: &str) -> Result<Notebook, JupyterError> {
    let notebook: JupyterNotebook =
        serde_json::from_str(input).map_err(JupyterError::Deserialization)?;
    if notebook.nbformat != 4 {
        return Err(JupyterError::UnsupportedFormat(notebook.nbformat));
    }
    let _minor_version = notebook.nbformat_minor;

    let mut metadata = NotebookMetadata {
        title: metadata_string(&notebook.metadata, "title"),
        language: language_from_metadata(&notebook.metadata),
        dependencies: BTreeMap::new(),
        cargo_lock: None,
        page_size: PaperSize::default(),
        extra: Map::new(),
    };
    metadata
        .extra
        .insert("jupyter".to_owned(), Value::Object(notebook.metadata));

    let language = metadata.language.clone();
    let cells = notebook
        .cells
        .into_iter()
        .map(|cell| {
            let cell_type = match cell.cell_type.as_str() {
                "code" => CellType::Code,
                "markdown" => CellType::Markdown,
                "raw" => CellType::Raw,
                unsupported => {
                    return Err(JupyterError::UnsupportedCellType(unsupported.to_owned()));
                }
            };
            let mut mars_cell = Cell::new(cell_type, cell.source.into_string());
            mars_cell.metadata = cell.metadata;
            // Stamp the notebook language on code cells so per-cell language
            // resolution works without notebook context (desktop, CLI).
            if mars_cell.cell_type == CellType::Code
                && language != "rust"
                && mars_cell
                    .metadata
                    .get("language")
                    .and_then(Value::as_str)
                    .is_none()
            {
                mars_cell
                    .metadata
                    .insert("language".to_owned(), Value::String(language.clone()));
            }
            mars_cell.execution_count = cell.execution_count;
            mars_cell.outputs = cell
                .outputs
                .into_iter()
                .filter_map(convert_output)
                .collect();
            Ok(mars_cell)
        })
        .collect::<Result<Vec<_>, JupyterError>>()?;

    let result = Notebook {
        format_version: CURRENT_FORMAT_VERSION,
        metadata,
        cells,
    };
    mars_document::validate(&result).map_err(JupyterError::Document)?;
    Ok(result)
}
