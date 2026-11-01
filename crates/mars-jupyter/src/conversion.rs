use mars_core::Output;
use serde_json::{Map, Value};

pub fn convert_output(output: super::import::JupyterOutput) -> Option<Output> {
    match output {
        super::import::JupyterOutput::Stream { text, .. } => Some(Output::Text {
            text: text.into_string(),
        }),
        super::import::JupyterOutput::Error {
            ename,
            evalue,
            traceback,
        } => Some(Output::Error {
            name: ename,
            message: evalue,
            traceback,
        }),
        super::import::JupyterOutput::DisplayData { data, metadata }
        | super::import::JupyterOutput::ExecuteResult { data, metadata } => {
            convert_mime_bundle(data, metadata)
        }
    }
}

pub fn convert_mime_bundle(
    mut data: Map<String, Value>,
    metadata: Map<String, Value>,
) -> Option<Output> {
    if data.len() > 1 || !metadata.is_empty() {
        return Some(Output::MimeBundle { data, metadata });
    }
    if let Some(Value::String(html)) = data.remove("text/html") {
        return Some(Output::Html { html });
    }
    if let Some(Value::String(text)) = data.remove("text/plain") {
        return Some(Output::Text { text });
    }
    if let Some(Value::String(latex)) = data.remove("text/latex") {
        return Some(Output::Latex { latex });
    }
    if let Some(spec) = data
        .remove("application/vnd.vega.v5+json")
        .or_else(|| data.remove("application/vnd.vega-lite.v5+json"))
        .or_else(|| data.remove("application/vnd.vega-lite+json"))
    {
        return Some(Output::Chart { spec });
    }
    for (mime_type, value) in data.iter() {
        if matches!(
            mime_type.as_str(),
            "image/png" | "image/jpeg" | "image/svg+xml"
        ) {
            if let Some(data) = value.as_str() {
                return Some(Output::Image {
                    mime_type: mime_type.clone(),
                    data: data.to_owned(),
                });
            }
        }
    }
    if data.is_empty() {
        None
    } else {
        Some(Output::MimeBundle { data, metadata })
    }
}

pub fn table_to_html(columns: &[String], rows: &[Vec<String>]) -> String {
    let headers = columns
        .iter()
        .map(|column| format!("<th>{}</th>", escape_html(column)))
        .collect::<String>();
    let body = rows
        .iter()
        .map(|row| {
            let cells = row
                .iter()
                .map(|value| format!("<td>{}</td>", escape_html(value)))
                .collect::<String>();
            format!("<tr>{cells}</tr>")
        })
        .collect::<String>();
    format!("<table><thead><tr>{headers}</tr></thead><tbody>{body}</tbody></table>")
}

pub fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
