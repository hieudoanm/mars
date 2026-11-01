use mars_core::{CellType, Output, PaperSize};
use mars_document::{Cell, Notebook};
use mars_renderer::render_pdf_notebook;

#[test]
fn renders_minimal_pdf() {
    let mut notebook = Notebook::new();
    notebook
        .add_cell(Cell::new(CellType::Markdown, "# Hello"))
        .expect("unique_id");

    let pdf = render_pdf_notebook(&notebook, PaperSize::A4);
    let content = String::from_utf8_lossy(&pdf);
    assert!(content.starts_with("%PDF-1.4"));
    assert!(content.contains("%%EOF"));
    assert!(content.contains("Hello"));
}

#[test]
fn renders_code_cell_pdf() {
    let mut notebook = Notebook::new();
    let mut cell = Cell::new(CellType::Code, "1 + 1");
    cell.outputs.push(Output::Text {
        text: "2\n".to_owned(),
    });
    notebook.add_cell(cell).expect("unique_id");

    let pdf = render_pdf_notebook(&notebook, PaperSize::Letter);
    let content = String::from_utf8_lossy(&pdf);
    assert!(content.starts_with("%PDF-1.4"));
    assert!(content.contains("(1)"));
    assert!(content.contains("(+)"));
    assert!(content.contains("(2)"));
}

#[test]
fn a4_dimensions() {
    let (w, h) = PaperSize::A4.dimensions();
    assert!((w - 595.28).abs() < 0.01);
    assert!((h - 841.89).abs() < 0.01);
    assert!((w / h - (1.0 / 2.0_f64.sqrt())).abs() < 0.01);
}

#[test]
fn letter_dimensions() {
    let (w, h) = PaperSize::Letter.dimensions();
    assert!((w - 612.0).abs() < 0.01);
    assert!((h - 792.0).abs() < 0.01);
}

#[test]
fn parse_paper_size() {
    assert_eq!(PaperSize::parse("a4").unwrap(), PaperSize::A4);
    assert_eq!(PaperSize::parse("letter").unwrap(), PaperSize::Letter);
    assert_eq!(PaperSize::parse("a3").unwrap(), PaperSize::A3);
    assert!(PaperSize::parse("invalid").is_err());
}

#[test]
fn iso216_check() {
    assert!(PaperSize::A4.is_iso216());
    assert!(PaperSize::A3.is_iso216());
    assert!(!PaperSize::Letter.is_iso216());
}

#[test]
fn default_is_a4() {
    assert_eq!(PaperSize::default(), PaperSize::A4);
}

#[test]
fn pdf_contains_title() {
    let mut notebook = Notebook::new();
    notebook.metadata.title = Some("My Report".to_owned());
    notebook
        .add_cell(Cell::new(CellType::Markdown, "# My Report"))
        .expect("unique_id");

    let pdf = render_pdf_notebook(&notebook, PaperSize::A4);
    let content = String::from_utf8_lossy(&pdf);
    assert!(content.contains("My Report"));
}
