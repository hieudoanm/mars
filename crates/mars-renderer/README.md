# mars-renderer

Output rendering for the Project Mars notebook platform.

## Purpose

`mars-renderer` converts notebook cells and outputs into HTML, Markdown, and PDF formats.

## Modules

- **`html`** — `render_notebook`: HTML notebook rendering.
- **`markdown`** — `render_markdown_notebook`: Portable Markdown export.
- **`markdown_fence`** — Markdown fenced-code helpers.
- **`markdown_output`** — Per-output Markdown rendering.
- **`output`** — Output rendering helpers.
- **`pdf`** — PDF notebook rendering (`render_pdf_notebook`).
  - `pdf_cell_code`, `pdf_cell_markdown`, `pdf_cell_raw` — per-cell renderers.
  - `pdf_markdown` — Markdown-to-PDF line conversion.
  - `pdf_output` — Per-output PDF line rendering.
  - `pdf_page` — PDF page content stream generation.
  - `pdf_render` — `PdfRenderContext`, page orchestration.
  - `pdf_tokens` — Rust syntax highlighting for PDF.
  - `pdf_wrap` — Text wrapping.
  - `pdf_writer` — PDF object writer.
- **`util`** — Shared utilities (data URIs, image MIME checks, SVG extraction).
- **`chart`** — Chart SVG rendering.
- **`highlight`** — Syntax highlighting.
- **`latex`** — LaTeX rendering.

## Usage

```rust
use mars_renderer::{render_notebook, render_markdown_notebook, render_pdf_notebook};
use mars_document::Notebook;

let html = render_notebook(&notebook);
let markdown = render_markdown_notebook(&notebook);
let pdf_bytes = render_pdf_notebook(&notebook, mars_core::PaperSize::A4);
```
