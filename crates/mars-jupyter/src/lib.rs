use mars_core::{CellType, Output};
use mars_document::{Cell, Notebook, NotebookMetadata};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

#[derive(Debug, Deserialize)]
struct JupyterNotebook {
    #[serde(default)]
    cells: Vec<JupyterCell>,
    #[serde(default)]
    metadata: Map<String, Value>,
    #[serde(default = "default_nbformat")]
    nbformat: u32,
    #[serde(default = "default_nbformat_minor")]
    nbformat_minor: u32,
}

fn default_nbformat() -> u32 {
    4
}

fn default_nbformat_minor() -> u32 {
    5
}

#[derive(Debug, Deserialize)]
struct JupyterCell {
    cell_type: String,
    #[serde(default)]
    source: Source,
    #[serde(default)]
    metadata: Map<String, Value>,
    #[serde(default)]
    execution_count: Option<u64>,
    #[serde(default)]
    outputs: Vec<JupyterOutput>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(untagged)]
enum Source {
    #[default]
    Empty,
    Text(String),
    Lines(Vec<String>),
}

impl Source {
    fn into_string(self) -> String {
        match self {
            Self::Empty => String::new(),
            Self::Text(source) => source,
            Self::Lines(lines) => lines.concat(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "output_type")]
enum JupyterOutput {
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

#[derive(Debug, Serialize)]
struct ExportedNotebook {
    cells: Vec<ExportedCell>,
    metadata: Map<String, Value>,
    nbformat: u32,
    nbformat_minor: u32,
}

#[derive(Debug, Serialize)]
struct ExportedCell {
    cell_type: String,
    source: String,
    metadata: Map<String, Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    execution_count: Option<u64>,
    outputs: Vec<ExportedOutput>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "output_type")]
enum ExportedOutput {
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
        extra: Map::new(),
    };
    metadata
        .extra
        .insert("jupyter".to_owned(), Value::Object(notebook.metadata));

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
        format_version: mars_document::CURRENT_FORMAT_VERSION,
        metadata,
        cells,
    };
    result.validate().map_err(JupyterError::Document)?;
    Ok(result)
}

pub fn export_ipynb(notebook: &Notebook) -> Result<String, JupyterError> {
    notebook.validate().map_err(JupyterError::Document)?;
    let metadata = notebook
        .metadata
        .extra
        .get("jupyter")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
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

fn metadata_string(metadata: &Map<String, Value>, key: &str) -> Option<String> {
    metadata.get(key).and_then(Value::as_str).map(str::to_owned)
}

fn language_from_metadata(metadata: &Map<String, Value>) -> String {
    metadata
        .get("language_info")
        .and_then(Value::as_object)
        .and_then(|language_info| language_info.get("name"))
        .and_then(Value::as_str)
        .unwrap_or("rust")
        .to_owned()
}

fn convert_output(output: JupyterOutput) -> Option<Output> {
    match output {
        JupyterOutput::Stream { text, .. } => Some(Output::Text {
            text: text.into_string(),
        }),
        JupyterOutput::Error {
            ename,
            evalue,
            traceback,
        } => Some(Output::Error {
            name: ename,
            message: evalue,
            traceback,
        }),
        JupyterOutput::DisplayData { data, metadata }
        | JupyterOutput::ExecuteResult { data, metadata } => convert_mime_bundle(data, metadata),
    }
}

fn convert_mime_bundle(
    mut data: Map<String, Value>,
    metadata: Map<String, Value>,
) -> Option<Output> {
    if data.len() > 1 || !metadata.is_empty() {
        return Some(Output::MimeBundle { data, metadata });
    }
    if let Some(Value::String(html)) = data.remove("text/html") {
        return Some(Output::Html { html });
    }
    if let Some(Value::String(text)) = data.remove("text/plain") {
        return Some(Output::Text { text });
    }
    if let Some(Value::String(latex)) = data.remove("text/latex") {
        return Some(Output::Latex { latex });
    }
    if let Some(spec) = data
        .remove("application/vnd.vega.v5+json")
        .or_else(|| data.remove("application/vnd.vega-lite.v5+json"))
        .or_else(|| data.remove("application/vnd.vega-lite+json"))
    {
        return Some(Output::Chart { spec });
    }
    for (mime_type, value) in data.iter() {
        if matches!(
            mime_type.as_str(),
            "image/png" | "image/jpeg" | "image/svg+xml"
        ) {
            if let Some(data) = value.as_str() {
                return Some(Output::Image {
                    mime_type: mime_type.clone(),
                    data: data.to_owned(),
                });
            }
        }
    }
    if data.is_empty() {
        None
    } else {
        Some(Output::MimeBundle { data, metadata })
    }
}

fn export_cell(cell: &Cell) -> Result<ExportedCell, JupyterError> {
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
        outputs,
    })
}

fn export_output(output: &Output) -> Result<ExportedOutput, JupyterError> {
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

fn table_to_html(columns: &[String], rows: &[Vec<String>]) -> String {
    let headers = columns
        .iter()
        .map(|column| format!("<th>{}</th>", escape_html(column)))
        .collect::<String>();
    let body = rows
        .iter()
        .map(|row| {
            let cells = row
                .iter()
                .map(|value| format!("<td>{}</td>", escape_html(value)))
                .collect::<String>();
            format!("<tr>{cells}</tr>")
        })
        .collect::<String>();
    format!("<table><thead><tr>{headers}</tr></thead><tbody>{body}</tbody></table>")
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[derive(Debug, thiserror::Error)]
pub enum JupyterError {
    #[error("failed to deserialize Jupyter notebook")]
    Deserialization(#[source] serde_json::Error),
    #[error("failed to serialize Jupyter notebook")]
    Serialization(#[source] serde_json::Error),
    #[error("Jupyter notebook format {0} is not supported")]
    UnsupportedFormat(u32),
    #[error("Jupyter cell type {0} is not supported")]
    UnsupportedCellType(String),
    #[error("Mars output type {0} cannot be exported to Jupyter")]
    UnsupportedOutput(String),
    #[error("invalid Mars notebook")]
    Document(#[source] mars_document::DocumentError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imports_and_exports_basic_notebook() {
        let input = r##"{
          "cells": [
            {"cell_type": "markdown", "metadata": {}, "source": ["# Hi\n"], "outputs": []},
            {"cell_type": "code", "execution_count": 1, "metadata": {}, "source": "println!(\"hi\");", "outputs": [
              {"name": "stdout", "output_type": "stream", "text": ["hi\n"]}
            ]}
          ],
          "metadata": {"language_info": {"name": "rust"}},
          "nbformat": 4,
          "nbformat_minor": 5
        }"##;

        let notebook = import_ipynb(input).expect("imports");
        let exported = export_ipynb(&notebook).expect("exports");
        let value: Value = serde_json::from_str(&exported).expect("valid JSON");

        assert_eq!(value["cells"].as_array().expect("cells").len(), 2);
        assert_eq!(value["metadata"]["language_info"]["name"], "rust");
    }

    #[test]
    fn exports_tables_as_escaped_html() {
        let mut notebook = Notebook::new();
        let mut cell = Cell::new(CellType::Code, "table()");
        cell.outputs.push(Output::Table {
            columns: vec!["Name".to_owned()],
            rows: vec![vec!["<Mars>".to_owned()]],
        });
        notebook.add_cell(cell).expect("unique ID");

        let exported = export_ipynb(&notebook).expect("exports table");
        let value: Value = serde_json::from_str(&exported).expect("valid JSON");
        let html = value["cells"][0]["outputs"][0]["data"]["text/html"]
            .as_str()
            .expect("HTML output");

        assert!(html.contains("<th>Name</th>"));
        assert!(html.contains("&lt;Mars&gt;"));
        assert!(!html.contains("<Mars>"));
    }

    #[test]
    fn preserves_latex_charts_and_unknown_mime_bundles() {
        let input = r##"{
          "cells": [{
            "cell_type": "code",
            "metadata": {},
            "source": "display()",
            "execution_count": 1,
            "outputs": [{
              "output_type": "display_data",
              "metadata": {"source": "test"},
              "data": {
                "text/latex": "E = mc^2",
                "application/vnd.vega-lite+json": {"mark": "line"},
                "application/x-mars-custom": {"value": 7}
              }
            }]
          }],
          "metadata": {},
          "nbformat": 4,
          "nbformat_minor": 5
        }"##;

        let notebook = import_ipynb(input).expect("imports");
        let output = &notebook.cells[0].outputs[0];
        assert!(
            matches!(output, Output::MimeBundle { data, .. } if data.contains_key("text/latex"))
        );

        let exported = export_ipynb(&notebook).expect("exports");
        assert!(exported.contains("text/latex"));
    }

    #[test]
    fn round_trips_rich_fixture() {
        let input = include_str!("../../../tests/fixtures/rich-notebook.ipynb");
        let notebook = import_ipynb(input).expect("imports fixture");
        assert_eq!(notebook.cells.len(), 2);
        assert!(matches!(
            notebook.cells[1].outputs.first(),
            Some(Output::MimeBundle { data, .. }) if data.contains_key("text/latex")
        ));
        let exported = export_ipynb(&notebook).expect("exports fixture");
        assert!(exported.contains("application/vnd.vega-lite+json"));
    }

    #[test]
    fn preserves_metadata_and_all_image_variants_in_fixture() {
        let input = include_str!("../../../tests/fixtures/jupyter-compatibility.ipynb");
        let notebook = import_ipynb(input).expect("imports compatibility fixture");
        assert_eq!(notebook.cells[0].metadata["tags"][0], "compatibility");
        assert_eq!(
            notebook.metadata.extra["jupyter"]["language_info"]["version"],
            "1.85"
        );
        assert!(matches!(
            notebook.cells[1].outputs.first(),
            Some(Output::MimeBundle { data, metadata })
                if data.contains_key("image/png")
                    && data.contains_key("image/jpeg")
                    && data.contains_key("image/svg+xml")
                    && data.contains_key("application/vnd.jupyter.widget-view+json")
                    && data.contains_key("application/x-mars-unknown")
                    && metadata["image/png"]["width"] == 2
        ));
        assert!(matches!(
            notebook.cells[1].outputs.get(1),
            Some(Output::Error { name, traceback, .. })
                if name == "ValueError" && traceback.len() == 1
        ));
        let exported = export_ipynb(&notebook).expect("exports compatibility fixture");
        assert!(exported.contains("application/vnd.jupyter.widget-view+json"));
        assert!(exported.contains("application/x-mars-unknown"));
        assert!(exported.contains("image/jpeg"));
    }

    #[test]
    fn retains_single_mime_display_metadata() {
        let input = r##"{
          "cells": [{
            "cell_type": "code",
            "metadata": {},
            "source": "display()",
            "execution_count": 1,
            "outputs": [{
              "output_type": "display_data",
              "metadata": {"text/html": {"isolated": true}},
              "data": {"text/html": "<p>safe</p>"}
            }]
          }],
          "metadata": {},
          "nbformat": 4,
          "nbformat_minor": 5
        }"##;
        let notebook = import_ipynb(input).expect("imports");
        assert!(matches!(
            notebook.cells[0].outputs.first(),
            Some(Output::MimeBundle { data, metadata })
                if data.contains_key("text/html") && metadata["text/html"]["isolated"] == true
        ));
    }
}
