use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde_json::Value;

pub fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub fn image_data_uri(mime_type: &str, data: &str) -> Option<String> {
    let data = data.trim();
    if data.is_empty() {
        return None;
    }
    if data.starts_with("data:") {
        return Some(data.to_owned());
    }
    if mime_type == "image/svg+xml" {
        let payload = if data.starts_with('<') {
            STANDARD.encode(data.as_bytes())
        } else {
            data.to_owned()
        };
        return Some(format!("data:image/svg+xml;base64,{payload}"));
    }
    Some(format!("data:{mime_type};base64,{data}"))
}

pub fn is_safe_image_type(mime_type: &str) -> bool {
    matches!(
        mime_type,
        "image/png" | "image/jpeg" | "image/gif" | "image/webp" | "image/svg+xml"
    )
}

pub fn extract_svg(html: &str) -> Option<&str> {
    let lowercase = html.to_ascii_lowercase();
    let start = lowercase.find("<svg")?;
    let open_end = start + lowercase[start..].find('>')? + 1;
    let end = if lowercase[start..open_end].trim_end().ends_with("/>") {
        open_end
    } else {
        let after_open = &lowercase[open_end..];
        open_end + after_open.find("</svg>")? + "</svg>".len()
    };

    if has_visible_text(&html[..start]) || has_visible_text(&html[end..]) {
        return None;
    }

    Some(&html[start..end])
}

pub fn has_visible_text(fragment: &str) -> bool {
    let mut text = String::new();
    let mut in_tag = false;
    for character in fragment.chars() {
        match character {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => text.push(character),
            _ => {}
        }
    }
    !text.trim().is_empty()
}

pub fn mime_value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => Some(value.clone()),
        Value::Array(parts) => {
            let mut text = String::new();
            for part in parts {
                text.push_str(part.as_str()?);
            }
            Some(text)
        }
        _ => None,
    }
}

/// Rewrites `attachment:<name>` references in markdown/HTML source to the
/// data URLs stored in `cell.metadata.attachments`. Covers markdown image
/// syntax `![](attachment:name)`, `<img src="attachment:name">`, and
/// `<a href="attachment:name">`.
pub fn resolve_attachment_refs(
    source: &str,
    metadata: &serde_json::Map<String, serde_json::Value>,
) -> String {
    let Some(attachments) = metadata.get("attachments").and_then(|v| v.as_object()) else {
        return source.to_owned();
    };
    let mut resolved = source.to_owned();
    for (name, value) in attachments {
        let Some(data_url) = value.as_str().filter(|s| s.starts_with("data:")) else {
            continue;
        };
        for wrapper in [
            format!("](attachment:{name})"),
            format!("src=\"attachment:{name}\""),
            format!("href=\"attachment:{name}\""),
        ] {
            let replacement = if wrapper.starts_with("](") {
                format!("]({data_url})")
            } else {
                wrapper.replacen(&format!("attachment:{name}"), data_url, 1)
            };
            resolved = resolved.replace(&wrapper, &replacement);
        }
    }
    resolved
}
