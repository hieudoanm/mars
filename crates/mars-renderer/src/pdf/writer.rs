use std::io::Write as IoWrite;

pub(crate) const MARGIN: f64 = 54.0; // 0.75 inch in points
pub(crate) const LINE_HEIGHT: f64 = 12.5;
pub(crate) const CODE_LINE_HEIGHT: f64 = 12.0;
pub(crate) const BLOCK_GAP: f64 = 16.0; // uniform gap between every heading, paragraph, code block, output
pub(crate) const PARAGRAPH_GAP: f64 = 8.0; // gap between consecutive paragraphs and list items
pub(crate) const BOX_PAD: f64 = 14.0;
pub(crate) const LINE_NUM_WIDTH: f64 = 28.0;

/// PDF object writer. Tracks object numbers and writes objects to a buffer.
pub(crate) struct PdfWriter {
    objects: Vec<String>,
    is_stream: Vec<bool>,
    xref_positions: Vec<usize>,
}

impl PdfWriter {
    pub(crate) fn new() -> Self {
        Self {
            objects: Vec::new(),
            is_stream: Vec::new(),
            xref_positions: Vec::new(),
        }
    }

    pub(crate) fn allocate(&mut self) -> usize {
        self.objects.push(String::new());
        self.is_stream.push(false);
        self.objects.len()
    }

    /// Allocates a stream object (written with /Length and stream/endstream).
    pub(crate) fn allocate_stream(&mut self) -> usize {
        self.objects.push(String::new());
        self.is_stream.push(true);
        self.objects.len()
    }

    pub(crate) fn write_object_content(&mut self, obj_num: usize, content: String) {
        let slot = &mut self.objects[obj_num - 1];
        slot.push_str(&content);
    }

    pub(crate) fn finish(mut self) -> Vec<u8> {
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
