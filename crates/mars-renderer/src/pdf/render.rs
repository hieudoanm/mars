use crate::pdf::cell_code::render_code_cell;
use crate::pdf::cell_markdown::render_markdown_cell;
use crate::pdf::cell_raw::render_raw_cell;
use crate::pdf::markdown::MdLineType;
use crate::pdf::page::PageContent;
use crate::pdf::writer::{LINE_HEIGHT, PdfWriter};
use mars_core::{CellType, PaperSize};
use mars_document::Notebook;

pub(crate) struct PdfRenderContext {
    pub(crate) writer: PdfWriter,
    pub(crate) resources: usize,
    pub(crate) pages_obj: usize,
    pub(crate) content: PageContent,
    pub(crate) content_nums: Vec<usize>,
    pub(crate) page_nums: Vec<usize>,
    pub(crate) page_number: usize,
    pub(crate) page_width: f64,
    pub(crate) page_height: f64,
}

impl PdfRenderContext {
    pub(crate) fn new(page_width: f64, page_height: f64) -> Self {
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

        let content = PageContent::new(page_height, 1);

        Self {
            writer,
            resources,
            pages_obj,
            content,
            content_nums: Vec::new(),
            page_nums: Vec::new(),
            page_number: 1,
            page_width,
            page_height,
        }
    }

    pub(crate) fn new_page(&mut self) {
        self.content.draw_footer(self.page_width);
        let cn = self.writer.allocate_stream();
        self.writer
            .write_object_content(cn, self.content.render_commands());
        self.content_nums.push(cn);
        let page = self.writer.allocate();
        self.writer.write_object_content(
            page,
            format!(
                "<< /Type /Page /Parent {} 0 R /Resources {} 0 R /Contents {} 0 R /MediaBox [0 0 {} {}] >>",
                self.pages_obj,
                self.resources,
                self.content_nums[self.content_nums.len() - 1],
                self.page_width,
                self.page_height
            ),
        );
        self.page_nums.push(page);
        self.page_number += 1;
        self.content = PageContent::new(self.page_height, self.page_number);
    }

    fn finish(mut self) -> Vec<u8> {
        self.content.draw_footer(self.page_width);
        let cn = self.writer.allocate_stream();
        self.writer
            .write_object_content(cn, self.content.render_commands());
        self.content_nums.push(cn);
        let page = self.writer.allocate();
        self.writer.write_object_content(
            page,
            format!(
                "<< /Type /Page /Parent {} 0 R /Resources {} 0 R /Contents {} 0 R /MediaBox [0 0 {} {}] >>",
                self.pages_obj,
                self.resources,
                self.content_nums[self.content_nums.len() - 1],
                self.page_width,
                self.page_height
            ),
        );
        self.page_nums.push(page);

        let kids = self
            .page_nums
            .iter()
            .map(|n| format!("{} 0 R", n))
            .collect::<Vec<_>>()
            .join(" ");
        self.writer.write_object_content(
            self.pages_obj,
            format!(
                "<< /Type /Pages /Kids [{kids}] /Count {} >>",
                self.page_nums.len()
            ),
        );

        let catalog_obj = self.writer.allocate();
        self.writer.write_object_content(
            catalog_obj,
            format!("<< /Type /Catalog /Pages {} 0 R >>", self.pages_obj),
        );

        self.writer.finish()
    }
}

pub fn render_pdf_notebook(notebook: &Notebook, page_size: PaperSize) -> Vec<u8> {
    let (page_width, page_height) = page_size.dimensions();
    let mut ctx = PdfRenderContext::new(page_width, page_height);

    for cell in &notebook.cells {
        match cell.cell_type {
            CellType::Markdown => {
                render_markdown_cell(&mut ctx, cell);
            }
            CellType::Code => {
                render_code_cell(&mut ctx, cell);
            }
            CellType::Raw => {
                render_raw_cell(&mut ctx, cell);
            }
        }
    }

    ctx.finish()
}

pub(crate) fn line_height(kind: MdLineType) -> f64 {
    match kind {
        MdLineType::H1 => LINE_HEIGHT * 1.6,
        MdLineType::H2 => LINE_HEIGHT * 1.4,
        MdLineType::H3 => LINE_HEIGHT * 1.25,
        _ => LINE_HEIGHT,
    }
}

pub(crate) fn font_for_line(kind: MdLineType) -> (&'static str, f64) {
    match kind {
        MdLineType::H1 => ("/F3", 18.0),
        MdLineType::H2 => ("/F3", 15.0),
        MdLineType::H3 => ("/F3", 13.0),
        MdLineType::H4 => ("/F3", 12.0),
        MdLineType::H5 => ("/F1", 11.0),
        MdLineType::H6 => ("/F1", 10.0),
        MdLineType::Normal => ("/F1", 11.0),
    }
}

pub(crate) fn is_image_line(text: &str) -> bool {
    text.starts_with("[Image") && text.ends_with("]")
}
