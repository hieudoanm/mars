use crate::render_chart_svg;
use mars_core::{CellType, Output, PaperSize};
use mars_document::Notebook;
use pulldown_cmark::{Options, Parser};
use std::io::Write as IoWrite;

const MARGIN: f64 = 54.0; // 0.75 inch in points
const LINE_HEIGHT: f64 = 12.0;
const CODE_LINE_HEIGHT: f64 = 11.0;
const HEADING_GAP: f64 = 16.0;
const SECTION_GAP: f64 = 24.0;

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
/// All text operations are grouped inside a single `BT ... ET` text object.
/// Font changes use `Tf` within the block; positioning/showing uses `Td`/`Tj`.
struct PageContent {
    commands: String,
    y: f64,
    _page_height: f64,
    bottom_margin: f64,
    font: &'static str,
    font_size: f64,
}

impl PageContent {
    fn new(page_height: f64) -> Self {
        let mut commands = String::new();
        commands.push_str("BT\n");
        Self {
            commands,
            y: page_height - MARGIN,
            _page_height: page_height,
            bottom_margin: MARGIN,
            font: "/F1",
            font_size: 10.0,
        }
    }

    fn has_room(&self, needed: f64) -> bool {
        self.y - needed > self.bottom_margin
    }

    fn set_font(&mut self, font: &'static str, size: f64) {
        if self.font != font || self.font_size != size {
            self.commands
                .push_str(&format!("{} {} Tf\n", font, size));
            self.font = font;
            self.font_size = size;
        }
    }

    fn text(&mut self, x: f64, y: f64, text: &str) {
        let escaped = pdf_escape(text);
        self.commands
            .push_str(&format!("1 0 0 1 {} {} Tm ({}) Tj\n", x, y, escaped));
    }

    fn move_down(&mut self, amount: f64) {
        self.y -= amount;
    }

    fn into_string(self) -> String {
        let mut commands = self.commands;
        if commands.ends_with("ET\n") {
            commands
        } else {
            commands.push_str("ET\n");
            commands
        }
    }
}

fn render_markdown_to_pdf_lines(source: &str) -> Vec<String> {
    let options = Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH;
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, Parser::new_ext(source, options));
    let cleaned = ammonia::clean(&html);

    let mut lines = Vec::new();
    let mut current = String::new();
    let mut in_tag = false;
    for ch in cleaned.chars() {
        match ch {
            '<' => {
                in_tag = true;
                if !current.trim().is_empty() {
                    lines.push(current.trim().to_owned());
                    current.clear();
                }
            }
            '>' => in_tag = false,
            '\n' if !in_tag => {
                if !current.trim().is_empty() {
                    lines.push(current.trim().to_owned());
                    current.clear();
                }
            }
            _ if !in_tag => current.push(ch),
            _ => {}
        }
    }
    if !current.trim().is_empty() {
        lines.push(current.trim().to_owned());
    }
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
    let title = notebook
        .metadata
        .title
        .as_deref()
        .unwrap_or("Mars notebook");

    let mut writer = PdfWriter::new();

    // Font objects (built-in Type1 fonts, no embedding needed)
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

    // Resources object
    let resources = writer.allocate();
    writer.write_object_content(
        resources,
        format!(
            "<< /ProcSet [/PDF /Text] /Font << /F1 {} 0 R /F2 {} 0 R /F3 {} 0 R >> >>",
            font_helvetica, font_courier, font_bold
        ),
    );

    // Pages object (allocated now so page objects can reference it)
    let pages_obj = writer.allocate();

    // First pass: build content streams and collect page breaks
    let mut content = PageContent::new(page_height);
    content.set_font("/F3", 18.0);
    content.text(MARGIN, content.y, title);
    content.move_down(HEADING_GAP);

    let mut content_nums: Vec<usize> = Vec::new();
    let mut page_nums: Vec<usize> = Vec::new();

    for cell in &notebook.cells {
        match cell.cell_type {
            CellType::Markdown => {
                let lines = render_markdown_to_pdf_lines(&cell.source);
                if !lines.is_empty() {
                    let needed = (lines.len() as f64) * LINE_HEIGHT + SECTION_GAP;
                    if !content.has_room(needed) {
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
                        content = PageContent::new(page_height);
                    }
                    content.set_font("/F1", 12.0);
                    for line in &lines {
                        content.text(MARGIN, content.y, line);
                        content.move_down(LINE_HEIGHT);
                    }
                    content.move_down(SECTION_GAP - LINE_HEIGHT);
                }
            }
            CellType::Code => {
                let label = format!(
                    "In [{}]",
                    cell.execution_count
                        .map_or_else(|| " ".to_owned(), |c| c.to_string())
                );
                let code_lines =
                    wrap_text(&cell.source, page_width - 2.0 * MARGIN, "/F2", 9.0);
                let output_lines: Vec<String> = cell
                    .outputs
                    .iter()
                    .flat_map(|output| render_output_to_pdf_lines(output, page_width - 2.0 * MARGIN))
                    .collect();

                let total_lines = 1 + code_lines.len() + output_lines.len();
                let needed = (total_lines as f64) * CODE_LINE_HEIGHT + SECTION_GAP;
                if !content.has_room(needed) {
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
                    content = PageContent::new(page_height);
                }

                content.set_font("/F3", 10.0);
                content.text(MARGIN, content.y, &label);
                content.move_down(CODE_LINE_HEIGHT);

                content.set_font("/F2", 9.0);
                for line in &code_lines {
                    content.text(MARGIN, content.y, line);
                    content.move_down(CODE_LINE_HEIGHT);
                }

                if !output_lines.is_empty() {
                    content.set_font("/F1", 9.0);
                    for line in &output_lines {
                        content.text(MARGIN, content.y, line);
                        content.move_down(CODE_LINE_HEIGHT);
                    }
                }
                content.move_down(SECTION_GAP - CODE_LINE_HEIGHT);
            }
            CellType::Raw => {
                let raw_lines =
                    wrap_text(&cell.source, page_width - 2.0 * MARGIN, "/F2", 9.0);
                let needed = (raw_lines.len() as f64) * CODE_LINE_HEIGHT + SECTION_GAP;
                if !content.has_room(needed) {
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
                    content = PageContent::new(page_height);
                }
                content.set_font("/F2", 9.0);
                for line in &raw_lines {
                    content.text(MARGIN, content.y, line);
                    content.move_down(CODE_LINE_HEIGHT);
                }
                content.move_down(SECTION_GAP - CODE_LINE_HEIGHT);
            }
        }
    }

    // Save final page content
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
    writer.write_object_content(
        catalog_obj,
        format!("<< /Type /Catalog /Pages {} 0 R >>", pages_obj),
    );

    writer.finish()
}

fn render_output_to_pdf_lines(output: &Output, max_width: f64) -> Vec<String> {
    match output {
        Output::Text { text } => {
            let trimmed = text.trim_end_matches('\n');
            if trimmed.is_empty() {
                return Vec::new();
            }
            wrap_text(trimmed, max_width, "/F1", 9.0)
        }
        Output::Error {
            name,
            message,
            traceback,
        } => {
            let mut lines = vec![format!("**{name}**")];
            lines.extend(wrap_text(message, max_width, "/F1", 9.0));
            for trace in traceback {
                lines.extend(wrap_text(trace, max_width, "/F1", 9.0));
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
                wrap_text(text, max_width, "/F1", 9.0)
            }
        }
        Output::Image { mime_type, data } => {
            vec![format!("[Image: {mime_type}, {} bytes]", data.len())]
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
                    return vec![format!("[MIME image: {mime_type}, {} bytes]", value.len())];
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
                return wrap_text(text, max_width, "/F1", 9.0);
            }
            vec!["[MIME bundle output]".to_owned()]
        }
    }
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
        assert!(content.contains("1 + 1"));
        assert!(content.contains("2"));
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
            .add_cell(Cell::new(CellType::Markdown, "Content"))
            .expect("unique ID");

        let pdf = render_pdf_notebook(&notebook, PaperSize::A4);
        let content = String::from_utf8_lossy(&pdf);
        assert!(content.contains("My Report"));
    }
}
