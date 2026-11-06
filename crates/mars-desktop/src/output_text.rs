use mars_core::Output;

/// Flatten a kernel `Output` into displayable plain-text lines.
///
/// Rich outputs (HTML, images, charts, mime bundles) are represented by a
/// short placeholder until the GPUI renderer grows rich-output widgets.
pub fn output_lines(output: &Output) -> Vec<String> {
    match output {
        Output::Text { text } => text.lines().map(str::to_owned).collect(),
        Output::Error {
            name,
            message,
            traceback,
        } => {
            let mut lines = vec![format!("{name}: {message}")];
            lines.extend(traceback.iter().cloned());
            lines
        }
        Output::Html { .. } => vec!["[HTML output]".to_owned()],
        Output::Image { mime_type, .. } => vec![format!("[image: {mime_type}]")],
        Output::Table { columns, rows } => {
            let mut lines = vec![columns.join(" | ")];
            lines.extend(rows.iter().map(|row| row.join(" | ")));
            lines
        }
        Output::Latex { latex } => vec![format!("[latex] {latex}")],
        Output::Chart { .. } => vec!["[chart]".to_owned()],
        Output::MimeBundle { data, .. } => {
            let mut keys = data.keys().cloned().collect::<Vec<_>>();
            keys.sort();
            vec![format!("[mime bundle: {}]", keys.join(", "))]
        }
    }
}

/// Strip an HTML output down to readable text for the plain-text fallback.
pub fn html_text(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(start) = rest.find('<') {
        out.push_str(&rest[..start]);
        let after = &rest[start..];
        let Some(end) = after.find('>') else {
            rest = "";
            break;
        };
        let tag = &after[..=end];
        let lower = tag.to_ascii_lowercase();
        if lower.starts_with("<script") || lower.starts_with("<style") {
            // Drop the whole element including content.
            let close = format!(
                "</{}",
                &lower[1..lower.find(' ').unwrap_or(lower.len() - 1)]
            );
            if let Some(close_pos) = rest[start..].to_ascii_lowercase().find(&close) {
                rest = &rest[start + close_pos + close.len()..];
                continue;
            }
        }
        // Insert a space for block-ish tags to keep words separated.
        if lower.starts_with("<br")
            || lower.starts_with("<p")
            || lower.starts_with("<div")
            || lower.starts_with("<li")
            || lower.starts_with("</")
        {
            out.push(' ');
        }
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    out = out
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'");
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}
