use mars_core::Output;
use std::time::Duration;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionResult {
    pub outputs: Vec<Output>,
    pub duration: Duration,
    pub execution_count: u64,
    pub interrupted: bool,
}

#[derive(Clone, Copy)]
pub enum ExpressionOutput {
    Debug,
    Display,
    Discard,
}

#[derive(Clone, Debug)]
pub struct BinaryOutput {
    pub status: Option<std::process::ExitStatus>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub interrupted: bool,
}

pub fn normalize_stdout(stdout: &str) -> String {
    const MARKER: &str = "__MARS_RESULT__=";
    stdout
        .lines()
        .map(|line| line.strip_prefix(MARKER).unwrap_or(line))
        .collect::<Vec<_>>()
        .join("\n")
        + if stdout.ends_with('\n') { "\n" } else { "" }
}

pub fn parse_evcxr_content(stdout: &str) -> (String, Vec<Output>) {
    const BEGIN: &str = "EVCXR_BEGIN_CONTENT";
    const END: &str = "EVCXR_END_CONTENT";

    let lines = stdout.lines().collect::<Vec<_>>();
    let mut text_lines = Vec::new();
    let mut outputs = Vec::new();
    let mut index = 0;

    while index < lines.len() {
        let line = lines[index];
        if let Some(mime_type) = line.trim_start().strip_prefix(BEGIN) {
            let mime_type = mime_type.trim();
            if !mime_type.is_empty() {
                let terminator = lines[index + 1..]
                    .iter()
                    .position(|candidate| candidate.trim() == END);
                if let Some(relative) = terminator {
                    let end = index + 1 + relative;
                    let content = lines[index + 1..end].join("\n");
                    outputs.push(content_output(mime_type, content));
                    index = end + 1;
                    continue;
                }
            }
        }
        text_lines.push(line);
        index += 1;
    }

    let mut text = text_lines.join("\n");
    if stdout.ends_with('\n') && !text.is_empty() {
        text.push('\n');
    }
    (text, outputs)
}

fn content_output(mime_type: &str, content: String) -> Output {
    match mime_type {
        "text/plain" => Output::Text { text: content },
        "text/html" => match crate::output::extract_standalone_svg(&content) {
            Some(svg) => mime_bundle("image/svg+xml", svg, "Figure (SVG)"),
            None => Output::Html { html: content },
        },
        mime if mime.starts_with("image/") => mime_bundle(mime, &content, "Notebook image"),
        mime => {
            let mut data = serde_json::Map::new();
            data.insert(mime.to_owned(), serde_json::Value::String(content));
            Output::MimeBundle {
                data,
                metadata: serde_json::Map::new(),
            }
        }
    }
}

fn mime_bundle(mime_type: &str, content: &str, alt_text: &str) -> Output {
    let mut data = serde_json::Map::new();
    data.insert(
        mime_type.to_owned(),
        serde_json::Value::String(content.to_owned()),
    );
    data.insert(
        "text/plain".to_owned(),
        serde_json::Value::String(alt_text.to_owned()),
    );
    Output::MimeBundle {
        data,
        metadata: serde_json::Map::new(),
    }
}

pub fn extract_standalone_svg(html: &str) -> Option<&str> {
    let lowercase = html.to_ascii_lowercase();
    let start = lowercase.find("<svg")?;
    let open_end = start + lowercase[start..].find('>')? + 1;
    let end = if lowercase[start..open_end].trim_end().ends_with("/>") {
        open_end
    } else {
        let after_open = &lowercase[open_end..];
        open_end + after_open.find("</svg>")? + "</svg>".len()
    };

    if crate::output::has_visible_text(&html[..start])
        || crate::output::has_visible_text(&html[end..])
    {
        return None;
    }

    Some(&html[start..end])
}

pub fn has_visible_text(fragment: &str) -> bool {
    let mut text = String::new();
    let mut in_tag = false;
    for character in fragment.chars() {
        match character {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => text.push(character),
            _ => {}
        }
    }
    !text.trim().is_empty()
}

pub fn execution_outputs(stdout: &[u8], stderr: &[u8], error_name: &str) -> Vec<Output> {
    let mut outputs = Vec::new();
    if !stdout.is_empty() {
        let normalized = normalize_stdout(&String::from_utf8_lossy(stdout));
        let (text, content_outputs) = parse_evcxr_content(&normalized);
        if !text.is_empty() {
            outputs.push(Output::Text { text });
        }
        outputs.extend(content_outputs);
    }
    if !stderr.is_empty() {
        outputs.push(Output::Error {
            name: error_name.to_owned(),
            message: String::from_utf8_lossy(stderr).into_owned(),
            traceback: Vec::new(),
        });
    }
    outputs
}

pub fn interrupted_result(
    duration: Duration,
    execution_count: u64,
    message: &str,
) -> ExecutionResult {
    ExecutionResult {
        outputs: vec![Output::Error {
            name: "Interrupted".to_owned(),
            message: message.to_owned(),
            traceback: Vec::new(),
        }],
        duration,
        execution_count,
        interrupted: true,
    }
}
