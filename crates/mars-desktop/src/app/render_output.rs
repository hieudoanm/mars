use super::{BORDER, CODE_BG, ERROR_BG, ERROR_BORDER, FAINT, HEADER, MUTED, TEXT};
use crate::output_text::{html_text, output_lines};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use gpui::*;
use mars_core::Output;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

pub(super) fn render_output(output: &Output) -> AnyElement {
    match output {
        Output::Image { mime_type, data } => render_image_output(mime_type, data),
        Output::Chart { spec } => {
            let chart_svg = mars_renderer::chart::render_chart_svg(spec);
            if chart_svg.trim_start().starts_with("<svg") {
                div()
                    .mx_3()
                    .mb_2()
                    .px_3()
                    .py_2()
                    .child(svg().data(chart_svg.as_bytes()).w_full().max_h(px(320.)))
                    .into_any_element()
            } else {
                output_text_block(&output_lines(output), output)
            }
        }
        Output::Html { html } => {
            let text = html_text(html);
            if text.is_empty() {
                placeholder_block("[HTML output]".to_owned())
            } else {
                output_text_block(&[text], output)
            }
        }
        Output::Latex { latex } => div()
            .mx_3()
            .mb_2()
            .px_3()
            .py_2()
            .rounded_md()
            .bg(rgb(CODE_BG))
            .text_sm()
            .font_family("New York")
            .text_color(rgb(TEXT))
            .child(latex.clone())
            .into_any_element(),
        Output::Table { columns, rows } => render_table_output(columns, rows),
        Output::MimeBundle { data, .. } => {
            for (mime, value) in data {
                if mime.starts_with("image/")
                    && let Some(encoded) = value.as_str()
                {
                    return render_image_output(mime, encoded);
                }
            }
            output_text_block(&output_lines(output), output)
        }
        _ => output_text_block(&output_lines(output), output),
    }
}

fn render_table_output(columns: &[String], rows: &[Vec<String>]) -> AnyElement {
    const MAX_ROWS: usize = 200;
    let mut table = div()
        .mx_3()
        .mb_2()
        .flex()
        .flex_col()
        .rounded_md()
        .border_1()
        .border_color(rgb(BORDER))
        .overflow_hidden();
    let mut header = div().flex().bg(rgb(HEADER)).px_2().py_1().gap_2();
    for column in columns {
        header = header.child(
            div()
                .flex_1()
                .min_w(px(80.))
                .text_xs()
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(TEXT))
                .child(column.clone()),
        );
    }
    table = table.child(header);
    for row in rows.iter().take(MAX_ROWS) {
        let mut line = div()
            .flex()
            .px_2()
            .py_1()
            .gap_2()
            .border_t_1()
            .border_color(rgb(BORDER));
        for cell in row {
            line = line.child(
                div()
                    .flex_1()
                    .min_w(px(80.))
                    .text_xs()
                    .font_family("Menlo")
                    .text_color(rgb(MUTED))
                    .child(cell.clone()),
            );
        }
        table = table.child(line);
    }
    if rows.len() > MAX_ROWS {
        table = table.child(
            div()
                .px_2()
                .py_1()
                .text_xs()
                .text_color(rgb(FAINT))
                .child(format!("… {} more rows", rows.len() - MAX_ROWS)),
        );
    }
    table.into_any_element()
}

fn output_text_block(lines: &[String], output: &Output) -> AnyElement {
    let (bg, border, color) = match output {
        Output::Error { .. } => (ERROR_BG, Some(ERROR_BORDER), 0xfecaca),
        Output::Text { .. } => (CODE_BG, None, 0xd1d5db),
        _ => (CODE_BG, None, MUTED),
    };
    let mut block = div()
        .mx_3()
        .mb_2()
        .px_3()
        .py_2()
        .rounded_md()
        .bg(rgb(bg))
        .text_xs()
        .font_family("Menlo")
        .text_color(rgb(color))
        .child(lines.join("\n"));
    if let Some(border) = border {
        block = block.border_1().border_color(rgb(border));
    }
    block.into_any_element()
}

fn placeholder_block(text: String) -> AnyElement {
    div()
        .mx_3()
        .mb_2()
        .px_3()
        .py_2()
        .rounded_md()
        .bg(rgb(CODE_BG))
        .text_xs()
        .font_family("Menlo")
        .text_color(rgb(MUTED))
        .child(text)
        .into_any_element()
}

fn render_image_output(mime_type: &str, data: &str) -> AnyElement {
    let Ok(bytes) = BASE64.decode(data) else {
        return placeholder_block(format!("[image: {mime_type}]"));
    };
    if mime_type == "image/svg+xml" {
        return div()
            .mx_3()
            .mb_2()
            .px_3()
            .py_2()
            .child(svg().data(&bytes).w_full().max_h(px(360.)))
            .into_any_element();
    }
    let Some(format) = image_format(mime_type) else {
        return placeholder_block(format!("[image: {mime_type}]"));
    };
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    mime_type.hash(&mut hasher);
    data.hash(&mut hasher);
    let image = Image {
        format,
        bytes,
        id: hasher.finish(),
    };
    div()
        .mx_3()
        .mb_2()
        .child(
            img(ImageSource::Image(Arc::new(image)))
                .max_w_full()
                .max_h(px(360.))
                .object_fit(ObjectFit::Contain),
        )
        .into_any_element()
}

fn image_format(mime_type: &str) -> Option<ImageFormat> {
    match mime_type {
        "image/png" => Some(ImageFormat::Png),
        "image/jpeg" => Some(ImageFormat::Jpeg),
        "image/gif" => Some(ImageFormat::Gif),
        "image/webp" => Some(ImageFormat::Webp),
        "image/bmp" => Some(ImageFormat::Bmp),
        "image/tiff" => Some(ImageFormat::Tiff),
        "image/x-icon" | "image/vnd.microsoft.icon" => Some(ImageFormat::Ico),
        _ => None,
    }
}
