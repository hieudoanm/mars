pub mod conversion;
pub mod error;
pub mod export;
pub mod import;

pub use conversion::{convert_mime_bundle, convert_output, escape_html, table_to_html};
pub use error::JupyterError;
pub use export::{export_cell, export_ipynb, export_output};
pub use import::{
    JupyterCell, JupyterNotebook, JupyterOutput, Source, import_ipynb, language_from_metadata,
    metadata_string,
};
pub use mars_document::{Cell, Notebook, NotebookMetadata};

#[cfg(test)]
mod tests {
    use super::*;
    use mars_core::{CellType, Output};
    use serde_json::Value;

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
