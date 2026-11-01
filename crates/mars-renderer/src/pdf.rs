use crate::render_chart_svg;
use mars_core::{CellType, Output, PaperSize};
use mars_document::Notebook;
use pulldown_cmark::{Options, Parser};
use std::io::Write as IoWrite;

const MARGIN: f64 = 54.0; // 0.75 inch in points
const LINE_HEIGHT: f64 = 12.5;
const CODE_LINE_HEIGHT: f64 = 12.0;
const BLOCK_GAP: f64 = 16.0; // uniform gap between every heading, paragraph, code block, output
const PARAGRAPH_GAP: f64 = 8.0; // gap between consecutive paragraphs and list items
const BOX_PAD: f64 = 14.0;
const LINE_NUM_WIDTH: f64 = 28.0;

fn pdf_escape(text: &str) -> String {
    text.replace('\\', "\\\\").replace('(', "\\(").replace(')', "\\)")
}

/// PDF object writer. Tracks object numbers and writes objects to a buffer.
struct PdfWriter {
    objects: Vec<String>,
    is_stream: Vec<bool>,
    xref_positions: Vec<usize>,
}

impl PdfWriter {
    fn new() -> Self {
        Self {
            objects: Vec::new(),
            is_stream: Vec::new(),
            xref_positions: Vec::new(),
        }
    }

    fn allocate(&mut self) -> usize {
        self.objects.push(String::new());
        self.is_stream.push(false);
        self.objects.len()
    }

    /// Allocates a stream object (written with /Length and stream/endstream).
    fn allocate_stream(&mut self) -> usize {
        self.objects.push(String::new());
        self.is_stream.push(true);
        self.objects.len()
    }

    fn write_object_content(&mut self, obj_num: usize, content: String) {
        let slot = &mut self.objects[obj_num - 1];
        slot.push_str(&content);
    }

    fn finish(mut self) -> Vec<u8> {
        let mut out = Vec::new();

        out.extend_from_slice(b"%PDF-1.4\n");
        out.extend_from_slice(b"%\xE2\xE3\xCF\xD3\n");

        self.xref_positions.clear();
        let mut offset = out.len();
        for (index, object) in self.objects.iter().enumerate() {
            let obj_num = index + 1;
            self.xref_positions.push(offset);
            if self.is_stream[index] {
                writeln!(
                    out,
                    "{obj_num} 0 obj\n<< /Length {} >>\nstream\n{}\nendstream\nendobj\n",
                    object.len(),
                    object
                )
                .expect("write to vec");
            } else {
                writeln!(out, "{obj_num} 0 obj\n{object}\nendobj\n").expect("write to vec");
            }
            offset = out.len();
        }

        let xref_offset = out.len();
        writeln!(out, "xref\n0 {}\n", self.objects.len() + 1).expect("write to vec");
        writeln!(out, "0000000000 65535 f ").expect("write to vec");
        for &pos in &self.xref_positions {
            writeln!(out, "{pos:010} 00000 n ").expect("write to vec");
        }

        let root_obj = self.objects.len();
        writeln!(
            out,
            "trailer\n<< /Size {} /Root {} 0 R >>\nstartxref\n{}\n%%EOF",
            self.objects.len() + 1,
            root_obj,
            xref_offset
        )
        .expect("write to vec");

        out
    }
}

/// Minimal PDF content stream generator for a single page.
///
/// Drawing operations (rects for code boxes, line numbers, page furniture)
/// are emitted outside the `BT ... ET` text block; text operations go inside it.
struct PageContent {
    drawing: String,
    text: String,
    y: f64,
    _page_height: f64,
    bottom_margin: f64,
    font: &'static str,
    font_size: f64,
    color: (f64, f64, f64),
    page_number: usize,
}

impl PageContent {
    fn new(page_height: f64, page_number: usize) -> Self {
        Self {
            drawing: String::new(),
            text: "BT\n0 0 0 rg\n".to_owned(),
            y: page_height - MARGIN,
            _page_height: page_height,
            bottom_margin: MARGIN + 20.0, // reserve room for footer
            font: "/F1",
            font_size: 10.0,
            color: (0.0, 0.0, 0.0),
            page_number,
        }
    }

    fn has_room(&self, needed: f64) -> bool {
        self.y - needed > self.bottom_margin
    }

    fn rect_fill(&mut self, x: f64, w: f64, h: f64, color: (f64, f64, f64)) {
        let (r, g, b) = color;
        // re x y w h, then rg r g b, then f
        self.drawing
            .push_str(&format!("{} {} {} {} re {} {} {} rg f\n", x, self.y - h, w, h, r, g, b));
    }

    fn rect_stroke(&mut self, x: f64, w: f64, h: f64, color: (f64, f64, f64), wdt: f64) {
        let (r, g, b) = color;
        self.drawing.push_str(&format!(
            "{} w {} {} {} {} re {} {} {} RG S\n",
            wdt, x, self.y - h, w, h, r, g, b
        ));
    }

    fn set_font(&mut self, font: &'static str, size: f64) {
        if self.font != font || self.font_size != size {
            self.text
                .push_str(&format!("{} {} Tf\n", font, size));
            self.font = font;
            self.font_size = size;
        }
    }

    fn set_color(&mut self, color: (f64, f64, f64)) {
        if self.color != color {
            let (r, g, b) = color;
            self.text.push_str(&format!("{} {} {} rg\n", r, g, b));
            self.color = color;
        }
    }

    fn text(&mut self, x: f64, y: f64, text: &str) {
        let escaped = pdf_escape(text);
        self.text
            .push_str(&format!("1 0 0 1 {} {} Tm ({}) Tj\n", x, y, escaped));
    }

    fn move_down(&mut self, amount: f64) {
        self.y -= amount;
    }

    /// Draw page footer with page number at the bottom.
    fn draw_footer(&mut self, page_width: f64) {
        let footer_rule_y = MARGIN - 20.0;
        let footer_text_y = MARGIN - 32.0;
        self.drawing.push_str(&format!(
            "0.5 w {} {} m {} {} l S\n",
            MARGIN,
            footer_rule_y,
            page_width - MARGIN,
            footer_rule_y
        ));
        self.text.push_str(&format!(
            "1 0 0 1 {} {} Tm (Page {}) Tj\n",
            MARGIN,
            footer_text_y,
            self.page_number
        ));
    }

    fn into_string(self) -> String {
        let mut commands = self.drawing;
        commands.push_str(&self.text);
        commands.push_str("ET\n");
        commands
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MdLineType {
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
    Normal,
}

struct MdLine {
    text: String,
    kind: MdLineType,
}

fn render_markdown_to_pdf(source: &str) -> Vec<MdLine> {
    let options = Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH;
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, Parser::new_ext(source, options));
    let cleaned = ammonia::clean(&html);

    let mut lines: Vec<MdLine> = Vec::new();
    let mut current = String::new();
    let mut tag_buf = String::new();
    let mut in_tag = false;
    let mut list_depth = 0usize;
    let mut ordered_start: Option<usize> = None;
    let mut heading_level: Option<u32> = None;

    fn flush_current(
        current: &mut String,
        lines: &mut Vec<MdLine>,
        list_depth: usize,
        ordered_start: &mut Option<usize>,
        heading_level: Option<u32>,
    ) {
        let text = current.trim();
        if text.is_empty() {
            return;
        }
        let prefix = if list_depth > 0 {
            if let Some(start) = ordered_start {
                let idx = *start;
                *start += 1;
                format!("{}{}. ", "  ".repeat(list_depth - 1), idx)
            } else {
                format!("{}• ", "  ".repeat(list_depth - 1))
            }
        } else {
            String::new()
        };
        let kind = match heading_level {
            Some(1) => MdLineType::H1,
            Some(2) => MdLineType::H2,
            Some(3) => MdLineType::H3,
            Some(4) => MdLineType::H4,
            Some(5) => MdLineType::H5,
            Some(6) => MdLineType::H6,
            _ => MdLineType::Normal,
        };
        lines.push(MdLine {
            text: format!("{prefix}{text}"),
            kind,
        });
        current.clear();
    }

    for ch in cleaned.chars() {
        match ch {
            '<' => {
                in_tag = true;
                tag_buf.clear();
                flush_current(
                    &mut current,
                    &mut lines,
                    list_depth,
                    &mut ordered_start,
                    heading_level,
                );
            }
            '>' => {
                in_tag = false;
                let tag = tag_buf.trim().to_ascii_lowercase();
                if tag.starts_with('h') && tag.len() == 2 {
                    if let Some(lvl) = tag.chars().nth(1).and_then(|c| c.to_digit(10)) {
                        if (1..=6).contains(&lvl) {
                            heading_level = Some(lvl);
                        }
                    }
                } else if tag == "li" {
                    list_depth += 1;
                    if ordered_start.is_none() {
                        ordered_start = Some(1);
                    }
                } else if tag == "/li" {
                    list_depth = list_depth.saturating_sub(1);
                } else if tag == "/ol" || tag == "/ul" {
                    ordered_start = None;
                } else if tag.starts_with("img") {
                    if let Some(alt_start) = tag.find("alt=") {
                        let alt_rest = &tag[alt_start + 4..];
                        let alt = alt_rest
                            .chars()
                            .skip_while(|c| *c == '"' || *c == '\'')
                            .take_while(|c| *c != '"' && *c != '\'')
                            .collect::<String>();
                        if !alt.is_empty() {
                            lines.push(MdLine {
                                text: format!("[Image: {alt}]"),
                                kind: MdLineType::Normal,
                            });
                        } else {
                            lines.push(MdLine {
                                text: "[Image]".to_owned(),
                                kind: MdLineType::Normal,
                            });
                        }
                    } else {
                        lines.push(MdLine {
                            text: "[Image]".to_owned(),
                            kind: MdLineType::Normal,
                        });
                    }
                } else if let Some(close_tag) = tag.strip_prefix('/') {
                    if close_tag.starts_with('h')
                        && close_tag.len() == 2
                        && close_tag
                            .chars()
                            .nth(1)
                            .and_then(|c| c.to_digit(10))
                            == heading_level
                    {
                        heading_level = None;
                    }
                }
                tag_buf.clear();
            }
            '\n' if !in_tag => {
                flush_current(
                    &mut current,
                    &mut lines,
                    list_depth,
                    &mut ordered_start,
                    heading_level,
                );
            }
            _ if !in_tag => current.push(ch),
            _ if in_tag => tag_buf.push(ch),
            _ => {}
        }
    }
    flush_current(
        &mut current,
        &mut lines,
        list_depth,
        &mut ordered_start,
        heading_level,
    );
    lines
}

fn wrap_text(text: &str, max_width: f64, _font: &'static str, font_size: f64) -> Vec<String> {
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

pub fn render_pdf_notebook(notebook: &Notebook, page_size: PaperSize) -> Vec<u8> {
    let (page_width, page_height) = page_size.dimensions();

    let mut writer = PdfWriter::new();

    let font_helvetica = writer.allocate();
    writer.write_object_content(
        font_helvetica,
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_string(),
    );
    let font_courier = writer.allocate();
    writer.write_object_content(
        font_courier,
        "<< /Type /Font /Subtype /Type1 /BaseFont /Courier >>".to_string(),
    );
    let font_bold = writer.allocate();
    writer.write_object_content(
        font_bold,
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold >>".to_string(),
    );

    let resources = writer.allocate();
    writer.write_object_content(
        resources,
        format!(
            "<< /ProcSet [/PDF /Text] /Font << /F1 {} 0 R /F2 {} 0 R /F3 {} 0 R >> >>",
            font_helvetica, font_courier, font_bold
        ),
    );

    let pages_obj = writer.allocate();

    let mut content = PageContent::new(page_height, 1);

    let mut content_nums: Vec<usize> = Vec::new();
    let mut page_nums: Vec<usize> = Vec::new();
    let mut page_number = 1usize;

    for cell in &notebook.cells {
        match cell.cell_type {
            CellType::Markdown => {
                let lines = render_markdown_to_pdf(&cell.source);
                if !lines.is_empty() {
                    let mut needed: f64 = BLOCK_GAP;
                    let mut prev_normal = false;
                    for line in &lines {
                        let is_image = line.text.starts_with("[Image") && line.text.ends_with("]");
                        let (font, size) = match line.kind {
                            MdLineType::H1 => ("/F3", 18.0),
                            MdLineType::H2 => ("/F3", 15.0),
                            MdLineType::H3 => ("/F3", 13.0),
                            MdLineType::H4 => ("/F3", 12.0),
                            MdLineType::H5 => ("/F1", 11.0),
                            MdLineType::H6 => ("/F1", 10.0),
                            MdLineType::Normal => ("/F1", 11.0),
                        };
                        let wrap_w = page_width - 2.0 * MARGIN;
                        let n_wrapped = wrap_text(&line.text, wrap_w, font, size).len() as f64;
                        let line_h = match line.kind {
                            MdLineType::H1 => LINE_HEIGHT * 1.6,
                            MdLineType::H2 => LINE_HEIGHT * 1.4,
                            MdLineType::H3 => LINE_HEIGHT * 1.25,
                            _ => LINE_HEIGHT,
                        };
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
                    if !content.has_room(needed) {
                        content.draw_footer(page_width);
                        let cn = writer.allocate_stream();
                        writer.write_object_content(cn, content.into_string());
                        content_nums.push(cn);
                        let page = writer.allocate();
                        writer.write_object_content(
                            page,
                            format!(
                                "<< /Type /Page /Parent {} 0 R /Resources {} 0 R /Contents {} 0 R /MediaBox [0 0 {} {}] >>",
                                pages_obj, resources, content_nums[content_nums.len() - 1], page_width, page_height
                            ),
                        );
                        page_nums.push(page);
                        page_number += 1;
                        content = PageContent::new(page_height, page_number);
                    }
                    let mut prev_normal = false;
                    for line in &lines {
                        let is_image = line.text.starts_with("[Image") && line.text.ends_with("]");
                        let (font, size) = match line.kind {
                            MdLineType::H1 => ("/F3", 18.0),
                            MdLineType::H2 => ("/F3", 15.0),
                            MdLineType::H3 => ("/F3", 13.0),
                            MdLineType::H4 => ("/F3", 12.0),
                            MdLineType::H5 => ("/F1", 11.0),
                            MdLineType::H6 => ("/F1", 10.0),
                            MdLineType::Normal => ("/F1", 11.0),
                        };
                        let wrap_w = page_width - 2.0 * MARGIN;
                        let wrapped = wrap_text(&line.text, wrap_w, font, size);
                        let line_h = match line.kind {
                            MdLineType::H1 => LINE_HEIGHT * 1.6,
                            MdLineType::H2 => LINE_HEIGHT * 1.4,
                            MdLineType::H3 => LINE_HEIGHT * 1.25,
                            _ => LINE_HEIGHT,
                        };
                        if is_image {
                            // Draw image placeholder box
                            let img_h = 120.0;
                            let img_box_x = MARGIN - 8.0;
                            let img_box_w = page_width - 2.0 * MARGIN + 16.0;
                            content.rect_fill(img_box_x, img_box_w, img_h, (0.121, 0.121, 0.133)); // dark
                            content.rect_stroke(img_box_x, img_box_w, img_h, (0.60, 0.60, 0.65), 0.8);
                            content.rect_fill(MARGIN - 8.0, 4.0, img_h, (0.35, 0.75, 1.0));
                            content.move_down(BOX_PAD);
                            content.set_font("/F1", 9.0);
                            content.set_color((0.898, 0.898, 0.898)); // light text on dark bg
                            content.text(MARGIN + LINE_NUM_WIDTH + 8.0, content.y, &line.text);
                            content.move_down(img_h - BOX_PAD * 2.0 - CODE_LINE_HEIGHT);
                            content.move_down(BOX_PAD);
                            prev_normal = false;
                        } else {
                            if line.kind == MdLineType::Normal && prev_normal {
                                content.move_down(PARAGRAPH_GAP);
                            }
                            for wline in &wrapped {
                                content.set_font(font, size);
                                content.text(MARGIN, content.y, wline);
                                content.move_down(line_h);
                            }
                            prev_normal = line.kind == MdLineType::Normal;
                        }
                    }
                    content.move_down(BLOCK_GAP);
                }
            }
            CellType::Code => {
                let label = format!(
                    "In [{}]",
                    cell.execution_count
                        .map_or_else(|| " ".to_owned(), |c| c.to_string())
                );
                let code_inner_w = page_width - 2.0 * MARGIN - LINE_NUM_WIDTH - 32.0;
                let code_lines = wrap_text(&cell.source, code_inner_w, "/F2", 9.0);
                let output_lines: Vec<String> = cell
                    .outputs
                    .iter()
                    .flat_map(|output| render_output_to_pdf_lines(output, code_inner_w))
                    .collect();

                let output_line_heights: f64 = output_lines.iter().map(|l| {
                    if l.starts_with("[IMAGE:") {
                        100.0 // image placeholder box height
                    } else {
                        CODE_LINE_HEIGHT
                    }
                }).sum();

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
                if !content.has_room(needed) {
                    content.draw_footer(page_width);
                    let cn = writer.allocate_stream();
                    writer.write_object_content(cn, content.into_string());
                    content_nums.push(cn);
                    let page = writer.allocate();
                    writer.write_object_content(
                        page,
                        format!(
                            "<< /Type /Page /Parent {} 0 R /Resources {} 0 R /Contents {} 0 R /MediaBox [0 0 {} {}] >>",
                            pages_obj, resources, content_nums[content_nums.len() - 1], page_width, page_height
                        ),
                    );
                    page_nums.push(page);
                    page_number += 1;
                    content = PageContent::new(page_height, page_number);
                }

                // ---- Code box (input) ----
                let box_x = MARGIN - 8.0;
                let box_w = page_width - 2.0 * MARGIN + 16.0;
                content.rect_fill(box_x, box_w, code_box_h, (0.153, 0.153, 0.176)); // #262626 dark
                content.rect_stroke(box_x, box_w, code_box_h, (0.85, 0.85, 0.88), 1.0);
                // Left accent bar (Mars orange)
                content.rect_fill(MARGIN - 8.0, 4.0, code_box_h, (1.0, 0.55, 0.15));

                // Line number gutter background
                content.rect_fill(
                    MARGIN - 4.0,
                    LINE_NUM_WIDTH,
                    code_box_h,
                    (0.204, 0.204, 0.235), // #34343a slightly lighter than bg
                );

                content.move_down(BOX_PAD);

                // Execution label (top-left, outside line numbers)
                content.set_font("/F3", 8.0);
                content.set_color((0.898, 0.898, 0.898)); // light text on dark bg
                content.text(MARGIN + LINE_NUM_WIDTH + 8.0, content.y, &label);
                content.move_down(CODE_LINE_HEIGHT);

                // Line numbers + code
                content.set_font("/F2", 9.0);
                let mut code_y = content.y;
                for (index, line) in code_lines.iter().enumerate() {
                    let line_num = format!("{:>3}", index + 1);
                    content.set_color((0.55, 0.55, 0.60));
                    content.text(MARGIN + 2.0, code_y, &line_num);
                    let mut x = MARGIN + LINE_NUM_WIDTH + 8.0;
                    for (token, color) in highlight_rust_tokens(line) {
                        content.set_color(color);
                        content.text(x, code_y, token);
                        x += token.len() as f64 * 9.0 * 0.5;
                    }
                    code_y -= CODE_LINE_HEIGHT;
                }
                content.y = code_y;

                // ---- Output box (separate from code) ----
                if !output_lines.is_empty() {
                    content.y -= output_gap;

                    // Output box background and border
                    let out_box_x = MARGIN - 8.0;
                    let out_box_w = page_width - 2.0 * MARGIN + 16.0;
                    content.rect_fill(out_box_x, out_box_w, output_box_h, (0.121, 0.121, 0.133)); // #1f1f1f dark
                    content.rect_stroke(out_box_x, out_box_w, output_box_h, (0.65, 0.85, 0.98), 1.0);
                    // Left accent bar (blue for output)
                    content.rect_fill(MARGIN - 8.0, 4.0, output_box_h, (0.35, 0.75, 1.0));

                    content.move_down(BOX_PAD);

                    content.set_font("/F1", 8.5);
                    content.set_color((0.898, 0.898, 0.898)); // light text on dark output bg
                    for line in &output_lines {
                        let is_img = line.starts_with("[IMAGE:");
                        if is_img {
                            let img_h = 100.0;
                            let img_box_x = MARGIN + LINE_NUM_WIDTH + 4.0;
                            let img_box_w = page_width - 2.0 * MARGIN - LINE_NUM_WIDTH - 20.0;
                            content.rect_fill(img_box_x, img_box_w, img_h, (0.121, 0.121, 0.133)); // dark
                            content.rect_stroke(img_box_x, img_box_w, img_h, (0.65, 0.85, 0.98), 0.8);
                            content.rect_fill(img_box_x, 4.0, img_h, (0.35, 0.75, 1.0));
                            content.move_down(BOX_PAD);
                            content.set_font("/F1", 8.5);
                            content.text(img_box_x + 8.0, content.y, line);
                            content.move_down(img_h - BOX_PAD * 2.0 - CODE_LINE_HEIGHT);
                            content.move_down(BOX_PAD);
                        } else {
                            content.text(MARGIN + LINE_NUM_WIDTH + 8.0, content.y, line);
                            content.move_down(CODE_LINE_HEIGHT);
                        }
                    }
                    content.move_down(BOX_PAD);
                }
                content.move_down(BLOCK_GAP);
            }
            CellType::Raw => {
                let raw_lines =
                    wrap_text(&cell.source, page_width - 2.0 * MARGIN - 24.0, "/F2", 9.0);
                let box_h = (raw_lines.len() as f64) * CODE_LINE_HEIGHT + 2.0 * BOX_PAD;
                let needed = box_h + BLOCK_GAP;
                if !content.has_room(needed) {
                    content.draw_footer(page_width);
                    let cn = writer.allocate_stream();
                    writer.write_object_content(cn, content.into_string());
                    content_nums.push(cn);
                    let page = writer.allocate();
                    writer.write_object_content(
                        page,
                        format!(
                            "<< /Type /Page /Parent {} 0 R /Resources {} 0 R /Contents {} 0 R /MediaBox [0 0 {} {}] >>",
                            pages_obj, resources, content_nums[content_nums.len() - 1], page_width, page_height
                        ),
                    );
                    page_nums.push(page);
                    page_number += 1;
                    content = PageContent::new(page_height, page_number);
                }

                let box_x = MARGIN - 8.0;
                let box_w = page_width - 2.0 * MARGIN + 16.0;
                content.rect_fill(box_x, box_w, box_h, (0.94, 0.94, 0.96));
                content.rect_stroke(box_x, box_w, box_h, (0.82, 0.82, 0.86), 0.8);

                content.move_down(BOX_PAD);

                content.set_font("/F2", 9.0);
                for line in &raw_lines {
                    content.text(MARGIN + 4.0, content.y, line);
                    content.move_down(CODE_LINE_HEIGHT);
                }
                content.move_down(BOX_PAD);
                content.move_down(BLOCK_GAP - CODE_LINE_HEIGHT);
            }
        }
    }

    // Save final page content
    content.draw_footer(page_width);
    let cn = writer.allocate_stream();
    writer.write_object_content(cn, content.into_string());
    content_nums.push(cn);
    let page = writer.allocate();
    writer.write_object_content(
        page,
        format!(
            "<< /Type /Page /Parent {} 0 R /Resources {} 0 R /Contents {} 0 R /MediaBox [0 0 {} {}] >>",
            pages_obj, resources, content_nums[content_nums.len() - 1], page_width, page_height
        ),
    );
    page_nums.push(page);

    // Fix pages object with actual kids
    let kids = page_nums
        .iter()
        .map(|n| format!("{} 0 R", n))
        .collect::<Vec<_>>()
        .join(" ");
    writer.write_object_content(
        pages_obj,
        format!(
            "<< /Type /Pages /Kids [{kids}] /Count {} >>",
            page_nums.len()
        ),
    );

    // Catalog object (must be last)
    let catalog_obj = writer.allocate();
    writer.write_object_content(catalog_obj, format!("<< /Type /Catalog /Pages {} 0 R >>", pages_obj));

    writer.finish()
}

fn render_output_to_pdf_lines(output: &Output, max_width: f64) -> Vec<String> {
    match output {
        Output::Text { text } => {
            let trimmed = text.trim_end_matches('\n');
            if trimmed.is_empty() {
                return Vec::new();
            }
            wrap_text(trimmed, max_width, "/F1", 8.5)
        }
        Output::Error {
            name,
            message,
            traceback,
        } => {
            let mut lines = vec![format!("Error: {name}")];
            lines.extend(wrap_text(message, max_width, "/F1", 8.5));
            for trace in traceback {
                lines.extend(wrap_text(trace, max_width, "/F1", 8.5));
            }
            lines
        }
        Output::Html { html } => {
            let mut text = String::new();
            let mut in_tag = false;
            for ch in html.chars() {
                match ch {
                    '<' => in_tag = true,
                    '>' => in_tag = false,
                    _ if !in_tag => text.push(ch),
                    _ => {}
                }
            }
            let text = text.trim();
            if text.is_empty() {
                Vec::new()
            } else {
                wrap_text(text, max_width, "/F1", 8.5)
            }
        }
        Output::Image { mime_type, data } => {
            vec![format!("[IMAGE: {mime_type}, {} bytes]", data.len())]
        }
        Output::Table { columns, rows } => {
            let mut lines = Vec::new();
            let header = columns.join(" | ");
            lines.push(header);
            lines.push(columns.iter().map(|_| "---").collect::<Vec<_>>().join(" | "));
            for row in rows {
                lines.push(row.join(" | "));
            }
            lines
        }
        Output::Latex { latex } => {
            vec![format!("[LaTeX: {latex}]")]
        }
        Output::Chart { spec } => {
            let svg = render_chart_svg(spec);
            vec![format!("[Chart SVG: {} bytes]", svg.len())]
        }
        Output::MimeBundle { data, .. } => {
            for mime_type in [
                "image/svg+xml",
                "image/png",
                "image/jpeg",
                "image/gif",
                "image/webp",
            ] {
                if let Some(value) = data.get(mime_type).and_then(|v| v.as_str()) {
                    return vec![format!("[IMAGE: {mime_type}, {} bytes]", value.len())];
                }
            }
            if let Some(spec) = data
                .get("application/vnd.vega.v5+json")
                .or_else(|| data.get("application/vnd.vega-lite.v5+json"))
                .or_else(|| data.get("application/vnd.vega-lite+json"))
            {
                let svg = render_chart_svg(spec);
                return vec![format!("[Chart: {} bytes]", svg.len())];
            }
            if let Some(text) = data.get("text/plain").and_then(|v| v.as_str()) {
                return wrap_text(text, max_width, "/F1", 8.5);
            }
            vec!["[MIME bundle output]".to_owned()]
        }
    }
}

fn rust_keywords() -> &'static [&'static str] {
    &[
        "fn", "let", "mut", "const", "static", "struct", "enum", "trait", "impl", "pub",
        "use", "mod", "crate", "self", "super", "Self", "if", "else", "match", "loop",
        "while", "for", "in", "break", "continue", "return", "async", "await", "unsafe",
        "extern", "macro", "macro_rules", "where", "dyn", "box", "move", "ref", "type",
        "as", "true", "false", "None", "Some", "Ok", "Err",
    ]
}

fn rust_types() -> &'static [&'static str] {
    &[
        "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize",
        "f32", "f64", "bool", "char", "str", "String", "Vec", "Option", "Result", "Box",
        "Rc", "Arc", "Cell", "RefCell", "Mutex", "RwLock",
    ]
}

/// Tokenize a single line of Rust source into (text, color) pairs for PDF rendering.
fn highlight_rust_tokens(source: &str) -> Vec<(&str, (f64, f64, f64))> {
    let keyword = (0.776, 0.490, 0.870);   // #c678dd
    let r#type = (0.878, 0.423, 0.458);    // #e06c75
    let comment = (0.360, 0.388, 0.439);   // #5c6370
    let string = (0.765, 0.910, 0.553);    // #c3e88d bright green
    let number = (0.819, 0.603, 0.400);    // #d19a66
    let macro_ = (0.886, 0.752, 0.553);    // #e2c08d
    let default = (0.898, 0.898, 0.898);   // #e5e5e5

    let mut tokens: Vec<(&str, (f64, f64, f64))> = Vec::new();
    let chars: Vec<char> = source.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];

        if ch == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
            let end = source[i..].find('\n').map(|n| i + n).unwrap_or(chars.len());
            tokens.push((&source[i..end], comment));
            i = end;
            continue;
        }

        if ch == '"' {
            let mut end = i + 1;
            while end < chars.len() {
                if chars[end] == '\\' { end += 2; continue; }
                if chars[end] == '"' { break; }
                end += 1;
            }
            end = end.min(chars.len() - 1);
            if chars[end] == '"' { end += 1; }
            tokens.push((&source[i..end], string));
            i = end;
            continue;
        }

        if ch == '\'' {
            let mut end = i + 1;
            while end < chars.len() && chars[end] != '\'' { end += 1; }
            end = end.min(chars.len() - 1);
            if chars[end] == '\'' { end += 1; }
            tokens.push((&source[i..end], string));
            i = end;
            continue;
        }

        if ch.is_alphabetic() || ch == '_' {
            let mut end = i;
            while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') { end += 1; }
            let word = &source[i..end];
            let color = if rust_keywords().contains(&word) {
                keyword
            } else if rust_types().contains(&word) {
                r#type
            } else if end < chars.len() && chars[end] == '!' {
                macro_
            } else {
                default
            };
            tokens.push((word, color));
            i = end;
            continue;
        }

        if ch.is_ascii_digit() {
            let mut end = i;
            while end < chars.len() && (chars[end].is_ascii_digit() || chars[end] == '.' || chars[end] == '_') { end += 1; }
            tokens.push((&source[i..end], number));
            i = end;
            continue;
        }

        tokens.push((&source[i..i + 1], default));
        i += 1;
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;
    use mars_document::{Cell, Notebook};

    #[test]
    fn renders_minimal_pdf() {
        let mut notebook = Notebook::new();
        notebook
            .add_cell(Cell::new(CellType::Markdown, "# Hello"))
            .expect("unique ID");

        let pdf = render_pdf_notebook(&notebook, PaperSize::A4);
        let content = String::from_utf8_lossy(&pdf);
        assert!(content.starts_with("%PDF-1.4"));
        assert!(content.contains("%%EOF"));
        assert!(content.contains("Hello"));
    }

    #[test]
    fn renders_code_cell_pdf() {
        let mut notebook = Notebook::new();
        let mut cell = Cell::new(CellType::Code, "1 + 1");
        cell.outputs.push(Output::Text {
            text: "2\n".to_owned(),
        });
        notebook.add_cell(cell).expect("unique ID");

        let pdf = render_pdf_notebook(&notebook, PaperSize::Letter);
        let content = String::from_utf8_lossy(&pdf);
        assert!(content.starts_with("%PDF-1.4"));
        assert!(content.contains("(1)"));
        assert!(content.contains("(+)"));
        assert!(content.contains("(2)"));
    }

    #[test]
    fn a4_dimensions() {
        let (w, h) = PaperSize::A4.dimensions();
        assert!((w - 595.28).abs() < 0.01);
        assert!((h - 841.89).abs() < 0.01);
        assert!((w / h - (1.0 / 2.0_f64.sqrt())).abs() < 0.01);
    }

    #[test]
    fn letter_dimensions() {
        let (w, h) = PaperSize::Letter.dimensions();
        assert!((w - 612.0).abs() < 0.01);
        assert!((h - 792.0).abs() < 0.01);
    }

    #[test]
    fn parse_paper_size() {
        assert_eq!(PaperSize::parse("a4").unwrap(), PaperSize::A4);
        assert_eq!(PaperSize::parse("letter").unwrap(), PaperSize::Letter);
        assert_eq!(PaperSize::parse("a3").unwrap(), PaperSize::A3);
        assert!(PaperSize::parse("invalid").is_err());
    }

    #[test]
    fn iso216_check() {
        assert!(PaperSize::A4.is_iso216());
        assert!(PaperSize::A3.is_iso216());
        assert!(!PaperSize::Letter.is_iso216());
    }

    #[test]
    fn default_is_a4() {
        assert_eq!(PaperSize::default(), PaperSize::A4);
    }

    #[test]
    fn pdf_contains_title() {
        let mut notebook = Notebook::new();
        notebook.metadata.title = Some("My Report".to_owned());
        notebook
            .add_cell(Cell::new(CellType::Markdown, "# My Report"))
            .expect("unique ID");

        let pdf = render_pdf_notebook(&notebook, PaperSize::A4);
        let content = String::from_utf8_lossy(&pdf);
        assert!(content.contains("My Report"));
    }
}
