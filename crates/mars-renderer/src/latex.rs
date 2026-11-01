use crate::util::escape_html;

pub fn render_latex_static(latex: &str) -> String {
    let mut rendered = String::new();
    let mut cursor = 0;
    while let Some(relative) = latex[cursor..].find("\\frac{") {
        let start = cursor + relative;
        rendered.push_str(&escape_html(&latex[cursor..start]));
        let numerator_start = start + "\\frac{".len();
        let Some((numerator, after_numerator)) = braced_group(latex, numerator_start) else {
            rendered.push_str(&escape_html(&latex[start..]));
            return rendered;
        };
        let denominator_start = after_numerator + 1;
        if !latex[denominator_start..].starts_with('{') {
            rendered.push_str(&escape_html(&latex[start..]));
            return rendered;
        }
        let Some((denominator, after_denominator)) = braced_group(latex, denominator_start + 1)
        else {
            rendered.push_str(&escape_html(&latex[start..]));
            return rendered;
        };
        rendered.push_str(&format!(
            "<span class=\"math-frac\"><span>{}</span><span>{}</span></span>",
            escape_html(numerator),
            escape_html(denominator)
        ));
        cursor = after_denominator + 1;
    }
    rendered.push_str(&escape_html(&latex[cursor..]));
    rendered
}

pub fn braced_group(source: &str, start: usize) -> Option<(&str, usize)> {
    let bytes = source.as_bytes();
    let mut depth = 1;
    for index in start..bytes.len() {
        match bytes[index] {
            b'{' => depth += 1,
            b'}' if depth == 1 => return Some((&source[start..index], index)),
            b'}' if depth > 1 => depth -= 1,
            _ => {}
        }
    }
    None
}
