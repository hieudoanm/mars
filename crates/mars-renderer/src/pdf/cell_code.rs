use crate::pdf::output::render_output_to_pdf_lines;
use crate::pdf::render::PdfRenderContext;
use crate::pdf::tokens::highlight_rust_tokens;
use crate::pdf::writer::{BLOCK_GAP, BOX_PAD, CODE_LINE_HEIGHT, LINE_NUM_WIDTH, MARGIN};
use mars_document::Cell;

pub(crate) fn render_code_cell(ctx: &mut PdfRenderContext, cell: &Cell) {
    let label = format!(
        "In [{}]",
        cell.execution_count
            .map_or_else(|| " ".to_owned(), |c| c.to_string())
    );
    let code_inner_w = ctx.page_width - 2.0 * MARGIN - LINE_NUM_WIDTH - 32.0;
    let code_lines = crate::pdf::wrap::wrap_text(&cell.source, code_inner_w, "/F2", 9.0);
    let output_lines: Vec<String> = cell
        .outputs
        .iter()
        .flat_map(|output| render_output_to_pdf_lines(output, code_inner_w))
        .collect();

    let output_line_heights: f64 = output_lines
        .iter()
        .map(|l| {
            if l.starts_with("[IMAGE:") {
                100.0 // image placeholder box height
            } else {
                CODE_LINE_HEIGHT
            }
        })
        .sum();

    let output_gap = BLOCK_GAP;

    let code_box_h = (1 + code_lines.len()) as f64 * CODE_LINE_HEIGHT + BOX_PAD;
    let output_box_h = if !output_lines.is_empty() {
        output_line_heights + 2.0 * BOX_PAD
    } else {
        0.0
    };
    let needed = code_box_h
        + if output_lines.is_empty() {
            0.0
        } else {
            output_gap + output_box_h
        }
        + BLOCK_GAP;
    if !ctx.content.has_room(needed) {
        ctx.new_page();
    }

    // ---- Code box (input) ----
    let box_x = MARGIN - 8.0;
    let box_w = ctx.page_width - 2.0 * MARGIN + 16.0;
    ctx.content
        .rect_fill(box_x, box_w, code_box_h, (0.153, 0.153, 0.176)); // #262626 dark
    ctx.content
        .rect_stroke(box_x, box_w, code_box_h, (0.85, 0.85, 0.88), 1.0);
    // Left accent bar (Project Mars orange)
    ctx.content
        .rect_fill(MARGIN - 8.0, 4.0, code_box_h, (1.0, 0.55, 0.15));

    // Line number gutter background
    ctx.content.rect_fill(
        MARGIN - 4.0,
        LINE_NUM_WIDTH,
        code_box_h,
        (0.204, 0.204, 0.235), // #34343a slightly lighter than bg
    );

    ctx.content.move_down(BOX_PAD);

    // Execution label (top-left, outside line numbers)
    ctx.content.set_font("/F3", 8.0);
    ctx.content.set_color((0.898, 0.898, 0.898)); // light text on dark bg
    ctx.content
        .text(MARGIN + LINE_NUM_WIDTH + 8.0, ctx.content.y, &label);
    ctx.content.move_down(CODE_LINE_HEIGHT);

    // Line numbers + code
    ctx.content.set_font("/F2", 9.0);
    let mut code_y = ctx.content.y;
    for (index, line) in code_lines.iter().enumerate() {
        let line_num = format!("{:>3}", index + 1);
        ctx.content.set_color((0.55, 0.55, 0.60));
        ctx.content.text(MARGIN + 2.0, code_y, &line_num);
        let mut x = MARGIN + LINE_NUM_WIDTH + 8.0;
        for (token, color) in highlight_rust_tokens(line) {
            ctx.content.set_color(color);
            ctx.content.text(x, code_y, token);
            x += token.len() as f64 * 9.0 * 0.5;
        }
        code_y -= CODE_LINE_HEIGHT;
    }
    ctx.content.y = code_y;

    // ---- Output box (separate from code) ----
    if !output_lines.is_empty() {
        ctx.content.y -= output_gap;

        // Output box background and border
        let out_box_x = MARGIN - 8.0;
        let out_box_w = ctx.page_width - 2.0 * MARGIN + 16.0;
        ctx.content
            .rect_fill(out_box_x, out_box_w, output_box_h, (0.121, 0.121, 0.133)); // #1f1f1f dark
        ctx.content
            .rect_stroke(out_box_x, out_box_w, output_box_h, (0.65, 0.85, 0.98), 1.0);
        // Left accent bar (blue for output)
        ctx.content
            .rect_fill(MARGIN - 8.0, 4.0, output_box_h, (0.35, 0.75, 1.0));

        ctx.content.move_down(BOX_PAD);

        ctx.content.set_font("/F1", 8.5);
        ctx.content.set_color((0.898, 0.898, 0.898)); // light text on dark output bg
        for line in &output_lines {
            let is_img = line.starts_with("[IMAGE:");
            if is_img {
                let img_h = 100.0;
                let img_box_x = MARGIN + LINE_NUM_WIDTH + 4.0;
                let img_box_w = ctx.page_width - 2.0 * MARGIN - LINE_NUM_WIDTH - 20.0;
                ctx.content
                    .rect_fill(img_box_x, img_box_w, img_h, (0.121, 0.121, 0.133)); // dark
                ctx.content
                    .rect_stroke(img_box_x, img_box_w, img_h, (0.65, 0.85, 0.98), 0.8);
                ctx.content
                    .rect_fill(img_box_x, 4.0, img_h, (0.35, 0.75, 1.0));
                ctx.content.move_down(BOX_PAD);
                ctx.content.set_font("/F1", 8.5);
                ctx.content.text(img_box_x + 8.0, ctx.content.y, line);
                ctx.content
                    .move_down(img_h - BOX_PAD * 2.0 - CODE_LINE_HEIGHT);
                ctx.content.move_down(BOX_PAD);
            } else {
                ctx.content
                    .text(MARGIN + LINE_NUM_WIDTH + 8.0, ctx.content.y, line);
                ctx.content.move_down(CODE_LINE_HEIGHT);
            }
        }
        ctx.content.move_down(BOX_PAD);
    }
    ctx.content.move_down(BLOCK_GAP);
}
