use mars_core::Output;
use serde::Serialize;
use std::time::Duration;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StreamPhase {
    Compiling,
    Running,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChunkStream {
    Stdout,
    Stderr,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    Phase {
        phase: StreamPhase,
    },
    Chunk {
        stream: ChunkStream,
        data: String,
    },
    Prompt {
        message: String,
    },
    /// Jupyter-style comm traffic (widget state sync) published while a
    /// cell executes. Only interpreter backends with comm support emit this.
    Comm {
        message: mars_core::CommMessage,
    },
}

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

pub fn strip_prompt_protocol(stdout: &str) -> String {
    const MARKER: &str = "__MARS_PROMPT__";
    let mut output = String::with_capacity(stdout.len());
    let mut rest = stdout;
    while let Some(index) = rest.find(MARKER) {
        output.push_str(&rest[..index]);
        let after = &rest[index + MARKER.len()..];
        match after.find('\n') {
            Some(newline) => {
                output.push('\n');
                rest = &after[newline + 1..];
            }
            None => return output,
        }
    }
    output.push_str(rest);
    output
}

pub fn normalize_stdout(stdout: &str) -> String {
    const MARKER: &str = "__MARS_RESULT__=";
    strip_prompt_protocol(stdout)
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
        let (text, table_outputs) = extract_table_outputs(&text);
        if !text.is_empty() {
            outputs.push(Output::Text { text });
        }
        outputs.extend(table_outputs);
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

/// Rewrite `*.rs:LINE:COL` references in runtime stderr with Project Mars cell
/// annotations (`[mars:cell=HASH,line=N]`) so the UI can navigate to the
/// originating cell. `program` is the generated source that produced the
/// binary; `// Project Mars cell` markers delimit each cell's line range.
pub fn annotate_source_locations(stderr: &str, program: &str) -> String {
    let markers: Vec<(usize, u32)> = program
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let position = line.find("// Project Mars cell ")?;
            let rest = &line[position + "// Project Mars cell ".len()..];
            let hash = rest.split_whitespace().last()?.parse::<u32>().ok()?;
            Some((index, hash))
        })
        .collect();
    if markers.is_empty() || !stderr.contains(".rs:") {
        return stderr.to_owned();
    }

    let bytes = stderr.as_bytes();
    let mut output = String::with_capacity(stderr.len() + 64);
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        // Find the next ".rs:" occurrence.
        let window = &stderr[cursor..];
        let Some(rel) = window.find(".rs:") else {
            output.push_str(&stderr[cursor..]);
            break;
        };
        let match_start = cursor + rel;
        let digits_start = match_start + ".rs:".len();
        let line_end = stderr[digits_start..]
            .find(|c: char| !c.is_ascii_digit())
            .map(|offset| digits_start + offset)
            .unwrap_or(stderr.len());
        if line_end == digits_start {
            output.push_str(&stderr[cursor..digits_start]);
            cursor = digits_start;
            continue;
        }
        let Ok(panic_line) = stderr[digits_start..line_end].parse::<usize>() else {
            output.push_str(&stderr[cursor..line_end]);
            cursor = line_end;
            continue;
        };
        // Expect ":COL" after the line number.
        let after_line = &stderr[line_end..];
        let Some(col_part) = after_line.strip_prefix(':') else {
            output.push_str(&stderr[cursor..line_end]);
            cursor = line_end;
            continue;
        };
        let col_len = col_part
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(col_part.len());
        if col_len == 0 {
            output.push_str(&stderr[cursor..line_end]);
            cursor = line_end;
            continue;
        }
        let location_end = line_end + 1 + col_len;

        // Map the 1-based program line back to a cell via the markers.
        let panic_index = panic_line.saturating_sub(1);
        let cell = markers
            .iter()
            .rev()
            .find(|(marker_index, _)| *marker_index < panic_index)
            .map(|(marker_index, hash)| (*hash, panic_index - marker_index));

        output.push_str(&stderr[cursor..location_end]);
        if let Some((hash, local_line)) = cell {
            output.push_str(&format!(" [mars:cell={hash},line={local_line}]"));
        }
        cursor = location_end;
    }
    output
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

const TABLE_MARKER: &str = "__MARS_TABLE__=";
const HTML_MARKER: &str = "__MARS_HTML__";
const LATEX_MARKER: &str = "__MARS_LATEX__";
const IMAGE_MARKER: &str = "__MARS_IMAGE__=";
const CHART_MARKER: &str = "__MARS_CHART__";
const MIME_MARKER: &str = "__MARS_MIME__=";

/// Pull `__MARS_*__` marker lines (emitted by the injected `mars_*` display
/// helpers) out of stdout text and convert them into rich `Output` values.
fn extract_table_outputs(text: &str) -> (String, Vec<Output>) {
    let mut kept = Vec::new();
    let mut outputs = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim_end();
        if let Some(output) = parse_marker_line(trimmed) {
            outputs.push(output);
            continue;
        }
        kept.push(line);
    }
    let mut remaining = kept.join("\n");
    if text.ends_with('\n') && !remaining.is_empty() {
        remaining.push('\n');
    }
    (remaining, outputs)
}

fn parse_marker_line(line: &str) -> Option<Output> {
    if let Some(json) = line.strip_prefix(TABLE_MARKER) {
        return parse_table_json(json);
    }
    if let Some(json) = line.strip_prefix(HTML_MARKER) {
        let html: String = serde_json::from_str(json).ok()?;
        return Some(Output::Html { html });
    }
    if let Some(json) = line.strip_prefix(LATEX_MARKER) {
        let latex: String = serde_json::from_str(json).ok()?;
        return Some(Output::Latex { latex });
    }
    if let Some(json) = line.strip_prefix(IMAGE_MARKER) {
        let value: serde_json::Value = serde_json::from_str(json).ok()?;
        let mime_type = value.get("mime_type")?.as_str()?.to_owned();
        let data = value.get("data")?.as_str()?.to_owned();
        return Some(Output::Image { mime_type, data });
    }
    if let Some(json) = line.strip_prefix(CHART_MARKER) {
        let spec: serde_json::Value = serde_json::from_str(json).ok()?;
        return Some(Output::Chart { spec });
    }
    if let Some(json) = line.strip_prefix(MIME_MARKER) {
        let value: serde_json::Value = serde_json::from_str(json).ok()?;
        let serde_json::Value::Object(data) = value else {
            return None;
        };
        return Some(Output::MimeBundle {
            data,
            metadata: serde_json::Map::new(),
        });
    }
    None
}

fn parse_table_json(json: &str) -> Option<Output> {
    let value: serde_json::Value = serde_json::from_str(json).ok()?;
    let columns = value
        .get("columns")?
        .as_array()?
        .iter()
        .map(|entry| entry.as_str().map(str::to_owned))
        .collect::<Option<Vec<_>>>()?;
    let rows = value
        .get("rows")?
        .as_array()?
        .iter()
        .map(|row| {
            row.as_array()?
                .iter()
                .map(|cell| cell.as_str().map(str::to_owned))
                .collect::<Option<Vec<_>>>()
        })
        .collect::<Option<Vec<_>>>()?;
    Some(Output::Table { columns, rows })
}
