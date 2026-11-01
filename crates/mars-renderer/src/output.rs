use crate::chart::render_chart_svg;
use crate::latex::render_latex_static;
use crate::markdown::render_markdown;
use crate::util::{extract_svg, image_data_uri, is_safe_image_type, mime_value_to_string};
use mars_core::Output;
use serde_json::Value;

pub fn render_output(output: &Output) -> String {
    match output {
        Output::Text { text } => format!(
            "<div class=\"output output-text\"><pre>{}</pre></div>",
            crate::util::escape_html(text)
        ),
        Output::Error {
            name,
            message,
            traceback,
        } => {
            let trace = traceback.join("\n");
            format!(
                "<div class=\"output output-error\"><strong>{}</strong><pre>{}</pre>{}</div>",
                crate::util::escape_html(name),
                crate::util::escape_html(message),
                if trace.is_empty() {
                    String::new()
                } else {
                    format!("<pre>{}</pre>", crate::util::escape_html(&trace))
                }
            )
        }
        Output::Html { html } => render_html_output(html),
        Output::Image { mime_type, data } => render_image_output(mime_type, data),
        Output::Table { columns, rows } => {
            let headers = columns
                .iter()
                .map(|column| format!("<th>{}</th>", crate::util::escape_html(column)))
                .collect::<String>();
            let rows = rows
                .iter()
                .map(|row| {
                    let cells = row
                        .iter()
                        .map(|value| format!("<td>{}</td>", crate::util::escape_html(value)))
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
            crate::util::escape_html(latex),
            render_latex_static(latex)
        ),
        Output::Chart { spec } => {
            let encoded = crate::util::escape_html(&spec.to_string());
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
        let encoded = crate::util::escape_html(&spec.to_string());
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
            crate::util::escape_html(&latex),
            render_latex_static(&latex)
        );
    }
    if let Some(markdown) = data.get("text/markdown").and_then(mime_value_to_string) {
        return format!("<div class=\"output\">{}</div>", render_markdown(&markdown));
    }
    if let Some(text) = data.get("text/plain").and_then(mime_value_to_string) {
        return format!(
            "<div class=\"output output-text\"><pre>{}</pre></div>",
            crate::util::escape_html(&text)
        );
    }

    format!(
        "<pre class=\"output\">{}</pre>",
        crate::util::escape_html(&Value::Object(data.clone()).to_string())
    )
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
        crate::util::escape_html(&source)
    )
}
