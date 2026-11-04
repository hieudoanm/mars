use mars_core::Output;
use mars_runtime::ExecutionResult;

pub fn print_execution_result(index: usize, result: &ExecutionResult) {
    let failed = result
        .outputs
        .iter()
        .any(|output| matches!(output, Output::Error { .. }));
    let outcome = if result.interrupted {
        "interrupted"
    } else if failed {
        "failed"
    } else {
        "succeeded"
    };
    println!(
        "Cell {} {} in {} ms (execution {}, {} output{})",
        index + 1,
        outcome,
        result.duration.as_millis(),
        result.execution_count,
        result.outputs.len(),
        if result.outputs.len() == 1 { "" } else { "s" }
    );
    for output in &result.outputs {
        match output {
            Output::Text { text } => {
                println!("  stdout:");
                print_indented(text);
            }
            Output::Error { name, message, .. } => {
                eprintln!("  {name}:");
                print_indented_error(message);
            }
            Output::Html { .. } => println!("  html output"),
            Output::Image { mime_type, data } => println!(
                "  image output ({mime_type}, {})",
                format_image_size(mime_type, data)
            ),
            Output::Table { columns, rows } => println!(
                "  table output ({} columns, {} rows)",
                columns.len(),
                rows.len()
            ),
            Output::Latex { .. } => println!("  LaTeX output"),
            Output::Chart { .. } => println!("  chart output"),
            Output::MimeBundle { data, .. } => {
                if let Some((mime_type, value)) = preferred_image(data) {
                    println!(
                        "  image output ({mime_type}, {})",
                        format_image_size(mime_type, value)
                    );
                } else {
                    let types = data.keys().cloned().collect::<Vec<_>>().join(", ");
                    println!("  MIME bundle output ({types})");
                }
            }
        }
    }
}

fn preferred_image(data: &serde_json::Map<String, serde_json::Value>) -> Option<(&str, &str)> {
    [
        "image/svg+xml",
        "image/png",
        "image/jpeg",
        "image/gif",
        "image/webp",
    ]
    .into_iter()
    .find_map(|mime_type| {
        data.get(mime_type)
            .and_then(serde_json::Value::as_str)
            .map(|value| (mime_type, value))
    })
}

fn format_image_size(mime_type: &str, data: &str) -> String {
    let bytes = if mime_type == "image/svg+xml" && data.trim_start().starts_with('<') {
        data.len()
    } else {
        data.trim().len().saturating_mul(3) / 4
    };
    if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{bytes} B")
    }
}

pub fn print_indented(text: &str) {
    for line in text.lines() {
        println!("    {line}");
    }
}

pub fn print_indented_error(text: &str) {
    for line in text.lines() {
        eprintln!("    {line}");
    }
}
