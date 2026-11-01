use serde_json::Value;

/// Extracts the diagnostic object from a rustc/cargo JSON line.
///
/// Current rustc emits flat diagnostics (`{"$message_type":"diagnostic", ...}`),
/// cargo wraps them as `{"reason":"compiler-message","message":{...}}`, and some
/// tools emit bare diagnostic objects. Anything else (artifacts, build events,
/// human-readable cargo lines handled by the caller) yields `None`.
fn diagnostic_object(value: &Value) -> Option<&Value> {
    if value.get("$message_type").and_then(Value::as_str) == Some("diagnostic") {
        return Some(value);
    }
    if value.get("reason").and_then(Value::as_str) == Some("compiler-message") {
        return value.get("message").filter(|message| message.is_object());
    }
    if value.get("message").and_then(Value::as_str).is_some() && value.get("level").is_some() {
        return Some(value);
    }
    None
}

fn format_structured(diagnostic: &Value, diagnostics: &mut Vec<String>) {
    let level = diagnostic
        .get("level")
        .and_then(Value::as_str)
        .unwrap_or("error");
    let text = diagnostic
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or("compiler diagnostic");
    if let Some(code) = diagnostic
        .get("code")
        .and_then(|code| code.get("code"))
        .and_then(Value::as_str)
    {
        diagnostics.push(format!("{level} [{code}]: {text}"));
    } else {
        diagnostics.push(format!("{level}: {text}"));
    }
    if let Some(spans) = diagnostic.get("spans").and_then(Value::as_array) {
        for span in spans {
            let kind = if span.get("is_primary").and_then(Value::as_bool) == Some(true) {
                "primary"
            } else {
                "secondary"
            };
            let file = span
                .get("file_name")
                .and_then(Value::as_str)
                .unwrap_or("source");
            let line_start = span.get("line_start").and_then(Value::as_u64).unwrap_or(0);
            let column_start = span
                .get("column_start")
                .and_then(Value::as_u64)
                .unwrap_or(0);
            let line_end = span
                .get("line_end")
                .and_then(Value::as_u64)
                .unwrap_or(line_start);
            let column_end = span
                .get("column_end")
                .and_then(Value::as_u64)
                .unwrap_or(column_start);
            let label = span.get("label").and_then(Value::as_str).unwrap_or("");
            diagnostics.push(format!(
                "  --> {file}:{line_start}:{column_start}-{line_end}:{column_end} [{kind}] {label}"
            ));
        }
    }
    if let Some(children) = diagnostic.get("children").and_then(Value::as_array) {
        for child in children {
            let child_level = child.get("level").and_then(Value::as_str).unwrap_or("note");
            if let Some(child_message) = child.get("message").and_then(Value::as_str) {
                diagnostics.push(format!("  {child_level}: {child_message}"));
            }
        }
    }
}

pub fn format_compiler_diagnostics(stderr: &[u8]) -> String {
    let raw = String::from_utf8_lossy(stderr);
    let mut diagnostics = Vec::new();
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(value) = serde_json::from_str::<serde_json::Value>(line) else {
            diagnostics.push(line.to_owned());
            continue;
        };
        let Some(diagnostic) = diagnostic_object(&value) else {
            continue;
        };
        // rustc's own rendered output is the most faithful representation and
        // already includes children notes and span snippets.
        if let Some(rendered) = diagnostic.get("rendered").and_then(Value::as_str) {
            let rendered = rendered.trim();
            if !rendered.is_empty() {
                diagnostics.push(rendered.to_owned());
                continue;
            }
        }
        format_structured(diagnostic, &mut diagnostics);
    }
    if diagnostics.is_empty() {
        raw.into_owned()
    } else {
        diagnostics.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_flat_rustc_diagnostics() {
        let stderr = br#"{"$message_type":"diagnostic","message":"mismatched types","code":{"code":"E0308"},"level":"error","spans":[{"file_name":"main.rs","line_start":2,"column_start":18,"line_end":2,"column_end":24,"is_primary":true,"label":null}],"children":[],"rendered":"error[E0308]: mismatched types\n --> main.rs:2:18\n"}
{"$message_type":"diagnostic","message":"aborting due to 1 previous error","level":"error","spans":[],"children":[],"rendered":"error: aborting due to 1 previous error\n"}"#;
        let output = format_compiler_diagnostics(stderr);
        assert!(output.contains("error[E0308]: mismatched types"));
        assert!(output.contains("--> main.rs:2:18"));
        assert!(output.contains("aborting due to 1 previous error"));
        assert!(!output.contains("compiler diagnostic"));
    }

    #[test]
    fn formats_wrapped_cargo_diagnostics() {
        let stderr = br#"{"reason":"compiler-message","message":{"message":"cannot find value `x`","level":"error","code":{"code":"E0425"},"spans":[],"children":[],"rendered":null}}"#;
        let output = format_compiler_diagnostics(stderr);
        assert!(output.contains("error [E0425]: cannot find value `x`"));
    }

    #[test]
    fn falls_back_to_structured_output_without_rendered() {
        let stderr = br#"{"$message_type":"diagnostic","message":"some failure","level":"warning","spans":[{"file_name":"a.rs","line_start":1,"column_start":1,"line_end":1,"column_end":2,"is_primary":true,"label":"here"}],"children":[{"level":"note","message":"a note"}]}"#;
        let output = format_compiler_diagnostics(stderr);
        assert!(output.contains("warning: some failure"));
        assert!(output.contains("--> a.rs:1:1-1:2 [primary] here"));
        assert!(output.contains("note: a note"));
    }

    #[test]
    fn skips_non_diagnostic_json_and_keeps_plain_text() {
        let stderr = br#"{"reason":"build-finished","success":false}
{"reason":"compiler-artifact","package_id":"x"}
error: could not compile `demo`"#;
        let output = format_compiler_diagnostics(stderr);
        assert_eq!(output, "error: could not compile `demo`");
    }

    #[test]
    fn returns_raw_output_when_nothing_parses() {
        let stderr = b"plain rustc failure\nwith details";
        assert_eq!(
            format_compiler_diagnostics(stderr),
            "plain rustc failure\nwith details"
        );
    }
}
