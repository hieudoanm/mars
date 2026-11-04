use crate::pdf::markdown::{MdLineType, render_markdown_to_pdf};
use crate::pdf::render::{PdfRenderContext, font_for_line, is_image_line, line_height};
use crate::pdf::writer::{
    BLOCK_GAP, BOX_PAD, CODE_LINE_HEIGHT, LINE_NUM_WIDTH, MARGIN, PARAGRAPH_GAP,
};
use mars_document::Cell;

pub(crate) fn render_markdown_cell(ctx: &mut PdfRenderContext, cell: &Cell) {
    let lines = render_markdown_to_pdf(&cell.source);
    if lines.is_empty() {
        return;
    }

    let mut needed: f64 = BLOCK_GAP;
    let mut prev_normal = false;
    for line in &lines {
        let is_image = is_image_line(&line.text);
        let (_font, _size) = font_for_line(line.kind);
        let wrap_w = ctx.page_width - 2.0 * MARGIN;
        let n_wrapped = wrap_text_len(&line.text, wrap_w) as f64;
        let line_h = line_height(line.kind);
        if is_image {
            needed += 120.0; // image placeholder box height
            prev_normal = false;
        } else {
            if line.kind == MdLineType::Normal && prev_normal {
                needed += PARAGRAPH_GAP;
            }
            needed += n_wrapped * line_h;
            prev_normal = line.kind == MdLineType::Normal;
        }
    }
    if !ctx.content.has_room(needed) {
        ctx.new_page();
    }
    let mut prev_normal = false;
    for line in &lines {
        let is_image = is_image_line(&line.text);
        let (font, size) = font_for_line(line.kind);
        let wrap_w = ctx.page_width - 2.0 * MARGIN;
        let wrapped = wrap_lines(&line.text, wrap_w, font, size);
        let line_h = line_height(line.kind);
        if is_image {
            let img_h = 120.0;
            let img_box_x = MARGIN - 8.0;
            let img_box_w = ctx.page_width - 2.0 * MARGIN + 16.0;
            ctx.content
                .rect_fill(img_box_x, img_box_w, img_h, (0.121, 0.121, 0.133));
            ctx.content
                .rect_stroke(img_box_x, img_box_w, img_h, (0.60, 0.60, 0.65), 0.8);
            ctx.content
                .rect_fill(MARGIN - 8.0, 4.0, img_h, (0.35, 0.75, 1.0));
            ctx.content.move_down(BOX_PAD);
            ctx.content.set_font("/F1", 9.0);
            ctx.content.set_color((0.898, 0.898, 0.898));
            ctx.content
                .text(MARGIN + LINE_NUM_WIDTH + 8.0, ctx.content.y, &line.text);
            ctx.content
                .move_down(img_h - BOX_PAD * 2.0 - CODE_LINE_HEIGHT);
            ctx.content.move_down(BOX_PAD);
            prev_normal = false;
        } else {
            if line.kind == MdLineType::Normal && prev_normal {
                ctx.content.move_down(PARAGRAPH_GAP);
            }
            for wline in &wrapped {
                ctx.content.set_font(font, size);
                ctx.content.text(MARGIN, ctx.content.y, wline);
                ctx.content.move_down(line_h);
            }
            prev_normal = line.kind == MdLineType::Normal;
        }
    }
    ctx.content.move_down(BLOCK_GAP);
}

fn wrap_text_len(text: &str, max_width: f64) -> usize {
    crate::pdf::wrap::wrap_text(text, max_width, "/F1", 11.0).len()
}

fn wrap_lines(text: &str, max_width: f64, font: &'static str, size: f64) -> Vec<String> {
    crate::pdf::wrap::wrap_text(text, max_width, font, size)
}
