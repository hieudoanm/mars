use mars_core::Output;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};
use uuid::Uuid;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionResult {
    pub outputs: Vec<Output>,
    pub duration: Duration,
    pub execution_count: u64,
    pub interrupted: bool,
}

/// Common execution surface used by the kernel to select a runtime backend.
pub trait RuntimeBackend: Send {
    /// Execute one cell while honoring caller cancellation and optional timeout.
    fn execute_with_control(
        &mut self,
        source: &str,
        timeout: Option<Duration>,
        cancellation: Arc<AtomicBool>,
    ) -> Result<ExecutionResult, RuntimeError>;

    /// Clear session state and reset the execution count.
    fn reset(&mut self) -> Result<(), RuntimeError>;

    /// Number of successfully executed cells in this session.
    fn execution_count(&self) -> u64;

    /// Return a lockfile when the backend supports notebook lockfile persistence.
    fn cargo_lock(&self) -> Result<Option<String>, RuntimeError>;
}

pub struct RustSession {
    workdir: PathBuf,
    cells: Vec<String>,
    execution_count: u64,
    dependencies: BTreeMap<String, String>,
    cargo_lock: Option<String>,
    offline: bool,
    locked: bool,
}

#[derive(Clone, Debug, Default)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }

    pub fn flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.cancelled)
    }
}

impl RustSession {
    pub fn new() -> Result<Self, RuntimeError> {
        Self::with_dependencies(BTreeMap::new())
    }

    pub fn with_dependencies(dependencies: BTreeMap<String, String>) -> Result<Self, RuntimeError> {
        Self::with_configuration(dependencies, None, false, false)
    }

    pub fn with_configuration(
        dependencies: BTreeMap<String, String>,
        cargo_lock: Option<String>,
        offline: bool,
        locked: bool,
    ) -> Result<Self, RuntimeError> {
        validate_dependencies(&dependencies)?;
        let workdir = std::env::temp_dir().join(format!("mars-runtime-{}", Uuid::new_v4()));
        fs::create_dir(&workdir).map_err(|source| RuntimeError::Io {
            path: workdir.clone(),
            source,
        })?;
        Ok(Self {
            workdir,
            cells: Vec::new(),
            execution_count: 0,
            dependencies,
            cargo_lock,
            offline,
            locked,
        })
    }

    pub fn execute(&mut self, source: &str) -> Result<ExecutionResult, RuntimeError> {
        self.execute_with_control(source, None, Arc::new(AtomicBool::new(false)))
    }

    pub fn execute_with_timeout(
        &mut self,
        source: &str,
        timeout: Option<Duration>,
    ) -> Result<ExecutionResult, RuntimeError> {
        self.execute_with_control(source, timeout, Arc::new(AtomicBool::new(false)))
    }

    pub fn execute_with_control(
        &mut self,
        source: &str,
        timeout: Option<Duration>,
        cancellation: Arc<AtomicBool>,
    ) -> Result<ExecutionResult, RuntimeError> {
        if source.trim().is_empty() {
            return Err(RuntimeError::EmptySource);
        }
        if cancellation.load(Ordering::Relaxed) {
            return Ok(interrupted_result(
                Duration::ZERO,
                self.execution_count,
                "execution cancelled before compilation",
            ));
        }

        let started = Instant::now();
        let candidate_cells = self
            .cells
            .iter()
            .cloned()
            .chain(std::iter::once(source.to_owned()))
            .collect::<Vec<_>>();
        let source_path = self.workdir.join("session.rs");
        let binary_path = self.workdir.join("session-bin");
        let compile_options = CompileOptions {
            dependencies: &self.dependencies,
            cargo_lock: self.cargo_lock.as_deref(),
            offline: self.offline,
            locked: self.locked,
            timeout: remaining_timeout(started, timeout),
            cancellation: &cancellation,
        };
        let mut compile = compile_program(
            &self.workdir,
            &source_path,
            &binary_path,
            render_program(&candidate_cells),
            &compile_options,
        )?;
        if !compile.status.is_some_and(|status| status.success())
            && split_final_expression(source).is_some()
        {
            let display_compile = compile_program(
                &self.workdir,
                &source_path,
                &binary_path,
                render_program_with_last_output(&candidate_cells, ExpressionOutput::Display),
                &CompileOptions {
                    timeout: remaining_timeout(started, timeout),
                    ..compile_options
                },
            )?;
            if display_compile
                .status
                .is_some_and(|status| status.success())
            {
                compile = display_compile;
            }
        }
        if compile.interrupted {
            return Ok(interrupted_result(
                started.elapsed(),
                self.execution_count,
                interruption_message(&cancellation, "compilation"),
            ));
        }
        if !compile.status.is_some_and(|status| status.success()) {
            return Ok(ExecutionResult {
                outputs: vec![Output::Error {
                    name: "CompilationError".to_owned(),
                    message: format_compiler_diagnostics(&compile.stderr),
                    traceback: Vec::new(),
                }],
                duration: started.elapsed(),
                execution_count: self.execution_count,
                interrupted: false,
            });
        }

        let execution_path = if self.dependencies.is_empty() {
            binary_path.clone()
        } else {
            self.workdir
                .join("target")
                .join("debug")
                .join("mars-session")
        };
        let execution_child = configured_command(&execution_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(RuntimeError::Execution)?;
        let run = run_child(
            execution_child,
            remaining_timeout(started, timeout),
            &cancellation,
        )?;
        if run.interrupted {
            return Ok(interrupted_result(
                started.elapsed(),
                self.execution_count,
                interruption_message(&cancellation, "execution"),
            ));
        }
        if !run.status.is_some_and(|status| status.success()) {
            return Ok(ExecutionResult {
                outputs: execution_outputs(&run.stdout, &run.stderr, "RuntimeError"),
                duration: started.elapsed(),
                execution_count: self.execution_count,
                interrupted: false,
            });
        }

        self.cells.push(source.to_owned());
        self.execution_count += 1;
        Ok(ExecutionResult {
            outputs: execution_outputs(&run.stdout, &run.stderr, "RuntimeError"),
            duration: started.elapsed(),
            execution_count: self.execution_count,
            interrupted: false,
        })
    }

    pub fn reset(&mut self) {
        self.cells.clear();
        self.execution_count = 0;
    }

    pub fn execution_count(&self) -> u64 {
        self.execution_count
    }

    pub fn cargo_lock(&self) -> Result<Option<String>, RuntimeError> {
        let path = self.workdir.join("Cargo.lock");
        match fs::read_to_string(path) {
            Ok(lock) => Ok(Some(lock)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(source) => Err(RuntimeError::Io {
                path: self.workdir.join("Cargo.lock"),
                source,
            }),
        }
    }
}

impl RuntimeBackend for RustSession {
    fn execute_with_control(
        &mut self,
        source: &str,
        timeout: Option<Duration>,
        cancellation: Arc<AtomicBool>,
    ) -> Result<ExecutionResult, RuntimeError> {
        RustSession::execute_with_control(self, source, timeout, cancellation)
    }

    fn reset(&mut self) -> Result<(), RuntimeError> {
        RustSession::reset(self);
        Ok(())
    }

    fn execution_count(&self) -> u64 {
        RustSession::execution_count(self)
    }

    fn cargo_lock(&self) -> Result<Option<String>, RuntimeError> {
        RustSession::cargo_lock(self)
    }
}

struct BinaryOutput {
    status: Option<std::process::ExitStatus>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    interrupted: bool,
}

fn run_child(
    mut child: Child,
    timeout: Option<Duration>,
    cancellation: &AtomicBool,
) -> Result<BinaryOutput, RuntimeError> {
    let started = Instant::now();

    loop {
        if let Some(status) = child.try_wait().map_err(RuntimeError::Execution)? {
            let output = child.wait_with_output().map_err(RuntimeError::Execution)?;
            return Ok(BinaryOutput {
                status: Some(status),
                stdout: output.stdout,
                stderr: output.stderr,
                interrupted: false,
            });
        }

        if cancellation.load(Ordering::Relaxed)
            || timeout.is_some_and(|limit| started.elapsed() >= limit)
        {
            terminate_process_tree(&mut child)?;
            let _ = child.wait().map_err(RuntimeError::Execution)?;
            return Ok(BinaryOutput {
                status: None,
                stdout: Vec::new(),
                stderr: Vec::new(),
                interrupted: true,
            });
        }

        thread::sleep(Duration::from_millis(10));
    }
}

fn terminate_process_tree(child: &mut Child) -> Result<(), RuntimeError> {
    let pid = child.id();

    #[cfg(unix)]
    {
        let result = unsafe { libc::kill(-(pid as libc::pid_t), libc::SIGKILL) };
        if result == 0 {
            return Ok(());
        }
    }

    #[cfg(windows)]
    {
        let status = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .status()
            .map_err(RuntimeError::Execution)?;
        if status.success() {
            return Ok(());
        }
    }

    child.kill().map_err(RuntimeError::Execution)
}

fn configured_command(program: impl AsRef<std::ffi::OsStr>) -> Command {
    let mut command = Command::new(program);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    command
}

fn interruption_message(cancellation: &AtomicBool, phase: &str) -> &'static str {
    if cancellation.load(Ordering::Relaxed) {
        match phase {
            "compilation" => "compilation cancelled",
            _ => "execution cancelled",
        }
    } else {
        match phase {
            "compilation" => "compilation exceeded the configured time limit",
            _ => "execution exceeded the configured time limit",
        }
    }
}

fn remaining_timeout(started: Instant, timeout: Option<Duration>) -> Option<Duration> {
    timeout.map(|limit| limit.saturating_sub(started.elapsed()))
}

fn interrupted_result(duration: Duration, execution_count: u64, message: &str) -> ExecutionResult {
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

impl Drop for RustSession {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.workdir);
    }
}

#[derive(Clone, Copy)]
enum ExpressionOutput {
    Debug,
    Display,
}

fn render_program(cells: &[String]) -> String {
    render_program_with_last_output(cells, ExpressionOutput::Debug)
}

fn render_program_with_last_output(cells: &[String], output: ExpressionOutput) -> String {
    let mut program = String::from("fn main() {\n");
    for (index, cell) in cells.iter().enumerate() {
        program.push_str(&format!("    // Mars cell {index}\n"));
        let cell_output = if index + 1 == cells.len() {
            output
        } else {
            ExpressionOutput::Debug
        };
        let rendered_cell = render_cell_with_output(cell, cell_output);
        for line in rendered_cell.lines() {
            program.push_str("    ");
            program.push_str(line);
            program.push('\n');
        }
    }
    program.push_str("}\n");
    program
}

fn compile_command(
    workdir: &Path,
    source_path: &Path,
    binary_path: &Path,
    dependencies: &BTreeMap<String, String>,
    cargo_lock: Option<&str>,
    offline: bool,
    locked: bool,
) -> Result<(String, Vec<String>), RuntimeError> {
    if dependencies.is_empty() {
        return Ok((
            "rustc".to_owned(),
            vec![
                "--edition=2024".to_owned(),
                "--error-format=json".to_owned(),
                source_path.to_string_lossy().into_owned(),
                "-o".to_owned(),
                binary_path.to_string_lossy().into_owned(),
            ],
        ));
    }

    let source_directory = workdir.join("src");
    fs::create_dir_all(&source_directory).map_err(|source| RuntimeError::Io {
        path: source_directory.clone(),
        source,
    })?;
    fs::write(workdir.join("Cargo.toml"), cargo_manifest(dependencies)).map_err(|source| {
        RuntimeError::Io {
            path: workdir.join("Cargo.toml"),
            source,
        }
    })?;
    if let Some(lock) = cargo_lock {
        fs::write(workdir.join("Cargo.lock"), lock).map_err(|source| RuntimeError::Io {
            path: workdir.join("Cargo.lock"),
            source,
        })?;
    }
    fs::copy(source_path, source_directory.join("main.rs")).map_err(|source| RuntimeError::Io {
        path: source_directory.join("main.rs"),
        source,
    })?;
    let mut args = vec![
        "build".to_owned(),
        "--manifest-path".to_owned(),
        workdir.join("Cargo.toml").to_string_lossy().into_owned(),
    ];
    if offline {
        args.push("--offline".to_owned());
    }
    if locked {
        args.push("--locked".to_owned());
    }
    Ok(("cargo".to_owned(), args))
}

struct CompileOptions<'a> {
    dependencies: &'a BTreeMap<String, String>,
    cargo_lock: Option<&'a str>,
    offline: bool,
    locked: bool,
    timeout: Option<Duration>,
    cancellation: &'a AtomicBool,
}

fn compile_program(
    workdir: &Path,
    source_path: &Path,
    binary_path: &Path,
    program: String,
    options: &CompileOptions<'_>,
) -> Result<BinaryOutput, RuntimeError> {
    fs::write(source_path, program).map_err(|source| RuntimeError::Io {
        path: source_path.to_path_buf(),
        source,
    })?;
    let (compiler, compiler_args) = compile_command(
        workdir,
        source_path,
        binary_path,
        options.dependencies,
        options.cargo_lock,
        options.offline,
        options.locked,
    )?;
    let child = configured_command(compiler)
        .args(compiler_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(RuntimeError::CompilerUnavailable)?;
    run_child(child, options.timeout, options.cancellation)
}

fn format_compiler_diagnostics(stderr: &[u8]) -> String {
    let raw = String::from_utf8_lossy(stderr);
    let mut diagnostics = Vec::new();
    for line in raw.lines() {
        let Ok(value) = serde_json::from_str::<serde_json::Value>(line) else {
            diagnostics.push(line.to_owned());
            continue;
        };
        if value
            .get("reason")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|reason| reason != "compiler-message")
            && value.get("message").is_none()
        {
            continue;
        }
        let Some(message) = value.get("message").or(Some(&value)) else {
            continue;
        };
        let level = message
            .get("level")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("error");
        let text = message
            .get("message")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("compiler diagnostic");
        if let Some(code) = message
            .get("code")
            .and_then(|code| code.get("code"))
            .and_then(serde_json::Value::as_str)
        {
            diagnostics.push(format!("{level} [{code}]: {text}"));
        } else {
            diagnostics.push(format!("{level}: {text}"));
        }
        if let Some(spans) = message.get("spans").and_then(serde_json::Value::as_array) {
            for span in spans {
                let kind =
                    if span.get("is_primary").and_then(serde_json::Value::as_bool) == Some(true) {
                        "primary"
                    } else {
                        "secondary"
                    };
                let file = span
                    .get("file_name")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("source");
                let line_start = span
                    .get("line_start")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0);
                let column_start = span
                    .get("column_start")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0);
                let line_end = span
                    .get("line_end")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(line_start);
                let column_end = span
                    .get("column_end")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(column_start);
                let label = span
                    .get("label")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("");
                diagnostics.push(format!(
                    "  --> {file}:{line_start}:{column_start}-{line_end}:{column_end} [{kind}] {label}"
                ));
            }
        }
        if let Some(rendered) = message.get("rendered").and_then(serde_json::Value::as_str) {
            let rendered = rendered.trim();
            if !rendered.is_empty() && rendered != text {
                diagnostics.push(format!("  note: {rendered}"));
            }
        }
    }
    if diagnostics.is_empty() {
        raw.into_owned()
    } else {
        diagnostics.join("\n")
    }
}

fn cargo_manifest(dependencies: &BTreeMap<String, String>) -> String {
    let mut manifest = String::from(
        "[package]\nname = \"mars-session\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\n",
    );
    for (name, version) in dependencies {
        manifest.push_str(name);
        manifest.push_str(" = ");
        if let Some(raw) = toml_dependency_value(version) {
            manifest.push_str(raw);
        } else {
            manifest.push_str(&toml_string(version));
        }

        manifest.push('\n');
    }

    fn toml_dependency_value(value: &str) -> Option<&str> {
        let trimmed = value.trim();
        if trimmed.starts_with('{') && trimmed.ends_with('}') && !trimmed.contains('\n') {
            Some(trimmed)
        } else {
            None
        }
    }
    manifest
}

pub fn validate_dependencies(dependencies: &BTreeMap<String, String>) -> Result<(), RuntimeError> {
    for (name, specification) in dependencies {
        if name.is_empty()
            || !name.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '-' | '_')
            })
            || specification.trim().is_empty()
        {
            return Err(RuntimeError::InvalidDependency { name: name.clone() });
        }
    }
    Ok(())
}

fn toml_string(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn render_cell_with_output(source: &str, output: ExpressionOutput) -> String {
    let stripped = strip_trailing_line_comments(source);
    if let Some((prefix, expression)) = split_final_expression(&stripped) {
        let formatter = match output {
            ExpressionOutput::Debug => "{:?}",
            ExpressionOutput::Display => "{}",
        };
        return format!("{prefix}println!(\"__MARS_RESULT__={formatter}\", ({expression}));");
    }
    source.to_owned()
}

fn split_final_expression(source: &str) -> Option<(&str, &str)> {
    let trimmed = source.trim_end();
    let semicolon = find_last_top_level_semicolon(trimmed);
    let brace = find_last_top_level_closing_brace(trimmed).filter(|index| {
        let suffix = trimmed[index + 1..].trim_start();
        !suffix.is_empty() && !suffix.starts_with(';')
    });
    let boundary = match (semicolon, brace) {
        (Some(semicolon), Some(brace)) => Some(semicolon.max(brace)),
        (Some(semicolon), None) => Some(semicolon),
        (None, Some(brace)) => Some(brace),
        (None, None) => None,
    };
    let (prefix, expression) = boundary.map_or(("", trimmed), |index| trimmed.split_at(index + 1));
    let expression = expression.trim();
    if expression.is_empty() || !is_expression_candidate(expression) {
        return None;
    }
    Some((prefix, expression))
}

fn is_expression_candidate(expression: &str) -> bool {
    let expression = expression.trim();
    let excluded_prefixes = [
        "dbg!",
        "eprintln!",
        "fn ",
        "impl ",
        "let ",
        "macro_rules!",
        "mod ",
        "panic!",
        "println!",
        "return ",
        "struct ",
        "todo!",
        "trait ",
        "type ",
        "use ",
    ];
    !excluded_prefixes
        .iter()
        .any(|prefix| expression.starts_with(prefix))
        && !expression.ends_with(';')
}

fn find_last_top_level_semicolon(source: &str) -> Option<usize> {
    let mut depth = 0_i32;
    let mut string = false;
    let mut escaped = false;
    let mut last = None;
    for (index, character) in source.char_indices() {
        if string {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                string = false;
            }
            continue;
        }
        match character {
            '"' => string = true,
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            ';' if depth == 0 => last = Some(index),
            _ => {}
        }
    }
    last
}

fn find_last_top_level_closing_brace(source: &str) -> Option<usize> {
    let mut depth = 0_i32;
    let mut string = false;
    let mut escaped = false;
    let mut last = None;
    for (index, character) in source.char_indices() {
        if string {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                string = false;
            }
            continue;
        }
        match character {
            '"' => string = true,
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    last = Some(index);
                }
            }
            _ => {}
        }
    }
    last
}

fn strip_trailing_line_comments(source: &str) -> String {
    let mut output = String::with_capacity(source.len());
    let mut string = false;
    let mut escaped = false;
    let mut chars = source.chars().peekable();
    while let Some(character) = chars.next() {
        if string {
            output.push(character);
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                string = false;
            }
            continue;
        }
        if character == '"' {
            string = true;
            output.push(character);
        } else if character == '/' && chars.peek() == Some(&'/') {
            let _ = chars.next();
            for trailing in chars.by_ref() {
                if trailing == '\n' {
                    output.push('\n');
                    break;
                }
            }
        } else {
            output.push(character);
        }
    }
    output
}

fn execution_outputs(stdout: &[u8], stderr: &[u8], error_name: &str) -> Vec<Output> {
    let mut outputs = Vec::new();
    if !stdout.is_empty() {
        outputs.push(Output::Text {
            text: normalize_stdout(&String::from_utf8_lossy(stdout)),
        });
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

fn normalize_stdout(stdout: &str) -> String {
    const MARKER: &str = "__MARS_RESULT__=";
    stdout
        .lines()
        .map(|line| line.strip_prefix(MARKER).unwrap_or(line))
        .collect::<Vec<_>>()
        .join("\n")
        + if stdout.ends_with('\n') { "\n" } else { "" }
}

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("cannot execute an empty code cell")]
    EmptySource,
    #[error("invalid Cargo dependency declaration: {name}")]
    InvalidDependency { name: String },
    #[error("failed to access runtime file {path}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("rustc is unavailable")]
    CompilerUnavailable(#[source] std::io::Error),
    #[error("failed to start compiled notebook program")]
    Execution(#[source] std::io::Error),
    #[error("runtime backend failed: {0}")]
    Backend(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_bindings_between_cells() {
        let mut session = RustSession::new().expect("runtime directory");
        session
            .execute(r#"let numbers = vec![1, 2, 3];"#)
            .expect("first cell");
        let result = session
            .execute(r#"println!("{}", numbers.iter().sum::<i32>());"#)
            .expect("second cell");

        assert_eq!(result.execution_count, 2);
        assert_eq!(
            result.outputs,
            vec![Output::Text {
                text: "6\n".to_owned()
            }]
        );
    }

    #[test]
    fn captures_a_final_expression_result() {
        let mut session = RustSession::new().expect("runtime directory");
        let result = session.execute("let numbers = vec![1, 2, 3];\nnumbers.len()");

        assert_eq!(
            result.expect("execution result").outputs,
            vec![Output::Text {
                text: "3\n".to_owned()
            }]
        );
    }

    #[test]
    fn captures_multiline_expression_results() {
        let mut session = RustSession::new().expect("runtime directory");
        let result = session
            .execute("let values = vec![1, 2, 3];\nvalues\n    .iter()\n    .sum::<i32>()")
            .expect("execution result");

        assert_eq!(
            result.outputs,
            vec![Output::Text {
                text: "6\n".to_owned()
            }]
        );
    }

    #[test]
    fn ignores_trailing_comments_when_extracting_results() {
        let mut session = RustSession::new().expect("runtime directory");
        let result = session
            .execute("let answer = 42;\nanswer // show the result")
            .expect("execution result");

        assert_eq!(
            result.outputs,
            vec![Output::Text {
                text: "42\n".to_owned()
            }]
        );
    }

    #[test]
    fn falls_back_to_display_for_non_debug_values() {
        let mut session = RustSession::new().expect("runtime directory");
        session
            .execute("vec![1, 2, 3]")
            .expect("debug-only prior value");
        let result = session
            .execute(
                r#"
struct DisplayOnly;
impl std::fmt::Display for DisplayOnly {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "display-only")
    }
}
DisplayOnly
"#,
            )
            .expect("execution result");

        assert_eq!(
            result.outputs,
            vec![Output::Text {
                text: "[1, 2, 3]\ndisplay-only\n".to_owned()
            }]
        );
    }

    #[test]
    fn preserves_explicit_output_without_duplicate_expression_output() {
        let mut session = RustSession::new().expect("runtime directory");
        let result = session
            .execute(r#"println!("hello");"#)
            .expect("execution result");

        assert_eq!(
            result.outputs,
            vec![Output::Text {
                text: "hello\n".to_owned()
            }]
        );
    }

    #[test]
    fn does_not_commit_failed_cells() {
        let mut session = RustSession::new().expect("runtime directory");
        let result = session.execute("let value: = 1;");

        assert_eq!(session.execution_count(), 0);
        assert!(matches!(
            result.expect("compiler result").outputs.first(),
            Some(Output::Error { name, .. }) if name == "CompilationError"
        ));
    }

    #[test]
    fn formats_structured_diagnostics_with_all_labeled_spans() {
        let stderr = br#"{"reason":"compiler-message","message":{"message":"mismatched types","level":"error","code":{"code":"E0308"},"spans":[{"file_name":"session.rs","line_start":2,"column_start":5,"line_end":2,"column_end":8,"is_primary":true,"label":"expected integer"},{"file_name":"session.rs","line_start":3,"column_start":1,"line_end":3,"column_end":4,"is_primary":false,"label":"found here"}]}}"#;
        let formatted = format_compiler_diagnostics(stderr);

        assert!(formatted.contains("error [E0308]: mismatched types"));
        assert!(formatted.contains("[primary] expected integer"));
        assert!(formatted.contains("[secondary] found here"));
    }

    #[test]
    fn interrupts_long_running_cells() {
        let mut session = RustSession::new().expect("runtime directory");
        let result = session
            .execute_with_timeout(
                "std::thread::sleep(std::time::Duration::from_millis(100));",
                Some(Duration::from_millis(10)),
            )
            .expect("execution result");

        assert!(result.interrupted);
        assert_eq!(session.execution_count(), 0);
        assert!(matches!(
            result.outputs.first(),
            Some(Output::Error { name, .. }) if name == "Interrupted"
        ));
    }

    #[test]
    fn cancels_long_running_cells() {
        let mut session = RustSession::new().expect("runtime directory");
        let cancellation = Arc::new(AtomicBool::new(false));
        let signal = Arc::clone(&cancellation);
        let worker = thread::spawn(move || {
            session
                .execute_with_control(
                    "std::thread::sleep(std::time::Duration::from_millis(500));",
                    None,
                    cancellation,
                )
                .expect("execution result")
        });

        thread::sleep(Duration::from_millis(25));
        signal.store(true, Ordering::Relaxed);
        let result = worker.join().expect("worker completes");

        assert!(result.interrupted);
        assert!(result.outputs.iter().any(|output| {
            matches!(
                output,
                Output::Error { name, message, .. }
                    if name == "Interrupted" && message.contains("cancelled")
            )
        }));
    }

    #[test]
    fn cancellation_token_can_cancel_before_start() {
        let token = CancellationToken::new();
        token.cancel();
        let mut session = RustSession::new().expect("runtime directory");
        let result = session
            .execute_with_control("println!(\"not run\");", None, token.flag())
            .expect("execution result");

        assert!(result.interrupted);
        assert!(result.outputs.iter().any(|output| {
            matches!(output, Output::Error { message, .. } if message.contains("before compilation"))
        }));
    }

    #[test]
    fn replay_repeats_prior_side_effects_by_design() {
        let mut session = RustSession::new().expect("runtime directory");
        session
            .execute(r#"println!("side effect"); let value = 7;"#)
            .expect("first cell");
        let result = session.execute("value").expect("second cell");

        assert_eq!(
            result.outputs,
            vec![Output::Text {
                text: "side effect\n7\n".to_owned()
            }]
        );
    }

    #[test]
    fn writes_a_cargo_manifest_for_declared_dependencies() {
        let mut dependencies = BTreeMap::new();
        dependencies.insert("serde".to_owned(), "1".to_owned());
        let session = RustSession::with_dependencies(dependencies).expect("runtime directory");
        let manifest = cargo_manifest(&session.dependencies);

        assert!(manifest.contains("[dependencies]"));
        assert!(manifest.contains("serde = \"1\""));
    }

    #[test]
    fn writes_richer_cargo_dependency_specs() {
        let mut dependencies = BTreeMap::new();
        dependencies.insert(
            "serde".to_owned(),
            "{ version = \"1\", features = [\"derive\"] }".to_owned(),
        );
        assert!(
            cargo_manifest(&dependencies)
                .contains("serde = { version = \"1\", features = [\"derive\"] }")
        );
    }

    #[test]
    fn rejects_invalid_dependency_names() {
        let mut dependencies = BTreeMap::new();
        dependencies.insert("bad name".to_owned(), "1".to_owned());
        assert!(matches!(
            RustSession::with_dependencies(dependencies),
            Err(RuntimeError::InvalidDependency { .. })
        ));
    }
}
