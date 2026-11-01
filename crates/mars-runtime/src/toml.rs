pub(crate) fn toml_dependency_value(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.starts_with('{') && trimmed.ends_with('}') && !trimmed.contains('\n') {
        Some(trimmed)
    } else {
        None
    }
}

pub(crate) fn toml_string(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}
