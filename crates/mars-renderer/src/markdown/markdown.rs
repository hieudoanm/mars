use crate::markdown::fence::fenced_code;
use crate::markdown::output::render_markdown_output;
use mars_core::CellType;
use pulldown_cmark::{Options, Parser, html};

pub fn render_markdown(source: &str) -> String {
    let mut rendered = String::new();
    let options = Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH;
    html::push_html(&mut rendered, Parser::new_ext(source, options));
    crate::util::markdown_sanitizer()
        .clean(&rendered)
        .to_string()
}

/// Render a notebook as a portable Markdown document.
///
/// Markdown cells keep their original source, code cells become fenced Rust
/// blocks, and outputs are converted into Markdown-friendly equivalents such
/// as tables, LaTeX blocks, and data-URI images.
pub fn render_markdown_notebook(notebook: &mars_document::Notebook) -> String {
    let title = notebook
        .metadata
        .title
        .as_deref()
        .map(str::trim)
        .filter(|title| !title.is_empty());
    let has_title_heading = title.is_some_and(|title| {
        notebook
            .cells
            .iter()
            .find(|cell| cell.cell_type == CellType::Markdown && !cell.source.trim().is_empty())
            .is_some_and(|cell| {
                cell.source
                    .trim_start()
                    .lines()
                    .next()
                    .is_some_and(|line| line.trim() == format!("# {title}"))
            })
    });

    let mut blocks = Vec::new();
    if let Some(title) = title.filter(|_| !has_title_heading) {
        blocks.push(format!("# {title}"));
    }

    for cell in &notebook.cells {
        match cell.cell_type {
            CellType::Markdown => {
                let resolved = crate::util::resolve_attachment_refs(&cell.source, &cell.metadata);
                let source = resolved.trim();
                if !source.is_empty() {
                    blocks.push(source.to_owned());
                }
            }
            CellType::Code => {
                let mut block = fenced_code("rust", &cell.source);
                for output in &cell.outputs {
                    if let Some(rendered) = render_markdown_output(output) {
                        block.push_str("\n\n");
                        block.push_str(&rendered);
                    }
                }
                blocks.push(block);
            }
            CellType::Raw => {
                let source = cell.source.trim();
                if !source.is_empty() {
                    blocks.push(fenced_code("text", source));
                }
            }
        }
    }

    blocks.join("\n\n")
}
