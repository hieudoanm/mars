use mars_core::{CellType, Output};
use mars_jupyter::{Cell, Notebook, export_ipynb, import_ipynb};
use serde_json::{Value, json};

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
    assert!(matches!(output, Output::MimeBundle { data, .. } if data.contains_key("text/latex")));

    let exported = export_ipynb(&notebook).expect("exports");
    assert!(exported.contains("text/latex"));
}

#[test]
fn round_trips_rich_fixture() {
    let input = include_str!("fixtures/rich-notebook.ipynb");
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
    let input = include_str!("fixtures/jupyter-compatibility.ipynb");
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
fn imports_kernelspec_language_and_stamps_code_cells() {
    let input = r##"{
      "cells": [
        {"cell_type": "code", "execution_count": null, "metadata": {}, "source": "x = 1", "outputs": []},
        {"cell_type": "code", "execution_count": null, "metadata": {"language": "rust"}, "source": "let y = 1;", "outputs": []}
      ],
      "metadata": {"kernelspec": {"display_name": "Python 3", "language": "python", "name": "python3"}},
      "nbformat": 4,
      "nbformat_minor": 5
    }"##;
    let notebook = import_ipynb(input).expect("imports");
    assert_eq!(notebook.metadata.language, "python");
    assert_eq!(notebook.cells[0].metadata["language"], "python");
    // Explicit per-cell language is preserved.
    assert_eq!(notebook.cells[1].metadata["language"], "rust");
}

#[test]
fn exports_kernelspec_for_python_notebooks() {
    let input = r##"{
      "cells": [
        {"cell_type": "code", "execution_count": null, "metadata": {"language": "python"}, "source": "x = 1", "outputs": []}
      ],
      "metadata": {"language_info": {"name": "python"}},
      "nbformat": 4,
      "nbformat_minor": 5
    }"##;
    let notebook = import_ipynb(input).expect("imports");
    let exported = export_ipynb(&notebook).expect("exports");
    let value: Value = serde_json::from_str(&exported).expect("valid JSON");
    assert_eq!(value["metadata"]["kernelspec"]["name"], "python3");
    assert_eq!(value["metadata"]["language_info"]["name"], "python");
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

#[test]
fn converts_jupyter_output_variants() {
    use mars_jupyter::{JupyterOutput, Source, convert_output};
    use serde_json::Map;

    let stream = JupyterOutput::Stream {
        text: Source::Lines(vec!["a\n".to_owned(), "b\n".to_owned()]),
        _name: Some("stderr".to_owned()),
    };
    assert!(matches!(
        convert_output(stream),
        Some(Output::Text { text }) if text == "a\nb\n"
    ));

    let error = JupyterOutput::Error {
        ename: "ValueError".to_owned(),
        evalue: "bad".to_owned(),
        traceback: vec!["t".to_owned()],
    };
    assert!(matches!(
        convert_output(error),
        Some(Output::Error { name, message, traceback })
            if name == "ValueError" && message == "bad" && traceback.len() == 1
    ));

    let mut data = Map::new();
    data.insert("text/plain".to_owned(), Value::String("42".to_owned()));
    let display = JupyterOutput::DisplayData {
        data,
        metadata: Map::new(),
    };
    assert!(matches!(
        convert_output(display),
        Some(Output::Text { text }) if text == "42"
    ));

    let mut chart_data = Map::new();
    chart_data.insert(
        "application/vnd.vega-lite.v5+json".to_owned(),
        json!({"mark": "point"}),
    );
    let result = JupyterOutput::ExecuteResult {
        data: chart_data,
        metadata: Map::new(),
    };
    assert!(matches!(
        convert_output(result),
        Some(Output::Chart { spec }) if spec["mark"] == "point"
    ));
}

#[test]
fn mime_bundle_prefers_typed_outputs_and_preserves_unknowns() {
    use mars_jupyter::convert_mime_bundle;
    use serde_json::Map;

    let bundle = |pairs: &[(&str, Value)]| {
        let mut data = Map::new();
        for (key, value) in pairs {
            data.insert((*key).to_owned(), value.clone());
        }
        data
    };
    let string = |text: &str| Value::String(text.to_owned());

    assert!(matches!(
        convert_mime_bundle(bundle(&[("text/html", string("<p>x</p>"))]), Map::new()),
        Some(Output::Html { html }) if html == "<p>x</p>"
    ));
    assert!(matches!(
        convert_mime_bundle(bundle(&[("text/plain", string("hi"))]), Map::new()),
        Some(Output::Text { text }) if text == "hi"
    ));
    assert!(matches!(
        convert_mime_bundle(bundle(&[("text/latex", string("x^2"))]), Map::new()),
        Some(Output::Latex { latex }) if latex == "x^2"
    ));
    assert!(matches!(
        convert_mime_bundle(
            bundle(&[("application/vnd.vega.v5+json", json!({"a": 1}))]),
            Map::new()
        ),
        Some(Output::Chart { spec }) if spec["a"] == 1
    ));
    assert!(matches!(
        convert_mime_bundle(bundle(&[("image/png", string("aGk="))]), Map::new()),
        Some(Output::Image { mime_type, data }) if mime_type == "image/png" && data == "aGk="
    ));
    assert!(convert_mime_bundle(Map::new(), Map::new()).is_none());

    // Multiple MIME entries stay a bundle.
    let multi = convert_mime_bundle(
        bundle(&[("text/plain", string("x")), ("image/png", string("aGk="))]),
        Map::new(),
    )
    .expect("bundle");
    assert!(matches!(multi, Output::MimeBundle { data, .. } if data.len() == 2));

    // Non-empty metadata preserves the bundle even for a single entry.
    let mut metadata = Map::new();
    metadata.insert("isolated".to_owned(), Value::Bool(true));
    let kept = convert_mime_bundle(bundle(&[("text/html", string("<i>x</i>"))]), metadata)
        .expect("bundle");
    assert!(matches!(kept, Output::MimeBundle { .. }));
}

#[test]
fn table_to_html_and_escape_html_sanitize_content() {
    use mars_jupyter::{escape_html, table_to_html};

    assert_eq!(
        escape_html("<a href=\"x\">&'</a>"),
        "&lt;a href=&quot;x&quot;&gt;&amp;&#39;&lt;/a&gt;"
    );
    let html = table_to_html(
        &["Name".to_owned(), "Score".to_owned()],
        &[vec!["<b>Alice</b>".to_owned(), "10".to_owned()]],
    );
    assert!(html.contains("<th>Name</th><th>Score</th>"));
    assert!(html.contains("<td>&lt;b&gt;Alice&lt;/b&gt;</td><td>10</td>"));
    assert!(!html.contains("<b>Alice</b>"));
}

#[test]
fn language_from_metadata_normalizes_kernel_names() {
    use mars_jupyter::{language_from_metadata, metadata_string};
    use serde_json::Map;

    let metadata = |value: Value| {
        let mut map = Map::new();
        map.insert("language_info".to_owned(), value);
        map
    };
    assert_eq!(
        language_from_metadata(&metadata(json!({"name": "python3"}))),
        "python"
    );
    assert_eq!(language_from_metadata(&metadata(json!({"name": "R"}))), "r");
    assert_eq!(language_from_metadata(&Map::new()), "rust");

    let mut kernelspec = Map::new();
    kernelspec.insert("kernelspec".to_owned(), json!({"language": "javascript"}));
    assert_eq!(language_from_metadata(&kernelspec), "javascript");

    let mut spec_name = Map::new();
    spec_name.insert("kernelspec".to_owned(), json!({"name": "ir"}));
    assert_eq!(language_from_metadata(&spec_name), "ir");

    let mut strings = Map::new();
    strings.insert("title".to_owned(), json!("My Notebook"));
    strings.insert("count".to_owned(), json!(3));
    assert_eq!(
        metadata_string(&strings, "title").as_deref(),
        Some("My Notebook")
    );
    assert!(metadata_string(&strings, "count").is_none());
    assert!(metadata_string(&strings, "missing").is_none());
}

#[test]
fn import_rejects_bad_documents() {
    assert!(import_ipynb("not json").is_err());
    assert!(import_ipynb(r#"{"nbformat": 3, "cells": []}"#).is_err());
    assert!(
        import_ipynb(
            r#"{"nbformat": 4, "nbformat_minor": 5, "cells": [{"cell_type": "widget", "source": ""}]}"#
        )
        .is_err()
    );
}

#[test]
fn export_output_covers_every_variant() {
    use mars_jupyter::{export_cell, export_output};
    use serde_json::Map;

    let text = export_output(&Output::Text {
        text: "hi".to_owned(),
    })
    .expect("text");
    assert!(matches!(
        text,
        mars_jupyter::export::ExportedOutput::Stream { .. }
    ));

    let chart_spec = json!({"mark": "line", "data": {"values": []}});
    let chart = export_output(&Output::Chart {
        spec: chart_spec.clone(),
    })
    .expect("chart");
    let serialized = serde_json::to_value(&chart).expect("json");
    assert_eq!(
        serialized["data"]["application/vnd.vega-lite+json"],
        chart_spec
    );

    let mut bundle_data = Map::new();
    bundle_data.insert("application/x-custom".to_owned(), json!({"v": 1}));
    let mut bundle_meta = Map::new();
    bundle_meta.insert("source".to_owned(), json!("mars"));
    let bundle = export_output(&Output::MimeBundle {
        data: bundle_data.clone(),
        metadata: bundle_meta.clone(),
    })
    .expect("bundle");
    let serialized = serde_json::to_value(&bundle).expect("json");
    assert_eq!(serialized["data"]["application/x-custom"]["v"], 1);
    assert_eq!(serialized["metadata"]["source"], "mars");

    // Raw cells export without outputs.
    let mut notebook = Notebook::new();
    notebook.metadata.language = "cpp".to_owned();
    let mut cell = Cell::new(CellType::Raw, "raw content");
    cell.execution_count = None;
    notebook.add_cell(cell).expect("cell");
    let exported = export_cell(&notebook.cells[0]).expect("cell");
    assert_eq!(exported.cell_type, "raw");
    assert!(exported.outputs.is_empty());

    // Unknown languages get no kernelspec but still export.
    let exported_nb = export_ipynb(&notebook).expect("export");
    let value: Value = serde_json::from_str(&exported_nb).expect("json");
    assert!(value["metadata"].get("kernelspec").is_none());
    assert_eq!(value["metadata"]["language_info"]["name"], "cpp");
}
