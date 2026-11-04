use mars_core::Output;

pub fn render_output_to_pdf_lines(output: &Output, max_width: f64) -> Vec<String> {
    match output {
        Output::Text { text } => {
            let trimmed = text.trim_end_matches('\n');
            if trimmed.is_empty() {
                return Vec::new();
            }
            crate::pdf::wrap::wrap_text(trimmed, max_width, "/F1", 8.5)
        }
        Output::Error {
            name,
            message,
            traceback,
        } => {
            let mut lines = vec![format!("Error: {name}")];
            lines.extend(wrap_text(message, max_width, "/F1", 8.5));
            for trace in traceback {
                lines.extend(wrap_text(trace, max_width, "/F1", 8.5));
            }
            lines
        }
        Output::Html { html } => {
            let mut text = String::new();
            let mut in_tag = false;
            for ch in html.chars() {
                match ch {
                    '<' => in_tag = true,
                    '>' => in_tag = false,
                    _ if !in_tag => text.push(ch),
                    _ => {}
                }
            }
            let text = text.trim();
            if text.is_empty() {
                Vec::new()
            } else {
                wrap_text(text, max_width, "/F1", 8.5)
            }
        }
        Output::Image { mime_type, data } => {
            vec![format!("[IMAGE: {mime_type}, {} bytes]", data.len())]
        }
        Output::Table { columns, rows } => {
            let mut lines = Vec::new();
            let header = columns.join(" | ");
            lines.push(header);
            lines.push(
                columns
                    .iter()
                    .map(|_| "---")
                    .collect::<Vec<_>>()
                    .join(" | "),
            );
            for row in rows {
                lines.push(row.join(" | "));
            }
            lines
        }
        Output::Latex { latex } => {
            vec![format!("[LaTeX: {latex}]")]
        }
        Output::Chart { spec } => {
            let svg = crate::chart::render_chart_svg(spec);
            vec![format!("[Chart SVG: {} bytes]", svg.len())]
        }
        Output::MimeBundle { data, .. } => {
            for mime_type in [
                "image/svg+xml",
                "image/png",
                "image/jpeg",
                "image/gif",
                "image/webp",
            ] {
                if let Some(value) = data.get(mime_type).and_then(|v| v.as_str()) {
                    return vec![format!("[IMAGE: {mime_type}, {} bytes]", value.len())];
                }
            }
            if let Some(spec) = data
                .get("application/vnd.vega.v5+json")
                .or_else(|| data.get("application/vnd.vega-lite.v5+json"))
                .or_else(|| data.get("application/vnd.vega-lite+json"))
            {
                let svg = crate::chart::render_chart_svg(spec);
                return vec![format!("[Chart: {} bytes]", svg.len())];
            }
            if let Some(text) = data.get("text/plain").and_then(|v| v.as_str()) {
                return wrap_text(text, max_width, "/F1", 8.5);
            }
            vec!["[MIME bundle output]".to_owned()]
        }
    }
}

fn wrap_text(text: &str, max_width: f64, font: &'static str, font_size: f64) -> Vec<String> {
    crate::pdf::wrap::wrap_text(text, max_width, font, font_size)
}
