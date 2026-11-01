pub(crate) fn escape_markdown_cell(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('|', "\\|")
        .replace("\r\n", "<br>")
        .replace(['\r', '\n'], "<br>")
}

pub(crate) fn fenced_code(language: &str, content: &str) -> String {
    let fence = code_fence(content);
    format!(
        "{fence}{language}\n{}\n{fence}",
        content.trim_end_matches('\n')
    )
}

pub(crate) fn code_fence(content: &str) -> String {
    let mut longest = 0;
    let mut run = 0;
    for character in content.chars() {
        if character == '`' {
            run += 1;
            longest = longest.max(run);
        } else {
            run = 0;
        }
    }
    "`".repeat(longest.max(2) + 1)
}
