use base64::{Engine as _, engine::general_purpose::STANDARD};
use mars_core::{CellType, Output};
use mars_document::Notebook;
use pulldown_cmark::{Options, Parser, html};
use serde_json::Value;

mod pdf;
pub use pdf::render_pdf_notebook;

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
                    "<div class=\"cell-label\">In [{}]</div><pre class=\"code\"><code>{}</code></pre>",
                    cell.execution_count.map_or_else(|| " ".to_owned(), |count| count.to_string()),
                    escape_html(&cell.source)
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

    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{}</title>
<style>
:root {{ color-scheme: dark; font-family: system-ui, sans-serif; background: #171717; color: #e5e5e5; }}
body {{ max-width: 960px; margin: 0 auto; padding: 2rem; }}
.cell {{ margin: 1.5rem 0; }}
.cell-label {{ color: #f97316; font-size: .85rem; margin-bottom: .35rem; }}
pre {{ overflow-x: auto; border-radius: .5rem; padding: 1rem; }}
.code, .raw, .output-text, .output-error {{ background: #262626; }}
.output {{ margin-top: .75rem; }}
.output-error {{ border-left: 3px solid #ef4444; color: #fecaca; }}
table {{ border-collapse: collapse; }} th, td {{ border: 1px solid #525252; padding: .4rem .7rem; }}
img {{ max-width: 100%; }}
a {{ color: #fb923c; }}
.math {{ color: #fbbf24; font-family: serif; font-size: 1.15rem; }}
.chart {{ background: #202020; border: 1px solid #404040; padding: .5rem; }}
.math-text {{ display: inline-block; padding: .6rem 1rem; border-radius: .35rem; background: #202020; font-family: Georgia, serif; font-size: 1.2rem; }}
.math-frac {{ display: inline-flex; flex-direction: column; vertical-align: middle; text-align: center; line-height: 1.1; margin: 0 .15rem; }}
.math-frac > span:first-child {{ border-bottom: 1px solid currentColor; padding: 0 .2rem .1rem; }}
.math-frac > span:last-child {{ padding: .1rem .2rem 0; }}
.chart-svg {{ width: 100%; max-width: 720px; height: 260px; }}
</style>
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

pub(crate) fn render_markdown(source: &str) -> String {
    let mut rendered = String::new();
    let options = Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH;
    html::push_html(&mut rendered, Parser::new_ext(source, options));
    ammonia::clean(&rendered)
}

/// Render a notebook as a portable Markdown document.
///
/// Markdown cells keep their original source, code cells become fenced Rust
/// blocks, and outputs are converted into Markdown-friendly equivalents such
/// as tables, LaTeX blocks, and data-URI images.
pub fn render_markdown_notebook(notebook: &Notebook) -> String {
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
                let source = cell.source.trim();
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

fn render_markdown_output(output: &Output) -> Option<String> {
    match output {
        Output::Text { text } => {
            let text = text.trim_end_matches('\n');
            (!text.is_empty()).then(|| fenced_code("text", text))
        }
        Output::Error {
            name,
            message,
            traceback,
        } => {
            let mut content = message.trim_end_matches('\n').to_owned();
            let trace = traceback.join("\n");
            if !trace.trim().is_empty() {
                content.push('\n');
                content.push_str(trace.trim_end_matches('\n'));
            }
            Some(format!("**{name}**\n\n{}", fenced_code("text", &content)))
        }
        Output::Html { html } => Some(render_markdown_html_output(html)),
        Output::Image { mime_type, data } => Some(render_markdown_image(mime_type, data)),
        Output::Table { columns, rows } => Some(render_markdown_table(columns, rows)),
        Output::Latex { latex } => {
            let latex = latex.trim();
            (!latex.is_empty()).then(|| format!("$$\n{latex}\n$$"))
        }
        Output::Chart { spec } => Some(render_markdown_chart(spec)),
        Output::MimeBundle { data, .. } => render_markdown_mime_bundle(data),
    }
}

fn render_markdown_html_output(html: &str) -> String {
    if let Some(svg) = extract_svg(html) {
        return render_markdown_image("image/svg+xml", svg);
    }
    ammonia::clean(html)
}

fn render_markdown_image(mime_type: &str, data: &str) -> String {
    if !is_safe_image_type(mime_type) {
        return "_Unsupported image output._".to_owned();
    }
    let Some(source) = image_data_uri(mime_type, data) else {
        return "_Image output is empty._".to_owned();
    };
    format!("![Notebook output]({source})")
}

fn render_markdown_chart(spec: &Value) -> String {
    let svg = render_chart_svg(spec);
    if svg.trim_start().starts_with("<svg") {
        render_markdown_image("image/svg+xml", &svg)
    } else {
        svg
    }
}

fn render_markdown_table(columns: &[String], rows: &[Vec<String>]) -> String {
    if columns.is_empty() {
        return "_Empty table._".to_owned();
    }
    let header = columns
        .iter()
        .map(|column| escape_markdown_cell(column))
        .collect::<Vec<_>>()
        .join(" | ");
    let divider = columns
        .iter()
        .map(|_| "---")
        .collect::<Vec<_>>()
        .join(" | ");
    let mut lines = vec![format!("| {header} |"), format!("| {divider} |")];
    for row in rows {
        let cells = (0..columns.len())
            .map(|index| {
                row.get(index)
                    .map_or_else(String::new, |value| escape_markdown_cell(value))
            })
            .collect::<Vec<_>>()
            .join(" | ");
        lines.push(format!("| {cells} |"));
    }
    lines.join("\n")
}

fn render_markdown_mime_bundle(data: &serde_json::Map<String, Value>) -> Option<String> {
    for mime_type in [
        "image/svg+xml",
        "image/png",
        "image/jpeg",
        "image/gif",
        "image/webp",
    ] {
        if let Some(value) = data.get(mime_type).and_then(mime_value_to_string) {
            return Some(render_markdown_image(mime_type, &value));
        }
    }

    if let Some(spec) = data
        .get("application/vnd.vega.v5+json")
        .or_else(|| data.get("application/vnd.vega-lite.v5+json"))
        .or_else(|| data.get("application/vnd.vega-lite+json"))
    {
        return Some(render_markdown_chart(spec));
    }

    if let Some(html) = data.get("text/html").and_then(mime_value_to_string) {
        return Some(render_markdown_html_output(&html));
    }
    if let Some(latex) = data.get("text/latex").and_then(mime_value_to_string) {
        let latex = latex.trim();
        if !latex.is_empty() {
            return Some(format!("$$\n{latex}\n$$"));
        }
    }
    if let Some(markdown) = data.get("text/markdown").and_then(mime_value_to_string) {
        let markdown = markdown.trim();
        if !markdown.is_empty() {
            return Some(markdown.to_owned());
        }
    }
    if let Some(text) = data.get("text/plain").and_then(mime_value_to_string) {
        let text = text.trim_end_matches('\n');
        if !text.is_empty() {
            return Some(fenced_code("text", text));
        }
    }

    let json = serde_json::to_string_pretty(&Value::Object(data.clone())).ok()?;
    Some(fenced_code("json", &json))
}

fn escape_markdown_cell(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('|', "\\|")
        .replace("\r\n", "<br>")
        .replace(['\r', '\n'], "<br>")
}

fn fenced_code(language: &str, content: &str) -> String {
    let fence = code_fence(content);
    format!(
        "{fence}{language}\n{}\n{fence}",
        content.trim_end_matches('\n')
    )
}

fn code_fence(content: &str) -> String {
    let mut longest = 0;
    let mut run = 0;
    for character in content.chars() {
        if character == '`' {
            run += 1;
            longest = longest.max(run);
        } else {
            run = 0;
        }
    }
    "`".repeat(longest.max(2) + 1)
}

fn render_output(output: &Output) -> String {
    match output {
        Output::Text { text } => format!(
            "<div class=\"output output-text\"><pre>{}</pre></div>",
            escape_html(text)
        ),
        Output::Error {
            name,
            message,
            traceback,
        } => {
            let trace = traceback.join("\n");
            format!(
                "<div class=\"output output-error\"><strong>{}</strong><pre>{}</pre>{}</div>",
                escape_html(name),
                escape_html(message),
                if trace.is_empty() {
                    String::new()
                } else {
                    format!("<pre>{}</pre>", escape_html(&trace))
                }
            )
        }
        Output::Html { html } => render_html_output(html),
        Output::Image { mime_type, data } => render_image_output(mime_type, data),
        Output::Table { columns, rows } => {
            let headers = columns
                .iter()
                .map(|column| format!("<th>{}</th>", escape_html(column)))
                .collect::<String>();
            let rows = rows
                .iter()
                .map(|row| {
                    let cells = row
                        .iter()
                        .map(|value| format!("<td>{}</td>", escape_html(value)))
                        .collect::<String>();
                    format!("<tr>{cells}</tr>")
                })
                .collect::<String>();
            format!(
                "<div class=\"output\"><table><thead><tr>{headers}</tr></thead><tbody>{rows}</tbody></table></div>"
            )
        }
        Output::Latex { latex } => format!(
            "<div class=\"output math\" data-latex=\"{}\"><span class=\"math-text\">{}</span></div>",
            escape_html(latex),
            render_latex_static(latex)
        ),
        Output::Chart { spec } => {
            let encoded = escape_html(&spec.to_string());
            format!(
                "<div class=\"output chart\" data-chart=\"{encoded}\">{}</div>",
                render_chart_svg(spec)
            )
        }
        Output::MimeBundle { data, .. } => render_mime_bundle(data),
    }
}

fn render_mime_bundle(data: &serde_json::Map<String, Value>) -> String {
    for mime_type in [
        "image/svg+xml",
        "image/png",
        "image/jpeg",
        "image/gif",
        "image/webp",
    ] {
        if let Some(value) = data.get(mime_type).and_then(mime_value_to_string) {
            return render_image_output(mime_type, &value);
        }
    }

    if let Some(spec) = data
        .get("application/vnd.vega.v5+json")
        .or_else(|| data.get("application/vnd.vega-lite.v5+json"))
        .or_else(|| data.get("application/vnd.vega-lite+json"))
    {
        let encoded = escape_html(&spec.to_string());
        return format!(
            "<div class=\"output chart\" data-chart=\"{encoded}\">{}</div>",
            render_chart_svg(spec)
        );
    }

    if let Some(html) = data.get("text/html").and_then(mime_value_to_string) {
        return render_html_output(&html);
    }
    if let Some(latex) = data.get("text/latex").and_then(mime_value_to_string) {
        return format!(
            "<div class=\"output math\" data-latex=\"{}\"><span class=\"math-text\">{}</span></div>",
            escape_html(&latex),
            render_latex_static(&latex)
        );
    }
    if let Some(markdown) = data.get("text/markdown").and_then(mime_value_to_string) {
        return format!("<div class=\"output\">{}</div>", render_markdown(&markdown));
    }
    if let Some(text) = data.get("text/plain").and_then(mime_value_to_string) {
        return format!(
            "<div class=\"output output-text\"><pre>{}</pre></div>",
            escape_html(&text)
        );
    }

    format!(
        "<pre class=\"output\">{}</pre>",
        escape_html(&Value::Object(data.clone()).to_string())
    )
}

fn mime_value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => Some(value.clone()),
        Value::Array(parts) => {
            let mut text = String::new();
            for part in parts {
                text.push_str(part.as_str()?);
            }
            Some(text)
        }
        _ => None,
    }
}

fn render_html_output(html: &str) -> String {
    if let Some(svg) = extract_svg(html) {
        return render_image_output("image/svg+xml", svg);
    }
    format!("<div class=\"output\">{}</div>", ammonia::clean(html))
}

fn render_image_output(mime_type: &str, data: &str) -> String {
    if !is_safe_image_type(mime_type) {
        return "<div class=\"output output-error\">Unsupported image MIME type</div>".to_owned();
    }
    let Some(source) = image_data_uri(mime_type, data) else {
        return "<div class=\"output output-error\">Image data is empty</div>".to_owned();
    };
    format!(
        "<div class=\"output output-image\"><img src=\"{}\" alt=\"Notebook output\"></div>",
        escape_html(&source)
    )
}

fn image_data_uri(mime_type: &str, data: &str) -> Option<String> {
    let data = data.trim();
    if data.is_empty() {
        return None;
    }
    if data.starts_with("data:") {
        return Some(data.to_owned());
    }
    if mime_type == "image/svg+xml" {
        let payload = if data.starts_with('<') {
            STANDARD.encode(data.as_bytes())
        } else {
            data.to_owned()
        };
        return Some(format!("data:image/svg+xml;base64,{payload}"));
    }
    Some(format!("data:{mime_type};base64,{data}"))
}

fn extract_svg(html: &str) -> Option<&str> {
    let lowercase = html.to_ascii_lowercase();
    let start = lowercase.find("<svg")?;
    let open_end = start + lowercase[start..].find('>')? + 1;
    let end = if lowercase[start..open_end].trim_end().ends_with("/>") {
        open_end
    } else {
        let after_open = &lowercase[open_end..];
        open_end + after_open.find("</svg>")? + "</svg>".len()
    };

    // An SVG embedded in a larger HTML document (for example an icon inside a
    // table) is not a figure on its own. Only promote documents whose visible
    // content is the SVG itself.
    if has_visible_text(&html[..start]) || has_visible_text(&html[end..]) {
        return None;
    }

    Some(&html[start..end])
}

fn has_visible_text(fragment: &str) -> bool {
    let mut text = String::new();
    let mut in_tag = false;
    for character in fragment.chars() {
        match character {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => text.push(character),
            _ => {}
        }
    }
    !text.trim().is_empty()
}

fn render_chart_svg(spec: &Value) -> String {
    let values = spec
        .get("data")
        .and_then(|data| data.get("values"))
        .and_then(Value::as_array);
    let Some(values) = values else {
        return "<pre>Chart data is unavailable</pre>".to_owned();
    };
    let mark = spec.get("mark").and_then(Value::as_str).unwrap_or("point");
    let encoding = spec.get("encoding").and_then(Value::as_object);
    let x_field = encoding
        .and_then(|value| value.get("x"))
        .and_then(|value| value.get("field"))
        .and_then(Value::as_str);
    let y_field = encoding
        .and_then(|value| value.get("y"))
        .and_then(|value| value.get("field"))
        .and_then(Value::as_str);
    let Some(y_field) = y_field else {
        return "<pre>Chart requires an encoded y field</pre>".to_owned();
    };
    let y_values = values
        .iter()
        .filter_map(|value| value.get(y_field).and_then(Value::as_f64))
        .collect::<Vec<_>>();
    if y_values.is_empty() {
        return "<pre>Chart data has no numeric y values</pre>".to_owned();
    }
    let max = y_values.iter().copied().fold(0.0_f64, f64::max).max(1.0);
    let width = 720.0;
    let height = 240.0;
    let left = 42.0;
    let bottom = 24.0;
    let plot_width = width - left - 12.0;
    let plot_height = height - bottom - 12.0;
    let points = y_values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let x = left + (index as f64 / y_values.len().max(1) as f64) * plot_width;
            let y = 12.0 + plot_height - (value / max) * plot_height;
            (x, y, value)
        })
        .collect::<Vec<_>>();
    let shapes = match mark {
        "bar" => points
            .iter()
            .map(|(x, y, value)| {
                let bar_width = (plot_width / y_values.len().max(1) as f64 * 0.7).max(2.0);
                format!(
                    "<rect x=\"{x:.1}\" y=\"{y:.1}\" width=\"{bar_width:.1}\" height=\"{height:.1}\" fill=\"#f97316\"><title>{value}</title></rect>",
                    height = 12.0 + plot_height - y,
                )
            })
            .collect::<String>(),
        "line" | "point" => {
            let path = points
                .iter()
                .map(|(x, y, _)| format!("{x:.1},{y:.1}"))
                .collect::<Vec<_>>()
                .join(" ");
            let line = if mark == "line" {
                format!("<polyline points=\"{path}\" fill=\"none\" stroke=\"#fb923c\" stroke-width=\"3\"/>")
            } else {
                String::new()
            };
            let circles = points
                .iter()
                .map(|(x, y, value)| {
                    format!("<circle cx=\"{x:.1}\" cy=\"{y:.1}\" r=\"5\" fill=\"#f97316\"><title>{value}</title></circle>")
                })
                .collect::<String>();
            format!("{line}{circles}")
        }
        _ => "<text x=\"42\" y=\"30\" fill=\"#fca5a5\">Unsupported chart mark</text>".to_owned(),
    };
    let x_label = x_field.map_or(String::new(), |field| {
        format!(
            "<text x=\"{left}\" y=\"{height}\" fill=\"#a3a09b\">{}</text>",
            escape_html(field)
        )
    });
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" class=\"chart-svg\" viewBox=\"0 0 {width} {height}\" role=\"img\" aria-label=\"Offline chart\"><line x1=\"{left}\" y1=\"12\" x2=\"{left}\" y2=\"{plot_bottom}\" stroke=\"#737373\"/><line x1=\"{left}\" y1=\"{plot_bottom}\" x2=\"{width}\" y2=\"{plot_bottom}\" stroke=\"#737373\"/>{shapes}{x_label}</svg>",
        plot_bottom = 12.0 + plot_height,
    )
}

fn is_safe_image_type(mime_type: &str) -> bool {
    matches!(
        mime_type,
        "image/png" | "image/jpeg" | "image/gif" | "image/webp" | "image/svg+xml"
    )
}

fn render_latex_static(latex: &str) -> String {
    let mut rendered = String::new();
    let mut cursor = 0;
    while let Some(relative) = latex[cursor..].find("\\frac{") {
        let start = cursor + relative;
        rendered.push_str(&escape_html(&latex[cursor..start]));
        let numerator_start = start + "\\frac{".len();
        let Some((numerator, after_numerator)) = braced_group(latex, numerator_start) else {
            rendered.push_str(&escape_html(&latex[start..]));
            return rendered;
        };
        let denominator_start = after_numerator + 1;
        if !latex[denominator_start..].starts_with('{') {
            rendered.push_str(&escape_html(&latex[start..]));
            return rendered;
        }
        let Some((denominator, after_denominator)) = braced_group(latex, denominator_start + 1)
        else {
            rendered.push_str(&escape_html(&latex[start..]));
            return rendered;
        };
        rendered.push_str(&format!(
            "<span class=\"math-frac\"><span>{}</span><span>{}</span></span>",
            escape_html(numerator),
            escape_html(denominator)
        ));
        cursor = after_denominator + 1;
    }
    rendered.push_str(&escape_html(&latex[cursor..]));
    rendered
}

fn braced_group(source: &str, start: usize) -> Option<(&str, usize)> {
    let bytes = source.as_bytes();
    let mut depth = 1;
    for index in start..bytes.len() {
        match bytes[index] {
            b'{' => depth += 1,
            b'}' if depth == 1 => return Some((&source[start..index], index)),
            b'}' if depth > 1 => depth -= 1,
            _ => {}
        }
    }
    None
}

pub(crate) fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use mars_document::{Cell, Notebook};

    #[test]
    fn renders_markdown_and_escapes_code() {
        let mut notebook = Notebook::new();
        notebook
            .add_cell(Cell::new(CellType::Markdown, "# Hello\n\n**Mars**"))
            .expect("unique ID");
        notebook
            .add_cell(Cell::new(CellType::Code, "let x = \"<unsafe>\";"))
            .expect("unique ID");

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
        notebook.add_cell(cell).expect("unique ID");

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
        notebook.add_cell(cell).expect("unique ID");

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
        notebook.add_cell(cell).expect("unique ID");

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
        notebook.add_cell(cell).expect("unique ID");

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
            metadata: serde_json::Map::new(),
        });
        notebook.add_cell(cell).expect("unique ID");

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
            metadata: serde_json::Map::new(),
        });
        notebook.add_cell(cell).expect("unique ID");

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
        notebook.add_cell(cell).expect("unique ID");

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
        notebook.add_cell(cell).expect("unique ID");

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
        notebook.add_cell(cell).expect("unique ID");

        let html = render_notebook(&notebook);

        assert!(html.contains("data:image/svg+xml;base64,"));
    }

    #[test]
    fn exports_markdown_with_cells_and_outputs() {
        let mut notebook = Notebook::new();
        notebook.metadata.title = Some("Demo notebook".to_owned());
        notebook
            .add_cell(Cell::new(CellType::Markdown, "# Hello\n\n**Mars**"))
            .expect("unique ID");
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
        notebook.add_cell(cell).expect("unique ID");

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
            .expect("unique ID");

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
            metadata: serde_json::Map::new(),
        });
        notebook.add_cell(cell).expect("unique ID");

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
        notebook.add_cell(cell).expect("unique ID");

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
        notebook.add_cell(cell).expect("unique ID");

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
            .expect("unique ID");

        let markdown = render_markdown_notebook(&notebook);

        assert!(markdown.contains("````rust\nlet fence = \"```\";\n````"));
    }
}
