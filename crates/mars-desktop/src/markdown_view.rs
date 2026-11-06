use std::ops::Range;

use gpui::{FontStyle, FontWeight, HighlightStyle, StyledText, UnderlineStyle, px, rgb};

use crate::app::{ACCENT, CODE_BG, MUTED, TEXT};

/// Render a Markdown cell's source as styled text for preview mode.
///
/// Supports headings, `**bold**`, `*italic*`, `` `inline code` ``, and fenced
/// code blocks. Everything else (tables, lists, links, HTML) renders as plain
/// text — this is a preview, not a full CommonMark implementation.
pub fn markdown_preview(source: &str) -> StyledText {
    let mut text = String::new();
    let mut highlights: Vec<(Range<usize>, HighlightStyle)> = Vec::new();
    let mut in_code_block = false;

    for line in source.lines() {
        let line_start = text.len();
        let trimmed = line.trim_start();

        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            text.push('\n');
            continue;
        }

        if in_code_block {
            text.push_str(line);
            text.push('\n');
            let range = line_start..text.len() - 1;
            if !range.is_empty() {
                highlights.push((
                    range,
                    HighlightStyle {
                        background_color: Some(rgb(CODE_BG).into()),
                        ..Default::default()
                    },
                ));
            }
            continue;
        }

        let heading_level = trimmed.chars().take_while(|c| *c == '#').count();
        if (1..=6).contains(&heading_level) && trimmed[heading_level..].starts_with(' ') {
            let content = trimmed[heading_level..].trim_start();
            let content_start = text.len();
            text.push_str(content);
            let style = HighlightStyle {
                font_weight: Some(FontWeight::BOLD),
                color: if heading_level <= 2 {
                    Some(rgb(ACCENT).into())
                } else {
                    Some(rgb(TEXT).into())
                },
                ..Default::default()
            };
            highlights.push((content_start..text.len(), style));
            text.push('\n');
            continue;
        }

        push_line_with_inline(line, &mut text, &mut highlights);
    }

    StyledText::new(text).with_highlights(highlights)
}

/// Push a line, parsing inline `**bold**`, `*italic*`, and `` `code` `` spans.
fn push_line_with_inline(
    line: &str,
    text: &mut String,
    highlights: &mut Vec<(Range<usize>, HighlightStyle)>,
) {
    let mut rest = line;
    while !rest.is_empty() {
        // Find the earliest inline marker; "**" must win over "*" at the same pos.
        let mut earliest: Option<(usize, &'static str)> = None;
        for marker in ["**", "`", "*", "["] {
            if let Some(pos) = rest.find(marker)
                && earliest.is_none_or(|(p, _)| pos < p)
            {
                earliest = Some((pos, marker));
            }
        }
        let Some((pos, marker)) = earliest else {
            text.push_str(rest);
            break;
        };
        if marker == "[" {
            // Try to parse "[text](url)"; emit link text underlined, skip the URL.
            let after = &rest[pos + 1..];
            let Some(close) = after.find("](") else {
                text.push('[');
                rest = after;
                continue;
            };
            let link_text = &after[..close];
            let after_close = &after[close + 2..];
            let Some(url_end) = after_close.find(')') else {
                text.push('[');
                rest = after;
                continue;
            };
            let inner_start = text.len();
            text.push_str(link_text);
            highlights.push((
                inner_start..text.len(),
                HighlightStyle {
                    color: Some(rgb(ACCENT).into()),
                    underline: Some(UnderlineStyle {
                        thickness: px(1.),
                        color: Some(rgb(ACCENT).into()),
                        wavy: false,
                    }),
                    ..Default::default()
                },
            ));
            rest = &after_close[url_end + 1..];
            continue;
        }

        if pos > 0 {
            text.push_str(&rest[..pos]);
        }
        let after_marker = &rest[pos + marker.len()..];
        let Some(close) = after_marker.find(marker) else {
            // Unclosed marker — emit literally and stop.
            text.push_str(marker);
            text.push_str(after_marker);
            break;
        };

        let inner = &after_marker[..close];
        let inner_start = text.len();
        text.push_str(inner);
        let style = match marker {
            "**" => HighlightStyle {
                font_weight: Some(FontWeight::BOLD),
                ..Default::default()
            },
            "`" => HighlightStyle {
                background_color: Some(rgb(CODE_BG).into()),
                color: Some(rgb(MUTED).into()),
                ..Default::default()
            },
            _ => HighlightStyle {
                font_style: Some(FontStyle::Italic),
                ..Default::default()
            },
        };
        highlights.push((inner_start..text.len(), style));
        rest = &after_marker[close + marker.len()..];
    }
    text.push('\n');
}
