use crate::highlight::highlight_rust;
use crate::markdown::render_markdown;
use crate::output::render_output;
use crate::util::escape_html;
use mars_core::CellType;
use mars_document::Notebook;

pub fn render_notebook(notebook: &Notebook) -> String {
    let title = notebook
        .metadata
        .title
        .as_deref()
        .unwrap_or("Mars notebook");
    let mut body = String::new();
    for cell in &notebook.cells {
        let cell_class = match cell.cell_type {
            CellType::Markdown => "markdown",
            CellType::Code => "code",
            CellType::Raw => "raw",
        };
        body.push_str(&format!("<article class=\"cell cell-{cell_class}\">"));
        match cell.cell_type {
            CellType::Markdown => body.push_str(&render_markdown(&cell.source)),
            CellType::Code => {
                body.push_str(&format!(
                    "<div class=\"cell-label\">In [{}]</div><pre class=\"code\"><code class=\"language-rust\">{}</code></pre>",
                    cell.execution_count.map_or_else(|| " ".to_owned(), |count| count.to_string()),
                    highlight_rust(&cell.source)
                ));
                for output in &cell.outputs {
                    body.push_str(&render_output(output));
                }
            }
            CellType::Raw => body.push_str(&format!(
                "<pre class=\"raw\"><code>{}</code></pre>",
                escape_html(&cell.source)
            )),
        }
        body.push_str("</article>");
    }

    let css = r#"
:root { color-scheme: dark; font-family: system-ui, sans-serif; background: #171717; color: #e5e5e5; }
body { max-width: 960px; margin: 0 auto; padding: 2rem; }
.cell { margin: 1.5rem 0; }
.cell-label { color: #f97316; font-size: .85rem; margin-bottom: .35rem; }
pre { overflow-x: auto; border-radius: .5rem; padding: 1rem; }
.code { background: #262626; border: 1px solid #3a3a3a; border-left: 4px solid #f97316; }
.raw { background: #262626; border: 1px solid #3a3a3a; border-left: 4px solid #a3a3a3; }
.code code, .raw code { font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace; font-size: .9rem; line-height: 1.5; }
.rust-keyword { color: #c678dd; }
.rust-type { color: #e06c75; }
.rust-comment { color: #5c6370; font-style: italic; }
.rust-string { color: #c3e88d; }
.rust-number { color: #d19a66; }
.rust-ident { color: #e5e5e5; }
.rust-macro { color: #e2c08d; font-weight: 600; }
.output-text { background: #1f2937; border: 1px solid #374151; border-left: 4px solid #3b82f6; }
.output-error { background: #262626; border: 1px solid #3a3a3a; border-left: 4px solid #ef4444; color: #fecaca; }
table { border-collapse: collapse; } th, td { border: 1px solid #525252; padding: .4rem .7rem; }
img { max-width: 100%; }
a { color: #fb923c; }
.math { color: #fbbf24; font-family: serif; font-size: 1.15rem; }
.chart { background: #202020; border: 1px solid #404040; padding: .5rem; }
.math-text { display: inline-block; padding: .6rem 1rem; border-radius: .35rem; background: #202020; font-family: Georgia, serif; font-size: 1.2rem; }
.math-frac { display: inline-flex; flex-direction: column; vertical-align: middle; text-align: center; line-height: 1.1; margin: 0 .15rem; }
.math-frac > span:first-child { border-bottom: 1px solid currentColor; padding: 0 .2rem .1rem; }
.math-frac > span:last-child { padding: .1rem .2rem 0; }
.chart-svg { width: 100%; max-width: 720px; height: 260px; }
"#;
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{}</title>
<style>{css}</style>
</head>
<body>
<h1>{}</h1>
{}
</body>
</html>
"#,
        escape_html(title),
        escape_html(title),
        body
    )
}
