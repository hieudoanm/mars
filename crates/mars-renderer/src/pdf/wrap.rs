pub fn wrap_text(text: &str, max_width: f64, _font: &'static str, font_size: f64) -> Vec<String> {
    let avg_char_w = font_size * 0.5;
    let max_chars = (max_width / avg_char_w).floor() as usize;
    if max_chars == 0 {
        return vec![text.to_owned()];
    }

    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        if paragraph.is_empty() {
            lines.push(String::new());
            continue;
        }
        let mut remaining = paragraph.to_owned();
        while remaining.len() > max_chars {
            let split_at = remaining
                .char_indices()
                .take(max_chars)
                .last()
                .map(|(i, _)| i)
                .unwrap_or(max_chars);
            lines.push(remaining[..split_at].to_owned());
            remaining = remaining[split_at..].to_owned();
        }
        if !remaining.is_empty() {
            lines.push(remaining);
        }
    }
    lines
}
