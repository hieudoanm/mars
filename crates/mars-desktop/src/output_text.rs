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
