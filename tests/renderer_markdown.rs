use mars_core::{CellType, Output};
use mars_document::{Cell, Notebook};
use mars_renderer::render_markdown_notebook;
use serde_json::Map;

#[test]
fn exports_markdown_with_cells_and_outputs() {
    let mut notebook = Notebook::new();
    notebook.metadata.title = Some("Demo notebook".to_owned());
    notebook
        .add_cell(Cell::new(CellType::Markdown, "# Hello\n\n**Mars**"))
        .expect("unique_id");
    let mut cell = Cell::new(CellType::Code, "println!(\"hi\");");
    cell.outputs.push(Output::Text {
        text: "hi\n".to_owned(),
    });
    cell.outputs.push(Output::Latex {
        latex: "E = mc^2".to_owned(),
    });
    cell.outputs.push(Output::Table {
        columns: vec!["name".to_owned(), "value".to_owned()],
        rows: vec![vec!["a|b".to_owned(), "1".to_owned()]],
    });
    notebook.add_cell(cell).expect("unique_id");

    let markdown = render_markdown_notebook(&notebook);

    assert!(markdown.starts_with("# Demo notebook\n\n# Hello"));
    assert!(markdown.contains("**Mars**"));
    assert!(markdown.contains("```rust\nprintln!(\"hi\");\n```"));
    assert!(markdown.contains("```text\nhi\n```"));
    assert!(markdown.contains("$$\nE = mc^2\n$$"));
    assert!(markdown.contains("| name | value |"));
    assert!(markdown.contains("| a\\|b | 1 |"));
}

#[test]
fn avoids_duplicating_an_existing_title_heading() {
    let mut notebook = Notebook::new();
    notebook.metadata.title = Some("Demo notebook".to_owned());
    notebook
        .add_cell(Cell::new(CellType::Markdown, "# Demo notebook\n\nBody"))
        .expect("unique_id");

    let markdown = render_markdown_notebook(&notebook);

    assert_eq!(markdown.matches("# Demo notebook").count(), 1);
    assert!(markdown.ends_with("\n\nBody"));
}

#[test]
fn exports_markdown_images_as_data_uris() {
    let mut notebook = Notebook::new();
    let mut cell = Cell::new(CellType::Code, "plot()");
    cell.outputs.push(Output::MimeBundle {
        data: serde_json::json!({
            "image/png": "iVBORw0KGgo=",
            "image/svg+xml": "<svg xmlns=\"http://www.w3.org/2000/svg\"><circle cx=\"4\" cy=\"4\" r=\"4\"/></svg>",
            "text/plain": "<Figure size 640x480 with 1 Axes>"
        })
        .as_object()
        .cloned()
        .unwrap_or_default(),
        metadata: Map::new(),
    });
    notebook.add_cell(cell).expect("unique_id");

    let markdown = render_markdown_notebook(&notebook);

    assert!(markdown.contains("![Notebook output](data:image/svg+xml;base64,"));
    assert!(!markdown.contains("data:image/png"));
    assert!(!markdown.contains("<Figure size"));
}

#[test]
fn exports_markdown_charts_as_images() {
    let mut notebook = Notebook::new();
    let mut cell = Cell::new(CellType::Code, "chart()");
    cell.outputs.push(Output::Chart {
        spec: serde_json::json!({
            "mark": "line",
            "data": {"values": [{"x": 1, "y": 2}, {"x": 2, "y": 4}]},
            "encoding": {"x": {"field": "x"}, "y": {"field": "y"}}
        }),
    });
    notebook.add_cell(cell).expect("unique_id");

    let markdown = render_markdown_notebook(&notebook);

    assert!(markdown.contains("![Notebook output](data:image/svg+xml;base64,"));
}

#[test]
fn exports_markdown_errors_and_sanitizes_html() {
    let mut notebook = Notebook::new();
    let mut cell = Cell::new(CellType::Code, "report()");
    cell.outputs.push(Output::Error {
        name: "RuntimeError".to_owned(),
        message: "boom".to_owned(),
        traceback: vec!["at line 1".to_owned()],
    });
    cell.outputs.push(Output::Html {
        html: "<script>alert(1)</script><b>safe</b>".to_owned(),
    });
    notebook.add_cell(cell).expect("unique_id");

    let markdown = render_markdown_notebook(&notebook);

    assert!(markdown.contains("**RuntimeError**"));
    assert!(markdown.contains("boom"));
    assert!(markdown.contains("at line 1"));
    assert!(markdown.contains("<b>safe</b>"));
    assert!(!markdown.contains("<script>"));
}

#[test]
fn lengthens_markdown_fences_when_source_contains_backticks() {
    let mut notebook = Notebook::new();
    notebook
        .add_cell(Cell::new(CellType::Code, "let fence = \"```\";"))
        .expect("unique_id");

    let markdown = render_markdown_notebook(&notebook);

    assert!(markdown.contains("````rust\nlet fence = \"```\";\n````"));
}
