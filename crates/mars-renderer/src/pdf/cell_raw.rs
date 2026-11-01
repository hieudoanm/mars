use crate::pdf::render::PdfRenderContext;
use crate::pdf::writer::{BLOCK_GAP, BOX_PAD, CODE_LINE_HEIGHT, MARGIN};
use mars_document::Cell;

pub(crate) fn render_raw_cell(ctx: &mut PdfRenderContext, cell: &Cell) {
    let raw_lines = crate::pdf::wrap::wrap_text(
        &cell.source,
        ctx.page_width - 2.0 * MARGIN - 24.0,
        "/F2",
        9.0,
    );
    let box_h = (raw_lines.len() as f64) * CODE_LINE_HEIGHT + 2.0 * BOX_PAD;
    let needed = box_h + BLOCK_GAP;
    if !ctx.content.has_room(needed) {
        ctx.new_page();
    }

    let box_x = MARGIN - 8.0;
    let box_w = ctx.page_width - 2.0 * MARGIN + 16.0;
    ctx.content
        .rect_fill(box_x, box_w, box_h, (0.94, 0.94, 0.96));
    ctx.content
        .rect_stroke(box_x, box_w, box_h, (0.82, 0.82, 0.86), 0.8);

    ctx.content.move_down(BOX_PAD);

    ctx.content.set_font("/F2", 9.0);
    for line in &raw_lines {
        ctx.content.text(MARGIN + 4.0, ctx.content.y, line);
        ctx.content.move_down(CODE_LINE_HEIGHT);
    }
    ctx.content.move_down(BOX_PAD);
    ctx.content.move_down(BLOCK_GAP - CODE_LINE_HEIGHT);
}
