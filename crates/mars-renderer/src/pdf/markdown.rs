use pulldown_cmark::{Options, Parser};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MdLineType {
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
    Normal,
}

pub struct MdLine {
    pub text: String,
    pub kind: MdLineType,
}

pub fn render_markdown_to_pdf(source: &str) -> Vec<MdLine> {
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
                        && close_tag.chars().nth(1).and_then(|c| c.to_digit(10)) == heading_level
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
