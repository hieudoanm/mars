use mars_core::{CellType, Output, PaperSize};
use mars_document::{Cell, Notebook};
use mars_renderer::chart::render_chart_svg;
use mars_renderer::highlight::highlight_rust;
use mars_renderer::html::render_notebook;
use mars_renderer::latex::render_latex_static;
use mars_renderer::markdown::{render_markdown, render_markdown_notebook};
use mars_renderer::output::render_output;
use mars_renderer::pdf::markdown::{MdLineType, render_markdown_to_pdf};
use mars_renderer::pdf::render::render_pdf_notebook;
use mars_renderer::pdf::tokens::{highlight_rust_tokens, pdf_escape, rust_keywords, rust_types};
use mars_renderer::pdf::wrap::wrap_text;
use mars_renderer::util::{
    escape_html, extract_svg, has_visible_text, image_data_uri, is_safe_image_type,
    mime_value_to_string,
};

fn notebook_with(cells: Vec<Cell>) -> Notebook {
    let mut notebook = Notebook::new();
    notebook.metadata.title = Some("Demo <Notebook>".to_owned());
    for cell in cells {
        notebook.add_cell(cell).expect("adds cell");
    }
    notebook
}

fn chart_spec(mark: &str) -> serde_json::Value {
    serde_json::json!({
        "mark": mark,
        "data": {
            "values": [
                {"x": "a", "y": 1},
                {"x": "b", "y": 3},
                {"x": "c", "y": 2}
            ]
        },
        "encoding": {
            "x": {"field": "x"},
            "y": {"field": "y"}
        }
    })
}

#[test]
fn escape_html_escapes_markup_characters() {
    assert_eq!(
        escape_html("<tag attr=\"v\"> & 'text'"),
        "&lt;tag attr=&quot;v&quot;&gt; &amp; &#39;text&#39;"
    );
}

#[test]
fn image_data_uri_handles_empty_data_and_data_urls() {
    assert_eq!(image_data_uri("image/png", "   "), None);
    assert_eq!(
        image_data_uri("image/png", "data:image/png;base64,AAAA"),
        Some("data:image/png;base64,AAAA".to_owned())
    );
    assert_eq!(
        image_data_uri("image/png", "aGVsbG8="),
        Some("data:image/png;base64,aGVsbG8=".to_owned())
    );
}

#[test]
fn image_data_uri_encodes_svg_markup() {
    let svg = "<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>";
    let uri = image_data_uri("image/svg+xml", svg).expect("svg uri");
    assert!(uri.starts_with("data:image/svg+xml;base64,"));
    assert!(!uri.contains("<svg"));

    assert_eq!(
        image_data_uri("image/svg+xml", "PHN2Zy8+"),
        Some("data:image/svg+xml;base64,PHN2Zy8+".to_owned())
    );
}

#[test]
fn safe_image_types_and_svg_extraction_are_restricted() {
    assert!(is_safe_image_type("image/png"));
    assert!(is_safe_image_type("image/svg+xml"));
    assert!(!is_safe_image_type("image/bmp"));
    assert!(!is_safe_image_type("text/html"));

    assert_eq!(
        extract_svg("<svg><path/></svg>"),
        Some("<svg><path/></svg>")
    );
    assert_eq!(extract_svg("before <svg></svg> after"), None);
    assert_eq!(extract_svg("no svg here"), None);
    assert!(has_visible_text("hello <b>world</b>"));
    assert!(!has_visible_text("<b></b>"));
}

#[test]
fn mime_value_to_string_accepts_strings_and_string_arrays() {
    assert_eq!(
        mime_value_to_string(&serde_json::json!("plain")),
        Some("plain".to_owned())
    );
    assert_eq!(
        mime_value_to_string(&serde_json::json!(["a", "b"])),
        Some("ab".to_owned())
    );
    assert_eq!(mime_value_to_string(&serde_json::json!(42)), None);
    assert_eq!(mime_value_to_string(&serde_json::json!(["a", 1])), None);
}

#[test]
fn render_output_text_and_error_escape_content() {
    let text = render_output(&Output::Text {
        text: "<hello> & \"bye\"".to_owned(),
    });
    assert!(text.contains("output-text"));
    assert!(text.contains("&lt;hello&gt; &amp; &quot;bye&quot;"));

    let error = render_output(&Output::Error {
        name: "Panic".to_owned(),
        message: "boom".to_owned(),
        traceback: vec!["at main".to_owned(), "at run".to_owned()],
    });
    assert!(error.contains("output-error"));
    assert!(error.contains("<strong>Panic</strong>"));
    assert!(error.contains("boom"));
    assert!(error.contains("at main\nat run"));
}

#[test]
fn render_output_html_extracts_svg_and_sanitizes_markup() {
    let svg = render_output(&Output::Html {
        html: "<div><svg><path d=\"M0 0\"/></svg></div>".to_owned(),
    });
    assert!(svg.contains("output-image"));
    assert!(svg.contains("data:image/svg+xml;base64,"));

    let html = render_output(&Output::Html {
        html: "<p onclick=\"x()\">ok</p><script>alert(1)</script>".to_owned(),
    });
    assert!(html.contains("<p>ok</p>"));
    assert!(!html.contains("<script>"));
    assert!(!html.contains("onclick"));
}

#[test]
fn render_output_image_validates_mime_type_and_data() {
    let unsafe_output = render_output(&Output::Image {
        mime_type: "image/bmp".to_owned(),
        data: "AAAA".to_owned(),
    });
    assert!(unsafe_output.contains("Unsupported image MIME type"));

    let empty = render_output(&Output::Image {
        mime_type: "image/png".to_owned(),
        data: "  ".to_owned(),
    });
    assert!(empty.contains("Image data is empty"));

    let image = render_output(&Output::Image {
        mime_type: "image/png".to_owned(),
        data: "aGVsbG8=".to_owned(),
    });
    assert!(image.contains("output-image"));
    assert!(image.contains("data:image/png;base64,aGVsbG8="));
}

#[test]
fn render_output_table_latex_and_chart() {
    let table = render_output(&Output::Table {
        columns: vec!["name".to_owned(), "value".to_owned()],
        rows: vec![vec!["<x>".to_owned(), "1".to_owned()]],
    });
    assert!(table.contains("<th>name</th>"));
    assert!(table.contains("<td>&lt;x&gt;</td>"));

    let latex = render_output(&Output::Latex {
        latex: "\\frac{1}{2}".to_owned(),
    });
    assert!(latex.contains("math-frac"));
    assert!(latex.contains("data-latex=\"\\frac{1}{2}\""));

    let chart = render_output(&Output::Chart {
        spec: chart_spec("bar"),
    });
    assert!(chart.contains("output chart"));
    assert!(chart.contains("<rect"));
    assert!(chart.contains("data-chart="));
}

#[test]
fn render_output_mime_bundle_prefers_known_mime_types() {
    let mut image_bundle = serde_json::Map::new();
    image_bundle.insert("image/png".to_owned(), serde_json::json!("aGVsbG8="));
    image_bundle.insert("text/plain".to_owned(), serde_json::json!("ignored"));
    let image = render_output(&Output::MimeBundle {
        data: image_bundle,
        metadata: serde_json::Map::new(),
    });
    assert!(image.contains("output-image"));

    let mut text_bundle = serde_json::Map::new();
    text_bundle.insert("text/plain".to_owned(), serde_json::json!("plain <text>"));
    let text = render_output(&Output::MimeBundle {
        data: text_bundle,
        metadata: serde_json::Map::new(),
    });
    assert!(text.contains("output-text"));
    assert!(text.contains("plain &lt;text&gt;"));

    let mut markdown_bundle = serde_json::Map::new();
    markdown_bundle.insert("text/markdown".to_owned(), serde_json::json!("**bold**"));
    let markdown = render_output(&Output::MimeBundle {
        data: markdown_bundle,
        metadata: serde_json::Map::new(),
    });
    assert!(markdown.contains("<strong>bold</strong>"));

    let fallback = render_output(&Output::MimeBundle {
        data: serde_json::Map::new(),
        metadata: serde_json::Map::new(),
    });
    assert!(fallback.contains("output"));
}

#[test]
fn render_markdown_renders_commonmark_and_sanitizes_html() {
    let rendered = render_markdown(
        "# Title\n\nHello **bold** and ~~gone~~.\n\n| A | B |\n|---|---|\n| 1 | 2 |\n\n<script>alert(1)</script>",
    );
    assert!(rendered.contains("<h1>Title</h1>"));
    assert!(rendered.contains("<strong>bold</strong>"));
    assert!(rendered.contains("<del>gone</del>"));
    assert!(rendered.contains("<table>"));
    assert!(rendered.contains("<td>1</td>"));
    assert!(!rendered.contains("<script>"));
}

#[test]
fn render_markdown_notebook_exports_mixed_cells_and_outputs() {
    let mut code = Cell::new(CellType::Code, "let x = 1;");
    code.execution_count = Some(1);
    code.outputs = vec![
        Output::Text {
            text: "hello\n".to_owned(),
        },
        Output::Table {
            columns: vec!["a".to_owned(), "b".to_owned()],
            rows: vec![vec!["1".to_owned(), "2".to_owned()]],
        },
        Output::Latex {
            latex: "x^2".to_owned(),
        },
        Output::Image {
            mime_type: "image/png".to_owned(),
            data: "aGVsbG8=".to_owned(),
        },
    ];
    let notebook = notebook_with(vec![
        Cell::new(CellType::Markdown, "# Demo <Notebook>\n\nIntro"),
        code,
        Cell::new(CellType::Raw, "raw <text>"),
    ]);

    let markdown = render_markdown_notebook(&notebook);
    assert!(markdown.contains("# Demo <Notebook>"));
    assert!(markdown.contains("Intro"));
    assert!(markdown.contains("```rust\nlet x = 1;"));
    assert!(markdown.contains("```text\nhello\n```"));
    assert!(markdown.contains("| a | b |"));
    assert!(markdown.contains("$$\nx^2\n$$"));
    assert!(markdown.contains("![Notebook output](data:image/png;base64,aGVsbG8=)"));
    assert!(markdown.contains("```text\nraw <text>\n```"));
}

#[test]
fn render_notebook_builds_standalone_html_document() {
    let mut code = Cell::new(CellType::Code, "fn main() { println!(\"hi\"); }");
    code.execution_count = Some(1);
    code.outputs = vec![Output::Text {
        text: "ok".to_owned(),
    }];
    let notebook = notebook_with(vec![
        Cell::new(CellType::Markdown, "**bold** intro"),
        code,
        Cell::new(CellType::Raw, "<raw>"),
    ]);

    let html = render_notebook(&notebook);
    assert!(html.starts_with("<!doctype html>"));
    assert!(html.contains("<title>Demo &lt;Notebook&gt;</title>"));
    assert!(html.contains("cell-markdown"));
    assert!(html.contains("cell-code"));
    assert!(html.contains("cell-raw"));
    assert!(html.contains("In [1]"));
    assert!(html.contains("rust-keyword"));
    assert!(html.contains("output-text"));
    assert!(html.contains("&lt;raw&gt;"));
}

#[test]
fn render_latex_static_renders_fraction_and_escapes_text() {
    let rendered = render_latex_static("\\frac{a{b}c}{d} & <x>");
    assert!(rendered.contains("math-frac"));
    assert!(rendered.contains("a{b}c"));
    assert!(rendered.contains("&amp; &lt;x&gt;"));

    let malformed = render_latex_static("\\frac{1");
    assert!(malformed.contains("\\frac{1"));
}

#[test]
fn render_chart_svg_handles_marks_and_invalid_specs() {
    assert!(render_chart_svg(&serde_json::json!({})).contains("Chart data is unavailable"));
    assert!(
        render_chart_svg(&serde_json::json!({
            "data": {"values": []}
        }))
        .contains("requires an encoded y field")
    );
    assert!(
        render_chart_svg(&serde_json::json!({
            "data": {"values": [{"y": "nope"}]},
            "encoding": {"y": {"field": "y"}}
        }))
        .contains("no numeric y values")
    );

    for (mark, expected) in [
        ("bar", "<rect"),
        ("line", "<polyline"),
        ("point", "<circle"),
        ("area", "Unsupported chart mark"),
    ] {
        let svg = render_chart_svg(&chart_spec(mark));
        assert!(svg.contains("<svg"), "mark {mark}");
        assert!(svg.contains(expected), "mark {mark}");
    }
}

#[test]
fn highlight_rust_marks_tokens_and_escapes_html() {
    let highlighted =
        highlight_rust("let name: String = \"a<b>\"; // tail\nlet n = 42;\nprintln!(\"{name}\");");
    assert!(highlighted.contains("<span class=\"rust-keyword\">let</span>"));
    assert!(highlighted.contains("<span class=\"rust-type\">String</span>"));
    assert!(highlighted.contains("<span class=\"rust-string\">&quot;a&lt;b&gt;&quot;</span>"));
    assert!(highlighted.contains("<span class=\"rust-comment\">// tail</span>"));
    assert!(highlighted.contains("<span class=\"rust-number\">42</span>"));
    assert!(highlighted.contains("<span class=\"rust-macro\">println</span>!"));
}

#[test]
fn pdf_tokens_escape_text_and_classify_rust() {
    assert_eq!(pdf_escape("a(b)c\\d"), "a\\(b\\)c\\\\d");
    assert!(rust_keywords().contains(&"fn"));
    assert!(rust_types().contains(&"i32"));

    let tokens = highlight_rust_tokens("let x = 1; // done");
    let keyword = tokens.iter().find(|(text, _)| *text == "let");
    assert_eq!(
        keyword.map(|(_, color)| *color),
        Some((0.776, 0.490, 0.870))
    );
    let number = tokens.iter().find(|(text, _)| *text == "1");
    assert!(number.is_some());
    assert!(tokens.iter().any(|(text, _)| text.starts_with("//")));
}

#[test]
fn wrap_text_preserves_lines_and_empty_width() {
    assert_eq!(wrap_text("short", 1_000.0, "/F1", 10.0), vec!["short"]);
    assert_eq!(
        wrap_text("a\n\nb", 1_000.0, "/F1", 10.0),
        vec!["a", "", "b"]
    );
    assert_eq!(wrap_text("hello", 0.0, "/F1", 10.0), vec!["hello"]);
}

#[test]
fn render_markdown_to_pdf_produces_typed_lines() {
    let lines =
        render_markdown_to_pdf("# Title\n\nParagraph text.\n\n![alt](data:image/png;base64,AAA)");
    assert!(
        lines
            .iter()
            .any(|line| line.text == "Title" && line.kind == MdLineType::H1)
    );
    assert!(
        lines
            .iter()
            .any(|line| line.text.contains("Paragraph text"))
    );
    assert!(lines.iter().any(|line| line.text == "[Image: alt]"));
}

#[test]
fn render_pdf_notebook_emits_pdf_bytes_for_mixed_content() {
    let mut code = Cell::new(CellType::Code, "fn main() {\n    println!(\"hi\");\n}");
    code.execution_count = Some(1);
    let mut bundle = serde_json::Map::new();
    bundle.insert("text/plain".to_owned(), serde_json::json!("bundle text"));
    code.outputs = vec![
        Output::Text {
            text: "ok\n".to_owned(),
        },
        Output::Error {
            name: "Panic".to_owned(),
            message: "boom".to_owned(),
            traceback: vec!["at main".to_owned()],
        },
        Output::Html {
            html: "<p>html</p><script>x</script>".to_owned(),
        },
        Output::Image {
            mime_type: "image/png".to_owned(),
            data: "aGVsbG8=".to_owned(),
        },
        Output::Table {
            columns: vec!["a".to_owned(), "b".to_owned()],
            rows: vec![vec!["1".to_owned(), "2".to_owned()]],
        },
        Output::Latex {
            latex: "\\frac{1}{2}".to_owned(),
        },
        Output::Chart {
            spec: chart_spec("line"),
        },
        Output::MimeBundle {
            data: bundle,
            metadata: serde_json::Map::new(),
        },
    ];
    let notebook = notebook_with(vec![
        Cell::new(
            CellType::Markdown,
            "# Hello\n\nSome **bold** text.\n\n![chart](data:image/png;base64,AAA)",
        ),
        code,
        Cell::new(CellType::Raw, "raw <>&"),
    ]);

    let pdf = render_pdf_notebook(&notebook, PaperSize::A4);
    assert!(pdf.starts_with(b"%PDF-1.4"));
    assert!(pdf.windows(5).any(|window| window == b"%%EOF"));
    assert!(pdf.len() > 1_000);
}

#[test]
fn render_markdown_notebook_covers_output_variants() {
    let mut chart_bundle = serde_json::Map::new();
    chart_bundle.insert(
        "application/vnd.vega-lite.v5+json".to_owned(),
        chart_spec("point"),
    );
    let mut html_bundle = serde_json::Map::new();
    html_bundle.insert("text/html".to_owned(), serde_json::json!("<p>bundle</p>"));
    let mut latex_bundle = serde_json::Map::new();
    latex_bundle.insert("text/latex".to_owned(), serde_json::json!("\\alpha"));
    let mut markdown_bundle = serde_json::Map::new();
    markdown_bundle.insert("text/markdown".to_owned(), serde_json::json!("**md**"));
    let mut text_bundle = serde_json::Map::new();
    text_bundle.insert("text/plain".to_owned(), serde_json::json!("plain"));

    let mut code = Cell::new(CellType::Code, "outputs");
    code.outputs = vec![
        Output::Error {
            name: "Panic".to_owned(),
            message: "boom\n".to_owned(),
            traceback: vec!["trace".to_owned()],
        },
        Output::Html {
            html: "<svg><path/></svg>".to_owned(),
        },
        Output::Html {
            html: "<p>ok</p><script>x</script>".to_owned(),
        },
        Output::Image {
            mime_type: "image/bmp".to_owned(),
            data: "AAAA".to_owned(),
        },
        Output::Image {
            mime_type: "image/png".to_owned(),
            data: "  ".to_owned(),
        },
        Output::Table {
            columns: Vec::new(),
            rows: Vec::new(),
        },
        Output::Latex {
            latex: "\\alpha".to_owned(),
        },
        Output::Chart {
            spec: chart_spec("point"),
        },
        Output::MimeBundle {
            data: chart_bundle,
            metadata: serde_json::Map::new(),
        },
        Output::MimeBundle {
            data: html_bundle,
            metadata: serde_json::Map::new(),
        },
        Output::MimeBundle {
            data: latex_bundle,
            metadata: serde_json::Map::new(),
        },
        Output::MimeBundle {
            data: markdown_bundle,
            metadata: serde_json::Map::new(),
        },
        Output::MimeBundle {
            data: text_bundle,
            metadata: serde_json::Map::new(),
        },
        Output::MimeBundle {
            data: serde_json::Map::new(),
            metadata: serde_json::Map::new(),
        },
    ];
    let notebook = notebook_with(vec![code]);

    let markdown = render_markdown_notebook(&notebook);
    assert!(markdown.contains("**Panic**"));
    assert!(markdown.contains("boom"));
    assert!(markdown.contains("trace"));
    assert!(markdown.contains("![Notebook output](data:image/svg+xml;base64,"));
    assert!(markdown.contains("<p>ok</p>"));
    assert!(markdown.contains("_Unsupported image output._"));
    assert!(markdown.contains("_Image output is empty._"));
    assert!(markdown.contains("_Empty table._"));
    assert!(markdown.contains("$$\n\\alpha\n$$"));
    assert!(markdown.contains("<p>bundle</p>"));
    assert!(markdown.contains("**md**"));
    assert!(markdown.contains("```text\nplain\n```"));
    assert!(markdown.contains("```json"));
}
