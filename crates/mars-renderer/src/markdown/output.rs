use crate::chart::render_chart_svg;
use crate::markdown::fence::{escape_markdown_cell, fenced_code};
use crate::util::{image_data_uri, is_safe_image_type, mime_value_to_string};
use mars_core::Output;
use serde_json::Value;

pub(crate) fn render_markdown_output(output: &Output) -> Option<String> {
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
    if let Some(svg) = crate::util::extract_svg(html) {
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
