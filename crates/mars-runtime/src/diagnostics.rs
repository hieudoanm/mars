pub fn format_compiler_diagnostics(stderr: &[u8]) -> String {
    let raw = String::from_utf8_lossy(stderr);
    let mut diagnostics = Vec::new();
    for line in raw.lines() {
        let Ok(value) = serde_json::from_str::<serde_json::Value>(line) else {
            diagnostics.push(line.to_owned());
            continue;
        };
        if value
            .get("reason")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|reason| reason != "compiler-message")
            && value.get("message").is_none()
        {
            continue;
        }
        let Some(message) = value.get("message").or(Some(&value)) else {
            continue;
        };
        let level = message
            .get("level")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("error");
        let text = message
            .get("message")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("compiler diagnostic");
        if let Some(code) = message
            .get("code")
            .and_then(|code| code.get("code"))
            .and_then(serde_json::Value::as_str)
        {
            diagnostics.push(format!("{level} [{code}]: {text}"));
        } else {
            diagnostics.push(format!("{level}: {text}"));
        }
        if let Some(spans) = message.get("spans").and_then(serde_json::Value::as_array) {
            for span in spans {
                let kind =
                    if span.get("is_primary").and_then(serde_json::Value::as_bool) == Some(true) {
                        "primary"
                    } else {
                        "secondary"
                    };
                let file = span
                    .get("file_name")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("source");
                let line_start = span
                    .get("line_start")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0);
                let column_start = span
                    .get("column_start")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0);
                let line_end = span
                    .get("line_end")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(line_start);
                let column_end = span
                    .get("column_end")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(column_start);
                let label = span
                    .get("label")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("");
                diagnostics.push(format!(
                    "  --> {file}:{line_start}:{column_start}-{line_end}:{column_end} [{kind}] {label}"
                ));
            }
        }
        if let Some(rendered) = message.get("rendered").and_then(serde_json::Value::as_str) {
            let rendered = rendered.trim();
            if !rendered.is_empty() && rendered != text {
                diagnostics.push(format!("  note: {rendered}"));
            }
        }
    }
    if diagnostics.is_empty() {
        raw.into_owned()
    } else {
        diagnostics.join("\n")
    }
}
