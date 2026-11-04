use crate::pdf::writer::MARGIN;

/// Minimal PDF content stream generator for a single page.
///
/// Drawing operations (rects for code boxes, line numbers, page furniture)
/// are emitted outside the `BT ... ET` text block; text operations go inside it.
pub(crate) struct PageContent {
    drawing: String,
    text: String,
    pub(crate) y: f64,
    _page_height: f64,
    bottom_margin: f64,
    font: &'static str,
    font_size: f64,
    color: (f64, f64, f64),
    page_number: usize,
}

impl PageContent {
    pub(crate) fn new(page_height: f64, page_number: usize) -> Self {
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

    pub(crate) fn has_room(&self, needed: f64) -> bool {
        self.y - needed > self.bottom_margin
    }

    pub(crate) fn rect_fill(&mut self, x: f64, w: f64, h: f64, color: (f64, f64, f64)) {
        let (r, g, b) = color;
        // re x y w h, then rg r g b, then f
        self.drawing.push_str(&format!(
            "{} {} {} {} re {} {} {} rg f\n",
            x,
            self.y - h,
            w,
            h,
            r,
            g,
            b
        ));
    }

    pub(crate) fn rect_stroke(&mut self, x: f64, w: f64, h: f64, color: (f64, f64, f64), wdt: f64) {
        let (r, g, b) = color;
        self.drawing.push_str(&format!(
            "{} w {} {} {} {} re {} {} {} RG S\n",
            wdt,
            x,
            self.y - h,
            w,
            h,
            r,
            g,
            b
        ));
    }

    pub(crate) fn set_font(&mut self, font: &'static str, size: f64) {
        if self.font != font || self.font_size != size {
            self.text.push_str(&format!("{} {} Tf\n", font, size));
            self.font = font;
            self.font_size = size;
        }
    }

    pub(crate) fn set_color(&mut self, color: (f64, f64, f64)) {
        if self.color != color {
            let (r, g, b) = color;
            self.text.push_str(&format!("{} {} {} rg\n", r, g, b));
            self.color = color;
        }
    }

    pub(crate) fn text(&mut self, x: f64, y: f64, text: &str) {
        let escaped = crate::pdf::tokens::pdf_escape(text);
        self.text
            .push_str(&format!("1 0 0 1 {} {} Tm ({}) Tj\n", x, y, escaped));
    }

    pub(crate) fn move_down(&mut self, amount: f64) {
        self.y -= amount;
    }

    /// Draw page footer with page number at the bottom.
    pub(crate) fn draw_footer(&mut self, page_width: f64) {
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
            MARGIN, footer_text_y, self.page_number
        ));
    }

    pub(crate) fn render_commands(&mut self) -> String {
        let mut commands = std::mem::take(&mut self.drawing);
        commands.push_str(&std::mem::take(&mut self.text));
        commands.push_str("ET\n");
        commands
    }
}
