use crate::RustSession;
use crate::compilation::CompileOptions;
use crate::error::RuntimeError;
use crate::execution::{configured_command, remaining_timeout};
use crate::expression::render_cell_with_output;
use crate::output::ExpressionOutput;
use serde::Serialize;
use serde_json::Value;
use std::io::Write;
use std::process::Stdio;
use std::sync::atomic::AtomicBool;
use std::time::Instant;

const VAR_MARKER: &str = "__MARS_VAR__\t";
const MAX_VALUE_PREVIEW: usize = 160;
const MAX_DIAGNOSTICS: usize = 50;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VariableInfo {
    pub name: String,
    pub type_name: Option<String>,
    pub value: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CellDiagnostic {
    pub cell_index: Option<usize>,
    pub line: u32,
    pub column: u32,
    pub end_line: u32,
    pub end_column: u32,
    pub severity: String,
    pub code: Option<String>,
    pub message: String,
}

/// Extracts simple `let name` bindings from cell sources. Destructuring
/// patterns and `let` inside strings are approximated away; this is a
/// best-effort list used to instrument the replayed program.
#[doc(hidden)]
pub fn let_bindings(cells: &[String]) -> Vec<String> {
    let mut names = Vec::new();
    for cell in cells {
        let bytes = cell.as_bytes();
        let mut index = 0;
        while index + 4 <= bytes.len() {
            if &cell[index..index + 4] == "let " {
                let before_ok =
                    index == 0 || matches!(bytes[index - 1], b'\n' | b';' | b'{' | b'}' | b' ');
                let mut cursor = index + 4;
                if cell[cursor..].starts_with("mut ") {
                    cursor += 4;
                }
                let rest = &cell[cursor..];
                let ident: String = rest
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                let valid_ident = ident
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_alphabetic() || c == '_')
                    && ident.chars().all(|c| c.is_alphanumeric() || c == '_')
                    && !matches!(ident.as_str(), "_" | "mut");
                let ident_len = ident.len();
                if before_ok && valid_ident && !names.contains(&ident) {
                    names.push(ident);
                }
                index = cursor + ident_len;
                continue;
            }
            index += 1;
        }
    }
    names
}

#[doc(hidden)]
pub fn render_inspect_program(cells: &[String], names: &[String], with_values: bool) -> String {
    let mut program = String::from("fn main() {\n");
    for (index, cell) in cells.iter().enumerate() {
        program.push_str(&format!("    // Mars cell {index}\n"));
        let rendered = render_cell_with_output(cell, ExpressionOutput::Discard);
        for line in rendered.lines() {
            program.push_str("    ");
            program.push_str(line);
            program.push('\n');
        }
    }
    for name in names {
        if with_values {
            program.push_str(&format!(
                "    {{ let __mars_v = format!(\"{{:?}}\", &{name}); \
                 let __mars_v: String = __mars_v.chars().take({MAX_VALUE_PREVIEW}).collect(); \
                 println!(\"{VAR_MARKER}{{}}\\t{{}}\\t{{}}\", \"{name}\", \
                 std::any::type_name_of_val(&{name}), __mars_v); }}\n"
            ));
        } else {
            program.push_str(&format!(
                "    println!(\"{VAR_MARKER}{{}}\\t{{}}\", \"{name}\", \
                 std::any::type_name_of_val(&{name}));\n"
            ));
        }
    }
    program.push_str("}\n");
    program
}

fn parse_variables(stdout: &[u8]) -> Vec<VariableInfo> {
    let text = String::from_utf8_lossy(stdout);
    let mut variables = Vec::new();
    for line in text.lines() {
        let Some(rest) = line.strip_prefix(VAR_MARKER) else {
            continue;
        };
        let mut fields = rest.splitn(3, '\t');
        let (Some(name), Some(type_name)) = (fields.next(), fields.next()) else {
            continue;
        };
        let value = fields.next().map(str::to_owned);
        variables.push(VariableInfo {
            name: name.to_owned(),
            type_name: Some(type_name.to_owned()),
            value,
        });
    }
    variables
}

fn run_program(
    session: &RustSession,
    program: String,
    started: Instant,
) -> Result<std::process::Output, RuntimeError> {
    let source_path = session.workdir.join("session.rs");
    let binary_path = session.workdir.join("session-bin");
    let cancellation = AtomicBool::new(false);
    let options = CompileOptions {
        dependencies: &session.dependencies,
        cargo_lock: session.cargo_lock.as_deref(),
        offline: session.offline,
        locked: session.locked,
        timeout: remaining_timeout(started, None),
        cancellation: &cancellation,
    };
    let compile = crate::compilation::compile_program(
        &session.workdir,
        &source_path,
        &binary_path,
        program,
        &options,
    )?;
    if !compile.status.is_some_and(|status| status.success()) {
        return Err(RuntimeError::Backend(
            "inspection program failed to compile".to_owned(),
        ));
    }
    let execution_path = if session.dependencies.is_empty() {
        binary_path
    } else {
        session
            .workdir
            .join("target")
            .join("debug")
            .join("mars-session")
    };
    configured_command(&execution_path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .map_err(RuntimeError::Execution)
}

impl RustSession {
    /// Replays all committed cells with an appended inspection block and
    /// returns live bindings (name, type, Debug preview when available).
    /// Side effects of cells run again; this is a documented limitation of
    /// the replay architecture.
    pub fn inspect_variables(&mut self) -> Result<Vec<VariableInfo>, RuntimeError> {
        let names = let_bindings(&self.cells);
        if names.is_empty() {
            return Ok(Vec::new());
        }
        let started = Instant::now();
        match run_program(
            self,
            render_inspect_program(&self.cells, &names, true),
            started,
        ) {
            Ok(output) if output.status.success() => Ok(parse_variables(&output.stdout)),
            _ => {
                // Fall back to type-only inspection when any binding lacks Debug.
                let started = Instant::now();
                match run_program(
                    self,
                    render_inspect_program(&self.cells, &names, false),
                    started,
                ) {
                    Ok(output) if output.status.success() => Ok(parse_variables(&output.stdout)),
                    _ => Ok(Vec::new()),
                }
            }
        }
    }

    /// Compiles the committed cells plus `source` without running and maps
    /// rustc diagnostics back to cell indices and cell-relative positions.
    pub fn diagnose(&mut self, source: &str) -> Result<Vec<CellDiagnostic>, RuntimeError> {
        if source.trim().is_empty() {
            return Ok(Vec::new());
        }
        let mut candidate = self.cells.clone();
        candidate.push(source.to_owned());
        let program = crate::expression::render_program_with_last_output(
            &candidate,
            ExpressionOutput::Discard,
        );
        let started = Instant::now();
        let source_path = self.workdir.join("session.rs");
        let binary_path = self.workdir.join("session-bin");
        let cancellation = AtomicBool::new(false);
        let options = CompileOptions {
            dependencies: &self.dependencies,
            cargo_lock: self.cargo_lock.as_deref(),
            offline: self.offline,
            locked: self.locked,
            timeout: remaining_timeout(started, None),
            cancellation: &cancellation,
        };
        let compile = crate::compilation::compile_program(
            &self.workdir,
            &source_path,
            &binary_path,
            program.clone(),
            &options,
        )?;
        if compile.status.is_some_and(|status| status.success()) {
            return Ok(Vec::new());
        }
        Ok(map_diagnostics(&program, &compile.stderr, &source_path))
    }

    /// Writes a line to the stdin of the currently running cell program.
    pub fn write_stdin(&mut self, text: &str) -> Result<(), RuntimeError> {
        let stdin = self
            .stdin
            .as_ref()
            .ok_or_else(|| RuntimeError::Backend("no active execution".to_owned()))?;
        let mut guard = stdin
            .lock()
            .map_err(|_| RuntimeError::Backend("kernel input lock poisoned".to_owned()))?;
        writeln!(guard, "{text}").map_err(|_| RuntimeError::Backend("stdin closed".to_owned()))?;
        guard
            .flush()
            .map_err(|_| RuntimeError::Backend("stdin flush failed".to_owned()))
    }
}

/// Maps `// Mars cell N` marker lines in the generated program to cell
/// indices so rustc line numbers become (cell, line-in-cell) pairs.
fn cell_line_map(program: &str) -> Vec<(usize, usize)> {
    program
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let marker = line.trim().strip_prefix("// Mars cell ")?;
            marker
                .trim()
                .parse::<usize>()
                .ok()
                .map(|cell| (index + 1, cell))
        })
        .collect()
}

fn diagnostic_object(value: &Value) -> Option<&Value> {
    if value.get("$message_type").and_then(Value::as_str) == Some("diagnostic") {
        return Some(value);
    }
    if value.get("reason").and_then(Value::as_str) == Some("compiler-message") {
        return value.get("message").filter(|message| message.is_object());
    }
    None
}

fn map_diagnostics(
    program: &str,
    stderr: &[u8],
    source_path: &std::path::Path,
) -> Vec<CellDiagnostic> {
    let markers = cell_line_map(program);
    let expected_file = source_path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let text = String::from_utf8_lossy(stderr);
    let mut diagnostics = Vec::new();
    for line in text.lines() {
        let Ok(value) = serde_json::from_str::<Value>(line.trim()) else {
            continue;
        };
        let Some(diagnostic) = diagnostic_object(&value) else {
            continue;
        };
        let level = diagnostic
            .get("level")
            .and_then(Value::as_str)
            .unwrap_or("error");
        if level == "note" || level == "help" || level == "failure-note" {
            continue;
        }
        let message = diagnostic
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("compiler diagnostic")
            .to_owned();
        let code = diagnostic
            .get("code")
            .and_then(|code| code.get("code"))
            .and_then(Value::as_str)
            .map(str::to_owned);
        let Some(spans) = diagnostic.get("spans").and_then(Value::as_array) else {
            continue;
        };
        for span in spans {
            if span.get("is_primary").and_then(Value::as_bool) != Some(true) {
                continue;
            }
            let file_name = span
                .get("file_name")
                .and_then(Value::as_str)
                .unwrap_or_default();
            if !file_name.is_empty()
                && file_name != expected_file
                && !file_name.ends_with("session.rs")
                && !file_name.ends_with("main.rs")
            {
                continue;
            }
            let line_start = span.get("line_start").and_then(Value::as_u64).unwrap_or(0) as usize;
            if line_start == 0 {
                continue;
            }
            // marker lines are 1-based program lines recorded as (line, cell)
            let cell_index = markers
                .iter()
                .rev()
                .find(|(marker_line, _)| *marker_line < line_start)
                .map(|(_, cell)| *cell);
            let marker = markers
                .iter()
                .rev()
                .find(|(marker_line, _)| *marker_line < line_start);
            let offset = marker
                .map(|(marker_line, _)| line_start - marker_line)
                .unwrap_or(line_start);
            let line_end = span
                .get("line_end")
                .and_then(Value::as_u64)
                .unwrap_or(line_start as u64) as usize;
            let end_offset = markers
                .iter()
                .rev()
                .find(|(marker_line, _)| *marker_line < line_end)
                .map(|(marker_line, _)| line_end - marker_line)
                .unwrap_or(line_end);
            diagnostics.push(CellDiagnostic {
                cell_index,
                line: offset.max(1) as u32,
                column: span
                    .get("column_start")
                    .and_then(Value::as_u64)
                    .unwrap_or(1) as u32,
                end_line: end_offset.max(1) as u32,
                end_column: span.get("column_end").and_then(Value::as_u64).unwrap_or(1) as u32,
                severity: if level == "warning" {
                    "warning".to_owned()
                } else {
                    "error".to_owned()
                },
                code: code.clone(),
                message: message.clone(),
            });
            if diagnostics.len() >= MAX_DIAGNOSTICS {
                return diagnostics;
            }
        }
        if diagnostics.is_empty() && spans.is_empty() {
            diagnostics.push(CellDiagnostic {
                cell_index: None,
                line: 1,
                column: 1,
                end_line: 1,
                end_column: 1,
                severity: if level == "warning" {
                    "warning".to_owned()
                } else {
                    "error".to_owned()
                },
                code,
                message,
            });
        }
    }
    diagnostics
}
