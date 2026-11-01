use mars_core::{CellType, Output};
use mars_document::{Cell, Notebook};
use mars_renderer::render_notebook;
use serde_json::Map;

#[test]
fn renders_markdown_and_escapes_code() {
    let mut notebook = Notebook::new();
    notebook
        .add_cell(Cell::new(CellType::Markdown, "# Hello\n\n**Mars**"))
        .expect("unique_id");
    notebook
        .add_cell(Cell::new(CellType::Code, "let x = \"<unsafe>\";"))
        .expect("unique_id");

    let html = render_notebook(&notebook);

    assert!(html.contains("<h1>Hello</h1>"));
    assert!(html.contains("&lt;unsafe&gt;"));
}

#[test]
fn sanitizes_html_output() {
    let mut notebook = Notebook::new();
    let mut cell = Cell::new(CellType::Code, "println!(\"x\");");
    cell.outputs.push(Output::Html {
        html: "<script>alert(1)</script><b>safe</b>".to_owned(),
    });
    notebook.add_cell(cell).expect("unique_id");

    let html = render_notebook(&notebook);

    assert!(!html.contains("<script>alert(1)</script>"));
    assert!(html.contains("<b>safe</b>"));
}

#[test]
fn renders_latex_and_chart_outputs() {
    let mut notebook = Notebook::new();
    let mut cell = Cell::new(CellType::Code, "plot()");
    cell.outputs.push(Output::Latex {
        latex: "E = mc^2".to_owned(),
    });
    cell.outputs.push(Output::Chart {
        spec: serde_json::json!({"mark": "line"}),
    });
    notebook.add_cell(cell).expect("unique_id");

    let html = render_notebook(&notebook);

    assert!(html.contains("data-latex=\"E = mc^2\""));
    assert!(html.contains("&quot;mark&quot;"));
}

#[test]
fn renders_common_latex_fractions_without_network_assets() {
    let mut notebook = Notebook::new();
    let mut cell = Cell::new(CellType::Code, "math()");
    cell.outputs.push(Output::Latex {
        latex: r"\frac{a+b}{c}".to_owned(),
    });
    notebook.add_cell(cell).expect("unique_id");

    let html = render_notebook(&notebook);

    assert!(html.contains("class=\"math-frac\""));
    assert!(html.contains(">a+b</span>"));
    assert!(html.contains(">c</span>"));
}

#[test]
fn renders_chart_as_offline_svg() {
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

    let html = render_notebook(&notebook);

    assert!(html.contains("class=\"chart-svg\""));
    assert!(html.contains("<polyline"));
    assert!(!html.contains("https://"));
}

#[test]
fn prefers_svg_when_rendering_image_mime_bundles() {
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

    let html = render_notebook(&notebook);

    assert!(html.contains("data:image/svg+xml;base64,"));
    assert!(!html.contains("data:image/png;base64,"));
    assert!(html.contains("alt=\"Notebook output\""));
}

#[test]
fn renders_base64_png_mime_bundles() {
    let mut notebook = Notebook::new();
    let mut cell = Cell::new(CellType::Code, "plot()");
    cell.outputs.push(Output::MimeBundle {
        data: serde_json::json!({
            "image/png": "iVBORw0KGgo=",
            "text/plain": "<Figure size 640x480 with 1 Axes>"
        })
        .as_object()
        .cloned()
        .unwrap_or_default(),
        metadata: Map::new(),
    });
    notebook.add_cell(cell).expect("unique_id");

    let html = render_notebook(&notebook);

    assert!(html.contains("data:image/png;base64,iVBORw0KGgo="));
}

#[test]
fn renders_plotters_svg_html_as_vector_image() {
    let mut notebook = Notebook::new();
    let mut cell = Cell::new(CellType::Code, "chart.evcxr_display()");
    cell.outputs.push(Output::Html {
        html: "<div style=\"\"><svg xmlns=\"http://www.w3.org/2000/svg\"><rect width=\"10\" height=\"10\"/></svg></div>".to_owned(),
    });
    notebook.add_cell(cell).expect("unique_id");

    let html = render_notebook(&notebook);

    assert!(html.contains("class=\"output output-image\""));
    assert!(html.contains("data:image/svg+xml;base64,"));
    assert!(!html.contains("<div style=\"\">"));
}

#[test]
fn keeps_html_with_an_embedded_svg_and_visible_text() {
    let mut notebook = Notebook::new();
    let mut cell = Cell::new(CellType::Code, "report()");
    cell.outputs.push(Output::Html {
        html: "<p>Quarterly report</p><svg xmlns=\"http://www.w3.org/2000/svg\"><rect width=\"4\" height=\"4\"/></svg>".to_owned(),
    });
    notebook.add_cell(cell).expect("unique_id");

    let html = render_notebook(&notebook);

    assert!(html.contains("Quarterly report"));
    assert!(!html.contains("data:image/svg+xml;base64,"));
}

#[test]
fn renders_self_closing_svg_html_as_a_vector_image() {
    let mut notebook = Notebook::new();
    let mut cell = Cell::new(CellType::Code, "icon()");
    cell.outputs.push(Output::Html {
        html: "<svg xmlns=\"http://www.w3.org/2000/svg\"/>".to_owned(),
    });
    notebook.add_cell(cell).expect("unique_id");

    let html = render_notebook(&notebook);

    assert!(html.contains("data:image/svg+xml;base64,"));
}
