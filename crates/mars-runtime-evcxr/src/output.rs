use crossbeam_channel::Receiver;
use mars_core::Output;
use mars_runtime::extract_standalone_svg;
use serde_json::{Map, Value};

pub fn drain_channel(receiver: &Receiver<String>) -> String {
    let lines = receiver.try_iter().collect::<Vec<_>>();
    if lines.is_empty() {
        String::new()
    } else {
        format!("{}\n", lines.join("\n"))
    }
}

pub fn append_mime_output(outputs: &mut Vec<Output>, mut data: Map<String, Value>) {
    if data.is_empty() {
        return;
    }
    promote_html_svg(&mut data);
    if data.len() == 1 {
        if let Some(Value::String(text)) = data.remove("text/plain") {
            outputs.push(Output::Text { text });
            return;
        }
        if let Some(Value::String(html)) = data.remove("text/html") {
            outputs.push(Output::Html { html });
            return;
        }
    }
    outputs.push(Output::MimeBundle {
        data,
        metadata: Map::new(),
    });
}

pub fn promote_html_svg(data: &mut Map<String, Value>) {
    if data.keys().any(|mime_type| mime_type.starts_with("image/")) {
        return;
    }
    let Some(Value::String(html)) = data.get("text/html") else {
        return;
    };
    let Some(svg) = extract_standalone_svg(html) else {
        return;
    };
    let svg = svg.to_owned();
    data.remove("text/html");
    data.insert("image/svg+xml".to_owned(), Value::String(svg));
    data.entry("text/plain".to_owned())
        .or_insert_with(|| Value::String("Figure (SVG)".to_owned()));
}
