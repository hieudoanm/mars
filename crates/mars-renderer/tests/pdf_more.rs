use mars_core::{CellType, Output, PaperSize};
use mars_document::{Cell, Notebook};
use mars_renderer::pdf::output::render_output_to_pdf_lines;
use mars_renderer::pdf::render::render_pdf_notebook;
use serde_json::json;

fn notebook_with(cells: Vec<Cell>) -> Notebook {
    let mut notebook = Notebook::new();
    for cell in cells {
        notebook.add_cell(cell).expect("add cell");
    }
    notebook
}

#[test]
fn pdf_renders_every_paper_size() {
    let notebook = notebook_with(vec![
        Cell::new(CellType::Markdown, "# Sizes"),
        Cell::new(CellType::Code, "let x = 1;"),
    ]);
    for size in [
        PaperSize::A0,
        PaperSize::A1,
        PaperSize::A2,
        PaperSize::A3,
        PaperSize::A4,
        PaperSize::A5,
        PaperSize::A6,
        PaperSize::B4,
        PaperSize::B5,
        PaperSize::Letter,
        PaperSize::Legal,
        PaperSize::Tabloid,
    ] {
        let pdf = render_pdf_notebook(&notebook, size);
        assert!(pdf.starts_with(b"%PDF-1.4"), "{size:?} did not emit PDF");
        assert!(pdf.windows(5).any(|w| w == b"%%EOF"));
    }
}

#[test]
fn pdf_paginates_long_notebooks() {
    let mut cells = vec![Cell::new(CellType::Markdown, "# Long notebook")];
    for index in 0..40 {
        cells.push(Cell::new(
            CellType::Markdown,
            format!("## Section {index}\n\nParagraph {index} with **bold** and `code`."),
        ));
        let mut code = Cell::new(
            CellType::Code,
            format!("let value_{index} = {index};\nprintln!(\"cell {index}\");"),
        );
        code.execution_count = Some(index as u64 + 1);
        code.outputs = vec![Output::Text {
            text: format!("output {index}\n"),
        }];
        cells.push(code);
    }
    let pdf = render_pdf_notebook(&notebook_with(cells), PaperSize::A4);
    assert!(pdf.starts_with(b"%PDF-1.4"));
    // Many pages => multiple page objects.
    let text = String::from_utf8_lossy(&pdf);
    let pages = text.matches("/Type /Page").count();
    assert!(pages > 1, "expected pagination, got {pages} page objects");
}

#[test]
fn pdf_handles_long_unbroken_lines_and_markdown_structures() {
    let long_token = "x".repeat(400);
    let markdown = format!(
        "# Title\n\n- item one\n- item two\n\n```rust\nfn main() {{}}\n```\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\n{long_token}"
    );
    let mut code = Cell::new(CellType::Code, format!("let s = \"{long_token}\";"));
    code.outputs = vec![Output::Text {
        text: long_token.clone(),
    }];
    let pdf = render_pdf_notebook(
        &notebook_with(vec![
            Cell::new(CellType::Markdown, &markdown),
            code,
            Cell::new(CellType::Raw, &long_token),
        ]),
        PaperSize::Letter,
    );
    assert!(pdf.starts_with(b"%PDF-1.4"));
    assert!(pdf.len() > 1_000);
}

#[test]
fn output_to_pdf_lines_covers_all_variants() {
    let mut bundle = serde_json::Map::new();
    bundle.insert("text/plain".to_owned(), json!("bundle"));
    bundle.insert("image/png".to_owned(), json!("aGVsbG8="));

    let outputs = vec![
        Output::Text {
            text: "plain".to_owned(),
        },
        Output::Error {
            name: "Panic".to_owned(),
            message: "boom".to_owned(),
            traceback: vec!["trace".to_owned()],
        },
        Output::Html {
            html: "<p>hi</p>".to_owned(),
        },
        Output::Image {
            mime_type: "image/png".to_owned(),
            data: "aGVsbG8=".to_owned(),
        },
        Output::Table {
            columns: vec!["a".to_owned()],
            rows: vec![vec!["1".to_owned()]],
        },
        Output::Latex {
            latex: "\\frac{1}{2}".to_owned(),
        },
        Output::Chart {
            spec: json!({"mark": "bar", "data": {"values": []}}),
        },
        Output::MimeBundle {
            data: bundle,
            metadata: serde_json::Map::new(),
        },
    ];
    for output in &outputs {
        let lines = render_output_to_pdf_lines(output, 500.0);
        assert!(!lines.is_empty(), "no lines for {output:?}");
    }
    // Narrow width still produces lines.
    let narrow = render_output_to_pdf_lines(&outputs[0], 10.0);
    assert!(!narrow.is_empty());
}
