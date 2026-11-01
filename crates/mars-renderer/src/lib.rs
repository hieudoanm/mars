use mars_core::{CellType, Output};
use mars_document::Notebook;
use pulldown_cmark::{Options, Parser, html};

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

fn render_markdown(source: &str) -> String {
    let mut rendered = String::new();
    let options = Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH;
    html::push_html(&mut rendered, Parser::new_ext(source, options));
    ammonia::clean(&rendered)
}

fn render_output(output: &Output) -> String {
    return match output {
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
        Output::Html { html } => {
            format!("<div class=\"output\">{}</div>", ammonia::clean(html))
        }
        Output::Image { mime_type, data } if is_safe_image_type(mime_type) => format!(
            "<div class=\"output\"><img src=\"data:{};base64,{}\" alt=\"Notebook output\"></div>",
            escape_html(mime_type),
            escape_html(data)
        ),
        Output::Image { .. } => {
            "<div class=\"output output-error\">Unsupported image MIME type</div>".to_owned()
        }
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
        Output::MimeBundle { data, .. } => format!(
            "<pre class=\"output\">{}</pre>",
            escape_html(&serde_json::Value::Object(data.clone()).to_string())
        ),
    };

    fn render_chart_svg(spec: &serde_json::Value) -> String {
        let values = spec
            .get("data")
            .and_then(|data| data.get("values"))
            .and_then(serde_json::Value::as_array);
        let Some(values) = values else {
            return "<pre>Chart data is unavailable</pre>".to_owned();
        };
        let mark = spec
            .get("mark")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("point");
        let encoding = spec.get("encoding").and_then(serde_json::Value::as_object);
        let x_field = encoding
            .and_then(|value| value.get("x"))
            .and_then(|value| value.get("field"))
            .and_then(serde_json::Value::as_str);
        let y_field = encoding
            .and_then(|value| value.get("y"))
            .and_then(|value| value.get("field"))
            .and_then(serde_json::Value::as_str);
        let Some(y_field) = y_field else {
            return "<pre>Chart requires an encoded y field</pre>".to_owned();
        };
        let y_values = values
            .iter()
            .filter_map(|value| value.get(y_field).and_then(serde_json::Value::as_f64))
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
            "<svg class=\"chart-svg\" viewBox=\"0 0 {width} {height}\" role=\"img\" aria-label=\"Offline chart\"><line x1=\"{left}\" y1=\"12\" x2=\"{left}\" y2=\"{plot_bottom}\" stroke=\"#737373\"/><line x1=\"{left}\" y1=\"{plot_bottom}\" x2=\"{width}\" y2=\"{plot_bottom}\" stroke=\"#737373\"/>{shapes}{x_label}</svg>",
            plot_bottom = 12.0 + plot_height,
        )
    }
}

fn is_safe_image_type(mime_type: &str) -> bool {
    matches!(mime_type, "image/png" | "image/jpeg" | "image/svg+xml")
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

fn escape_html(value: &str) -> String {
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
}
