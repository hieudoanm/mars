use mars_core::Output;
use mars_runtime::execution::{configured_command, interruption_message, terminate_process_tree};
use mars_runtime::{
    CellDiagnostic, ChunkStream, ExecutionResult, RuntimeBackend, RuntimeError, StdinWriter,
    StreamEvent, StreamPhase, VariableInfo,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::ffi::OsString;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const DRIVER_SOURCE: &str = include_str!("driver.py");
const STDERR_TAIL_LIMIT: usize = 64 * 1024;
const CONTROL_TIMEOUT: Duration = Duration::from_secs(30);

/// Per-notebook Python configuration, typically parsed from the notebook
/// metadata `python` section (`metadata.python` in `.mars`/`.ipynb`).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PythonOptions {
    /// Explicit interpreter path or executable name. Wins over `MARS_PYTHON`
    /// and version discovery.
    #[serde(default)]
    pub interpreter: Option<String>,
    /// Version hint such as `"3.12"` used to discover a matching interpreter
    /// (`python3.12`, pyenv/uv installs) when no explicit interpreter is set.
    #[serde(default)]
    pub version: Option<String>,
    /// Requirements installed into the resolved interpreter on driver
    /// startup (equivalent to `pip install` before the first cell).
    #[serde(default)]
    pub packages: Vec<String>,
}

/// Persistent Python execution session backed by a long-lived driver
/// subprocess speaking newline-delimited JSON. Variables persist in the
/// driver's module namespace across cells.
///
/// Known limitations:
/// - Interactive `input()` works through the kernel input channel; a cell
///   blocked on input() still times out or cancels like any other cell.
/// - Matplotlib figures are captured as static PNGs (Agg backend).
/// - A timed-out or cancelled cell kills the driver; the next cell starts a
///   fresh interpreter and loses all state.
pub struct PythonSession {
    python: OsString,
    options: PythonOptions,
    workdir: PathBuf,
    driver_path: PathBuf,
    child: Option<Child>,
    stdin: Arc<Mutex<Option<ChildStdin>>>,
    lines: Option<Receiver<String>>,
    stderr_tail: Arc<Mutex<String>>,
    next_id: u64,
    execution_count: u64,
    executing: Arc<AtomicBool>,
}

impl PythonSession {
    pub fn new() -> Result<Self, RuntimeError> {
        Self::with_options(PythonOptions::default())
    }

    pub fn with_options(options: PythonOptions) -> Result<Self, RuntimeError> {
        let python = find_python(&options).ok_or_else(|| {
            RuntimeError::Backend(
                "no Python interpreter found; install python3 or set MARS_PYTHON".to_owned(),
            )
        })?;
        let workdir = std::env::temp_dir().join(format!("mars-python-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&workdir).map_err(|source| RuntimeError::Io {
            path: workdir.clone(),
            source,
        })?;
        let driver_path = workdir.join("driver.py");
        if let Err(source) = fs::write(&driver_path, DRIVER_SOURCE) {
            return Err(RuntimeError::Io {
                path: driver_path.clone(),
                source,
            });
        }
        Ok(Self {
            python,
            options,
            workdir,
            driver_path,
            child: None,
            stdin: Arc::new(Mutex::new(None)),
            lines: None,
            stderr_tail: Arc::new(Mutex::new(String::new())),
            next_id: 1,
            execution_count: 0,
            executing: Arc::new(AtomicBool::new(false)),
        })
    }

    fn ensure_spawn(&mut self) -> Result<(), RuntimeError> {
        if let Some(child) = self.child.as_mut() {
            if matches!(child.try_wait(), Ok(None)) {
                return Ok(());
            }
        }
        self.kill_child();
        *stderr_tail_lock(&self.stderr_tail) = String::new();

        let mut command = configured_command(&self.python);
        command
            .arg("-u")
            .arg(&self.driver_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if std::env::var_os("MPLBACKEND").is_none() {
            command.env("MPLBACKEND", "Agg");
        }
        if !self.options.packages.is_empty()
            && let Ok(encoded) = serde_json::to_string(&self.options.packages)
        {
            command.env("MARS_PYTHON_PACKAGES", encoded);
        }
        let mut child = command.spawn().map_err(|source| {
            RuntimeError::Backend(format!("failed to start Python driver: {source}"))
        })?;
        let Some(stdin) = child.stdin.take() else {
            return Err(RuntimeError::Backend(
                "python driver stdin unavailable".to_owned(),
            ));
        };
        let Some(stdout) = child.stdout.take() else {
            return Err(RuntimeError::Backend(
                "python driver stdout unavailable".to_owned(),
            ));
        };
        let stderr = child.stderr.take();

        if let Some(stderr) = stderr {
            let tail = Arc::clone(&self.stderr_tail);
            thread::spawn(move || drain_stderr(stderr, tail));
        }
        let (sender, receiver) = mpsc::channel::<String>();
        thread::spawn(move || read_stdout_lines(stdout, sender));

        *driver_stdin_lock(&self.stdin) = None;
        self.child = Some(child);
        *driver_stdin_lock(&self.stdin) = Some(stdin);
        self.lines = Some(receiver);
        Ok(())
    }

    fn write_driver_line(&self, payload: &[u8]) -> Result<(), RuntimeError> {
        let mut guard = match self.stdin.lock() {
            Ok(guard) => guard,
            Err(_) => {
                return Err(RuntimeError::Backend(
                    "python driver stdin lock poisoned".to_owned(),
                ));
            }
        };
        let Some(stdin) = guard.as_mut() else {
            return Err(RuntimeError::Backend(
                "python driver stdin unavailable".to_owned(),
            ));
        };
        let write = stdin
            .write_all(payload)
            .and_then(|()| stdin.write_all(b"\n"))
            .and_then(|()| stdin.flush());
        if let Err(source) = write {
            return Err(RuntimeError::Backend(format!(
                "failed to write to python driver: {source}"
            )));
        }
        Ok(())
    }

    fn send(&mut self, message: &Value) -> Result<(), RuntimeError> {
        self.ensure_spawn()?;
        let payload = match serde_json::to_vec(message) {
            Ok(payload) => payload,
            Err(source) => {
                return Err(RuntimeError::Backend(format!(
                    "failed to encode request: {source}"
                )));
            }
        };
        self.write_driver_line(&payload)
    }

    fn kill_child(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = terminate_process_tree(&mut child);
            let _ = child.wait();
        }
        driver_stdin_lock(&self.stdin).take();
        self.lines = None;
    }

    fn driver_exit_error(&self) -> RuntimeError {
        let tail = stderr_tail_lock(&self.stderr_tail).clone();
        let detail = if tail.trim().is_empty() {
            "process exited".to_owned()
        } else {
            format!("process exited: {}", tail.trim())
        };
        RuntimeError::Backend(format!("python driver {detail}"))
    }

    /// Sends a request and reads protocol lines until the response with the
    /// matching id and expected type arrives.
    fn transact(
        &mut self,
        request: Value,
        expected_type: &str,
        timeout: Duration,
    ) -> Result<Value, RuntimeError> {
        let Some(id) = request.get("id").and_then(Value::as_u64) else {
            return Err(RuntimeError::Backend("request missing id".to_owned()));
        };
        self.send(&request)?;
        let Some(receiver) = self.lines.as_ref() else {
            return Err(RuntimeError::Backend(
                "python driver stdout unavailable".to_owned(),
            ));
        };
        let started = Instant::now();
        loop {
            match receiver.recv_timeout(Duration::from_millis(20)) {
                Ok(line) => {
                    let Ok(message) = serde_json::from_str::<Value>(&line) else {
                        continue;
                    };
                    if message.get("id").and_then(Value::as_u64) != Some(id) {
                        continue;
                    }
                    if message.get("type").and_then(Value::as_str) == Some(expected_type) {
                        return Ok(message);
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    return Err(self.driver_exit_error());
                }
            }
            if started.elapsed() >= timeout {
                return Err(RuntimeError::Backend(format!(
                    "python driver did not respond to {expected_type} request"
                )));
            }
            if let Some(child) = self.child.as_mut() {
                if !matches!(child.try_wait(), Ok(None)) {
                    return Err(self.driver_exit_error());
                }
            }
        }
    }

    fn next_request_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn execute_inner(
        &mut self,
        source: &str,
        timeout: Option<Duration>,
        cancellation: Arc<AtomicBool>,
        mut events: Option<&mut dyn FnMut(StreamEvent)>,
    ) -> Result<ExecutionResult, RuntimeError> {
        if source.trim().is_empty() {
            return Err(RuntimeError::EmptySource);
        }
        self.executing.store(true, Ordering::Relaxed);
        let _executing_guard = ExecutingGuard(Arc::clone(&self.executing));
        self.ensure_spawn()?;
        let id = self.next_request_id();
        let started = Instant::now();
        if let Some(sink) = events.as_deref_mut() {
            sink(StreamEvent::Phase {
                phase: StreamPhase::Running,
            });
        }
        self.send(&json!({"type": "execute", "id": id, "source": source}))?;

        let mut stdout_text = String::new();
        let mut stderr_text = String::new();
        let mut displays: Vec<Vec<(String, String)>> = Vec::new();
        let mut bundles: Vec<(
            serde_json::Map<String, Value>,
            serde_json::Map<String, Value>,
        )> = Vec::new();
        let mut error: Option<(String, String, Vec<String>)> = None;

        let Some(receiver) = self.lines.as_ref() else {
            return Err(RuntimeError::Backend(
                "python driver stdout unavailable".to_owned(),
            ));
        };
        loop {
            if cancellation.load(Ordering::Relaxed)
                || timeout.is_some_and(|limit| started.elapsed() >= limit)
            {
                self.kill_child();
                let message = interruption_message(&cancellation, "execution");
                return Ok(ExecutionResult {
                    outputs: vec![Output::Error {
                        name: "Interrupted".to_owned(),
                        message: message.to_owned(),
                        traceback: Vec::new(),
                    }],
                    duration: started.elapsed(),
                    execution_count: self.execution_count,
                    interrupted: true,
                });
            }
            match receiver.recv_timeout(Duration::from_millis(20)) {
                Ok(line) => {
                    let Ok(message) = serde_json::from_str::<Value>(&line) else {
                        continue;
                    };
                    if message.get("id").and_then(Value::as_u64) != Some(id) {
                        continue;
                    }
                    match message.get("type").and_then(Value::as_str) {
                        Some("stream") => {
                            let stream = message.get("stream").and_then(Value::as_str);
                            let data = message
                                .get("data")
                                .and_then(Value::as_str)
                                .unwrap_or_default()
                                .to_owned();
                            match stream {
                                Some("stdout") => stdout_text.push_str(&data),
                                Some("stderr") => stderr_text.push_str(&data),
                                _ => {}
                            }
                            if let Some(sink) = events.as_deref_mut() {
                                sink(StreamEvent::Chunk {
                                    stream: match stream {
                                        Some("stderr") => ChunkStream::Stderr,
                                        _ => ChunkStream::Stdout,
                                    },
                                    data,
                                });
                            }
                        }
                        Some("display") => {
                            if let Some(parts) = message.get("data").and_then(Value::as_array) {
                                displays.push(
                                    parts
                                        .iter()
                                        .filter_map(|part| {
                                            let pair = part.as_array()?;
                                            Some((
                                                pair.first()?.as_str()?.to_owned(),
                                                pair.get(1)?.as_str()?.to_owned(),
                                            ))
                                        })
                                        .collect(),
                                );
                            }
                        }
                        Some("mime") => {
                            if let Some(data) = message.get("data").and_then(Value::as_object) {
                                let metadata = message
                                    .get("metadata")
                                    .and_then(Value::as_object)
                                    .cloned()
                                    .unwrap_or_default();
                                bundles.push((data.clone(), metadata));
                            }
                        }
                        Some("input_request") => {
                            let prompt = message
                                .get("prompt")
                                .and_then(Value::as_str)
                                .unwrap_or_default()
                                .to_owned();
                            if let Some(sink) = events.as_deref_mut() {
                                sink(StreamEvent::Prompt { message: prompt });
                            }
                        }
                        Some("comm") => {
                            if let Some(message) = parse_comm_message(&message) {
                                if let Some(sink) = events.as_deref_mut() {
                                    sink(StreamEvent::Comm { message });
                                }
                            }
                        }
                        Some("result") => {
                            let ok = message.get("ok").and_then(Value::as_bool).unwrap_or(false);
                            if !ok {
                                if let Some(error_value) = message.get("error") {
                                    error = Some((
                                        error_value
                                            .get("name")
                                            .and_then(Value::as_str)
                                            .unwrap_or("Error")
                                            .to_owned(),
                                        error_value
                                            .get("message")
                                            .and_then(Value::as_str)
                                            .unwrap_or_default()
                                            .to_owned(),
                                        error_value
                                            .get("traceback")
                                            .and_then(Value::as_array)
                                            .map(|lines| {
                                                lines
                                                    .iter()
                                                    .filter_map(Value::as_str)
                                                    .map(str::to_owned)
                                                    .collect()
                                            })
                                            .unwrap_or_default(),
                                    ));
                                } else {
                                    error = Some((
                                        "Error".to_owned(),
                                        "cell execution failed".to_owned(),
                                        Vec::new(),
                                    ));
                                }
                            }
                            break;
                        }
                        _ => {}
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if let Some(child) = self.child.as_mut() {
                        if !matches!(child.try_wait(), Ok(None)) {
                            return Err(self.driver_exit_error());
                        }
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    return Err(self.driver_exit_error());
                }
            }
        }

        if error.is_none() {
            self.execution_count += 1;
        }
        let mut outputs = Vec::new();
        if !stdout_text.is_empty() {
            outputs.push(Output::Text { text: stdout_text });
        }
        outputs.extend(displays.iter().filter_map(|parts| display_output(parts)));
        outputs.extend(
            bundles
                .into_iter()
                .map(|(data, metadata)| Output::MimeBundle { data, metadata }),
        );
        // stderr is a stream, not a failure: warnings and diagnostics written
        // there must not mark the cell as failed (Jupyter semantics). Genuine
        // exceptions arrive via the driver's error result instead.
        if !stderr_text.is_empty() {
            outputs.push(Output::Text {
                text: format!("[stderr]\n{stderr_text}"),
            });
        }
        if let Some((name, message, traceback)) = error {
            outputs.push(Output::Error {
                name,
                message,
                traceback,
            });
        }
        Ok(ExecutionResult {
            outputs,
            duration: started.elapsed(),
            execution_count: self.execution_count,
            interrupted: false,
        })
    }
}

struct ExecutingGuard(Arc<AtomicBool>);

impl Drop for ExecutingGuard {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Relaxed);
    }
}

fn read_stdout_lines(stdout: ChildStdout, sender: mpsc::Sender<String>) {
    for line in BufReader::new(stdout).lines() {
        match line {
            Ok(text) => {
                if sender.send(text).is_err() {
                    break;
                }
            }
            Err(_) => break,
        }
    }
}

fn drain_stderr(mut stderr: impl std::io::Read, tail: Arc<Mutex<String>>) {
    let mut buffer = [0_u8; 8192];
    loop {
        match stderr.read(&mut buffer) {
            Ok(0) | Err(_) => break,
            Ok(read) => {
                let mut guard = stderr_tail_lock(&tail);
                guard.push_str(&String::from_utf8_lossy(&buffer[..read]));
                if guard.len() > STDERR_TAIL_LIMIT {
                    let excess = guard.len() - STDERR_TAIL_LIMIT;
                    guard.drain(..excess);
                }
            }
        }
    }
}

fn stderr_tail_lock(tail: &Mutex<String>) -> std::sync::MutexGuard<'_, String> {
    match tail.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn driver_stdin_lock(
    slot: &Mutex<Option<ChildStdin>>,
) -> std::sync::MutexGuard<'_, Option<ChildStdin>> {
    match slot.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn parse_comm_message(message: &Value) -> Option<mars_core::CommMessage> {
    Some(mars_core::CommMessage {
        comm_id: message.get("comm_id")?.as_str()?.to_owned(),
        comm_type: message
            .get("comm_type")
            .and_then(Value::as_str)
            .unwrap_or("comm_msg")
            .to_owned(),
        target_name: message
            .get("target_name")
            .and_then(Value::as_str)
            .map(str::to_owned),
        data: message.get("data").cloned().unwrap_or(Value::Null),
        metadata: message.get("metadata").cloned().unwrap_or(Value::Null),
    })
}

fn display_output(parts: &[(String, String)]) -> Option<Output> {
    for (mime, data) in parts {
        match mime.as_str() {
            "text/html" => return Some(Output::Html { html: data.clone() }),
            "image/png" | "image/jpeg" | "image/svg+xml" => {
                return Some(Output::Image {
                    mime_type: mime.clone(),
                    data: data.clone(),
                });
            }
            _ => {}
        }
    }
    if let Some((_, data)) = parts.iter().find(|(mime, _)| mime == "text/plain") {
        return Some(Output::Text { text: data.clone() });
    }
    if parts.is_empty() {
        return None;
    }
    let mut data = serde_json::Map::new();
    for (mime, value) in parts {
        data.insert(mime.clone(), Value::String(value.clone()));
    }
    Some(Output::MimeBundle {
        data,
        metadata: serde_json::Map::new(),
    })
}

fn probe_python(candidate: &OsString) -> bool {
    matches!(
        configured_command(candidate)
            .arg("--version")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status(),
        Ok(status) if status.success()
    )
}

/// Interpreter resolution order: explicit `interpreter` option, then
/// `MARS_PYTHON`, then version discovery, then PATH probing.
fn find_python(options: &PythonOptions) -> Option<OsString> {
    if let Some(interpreter) = options.interpreter.as_deref()
        && !interpreter.is_empty()
    {
        let candidate = OsString::from(interpreter);
        return probe_python(&candidate).then_some(candidate);
    }
    if let Some(custom) = std::env::var_os("MARS_PYTHON") {
        return Some(custom);
    }
    if let Some(version) = options.version.as_deref()
        && let Some(found) = find_python_version(version)
    {
        return Some(found);
    }
    for candidate in ["python3", "python", "py"] {
        let candidate = OsString::from(candidate);
        if probe_python(&candidate) {
            return Some(candidate);
        }
    }
    None
}

/// Finds an interpreter matching a version hint like `"3.12"` by probing
/// `python<version>` executables and scanning pyenv/uv install directories.
fn find_python_version(version: &str) -> Option<OsString> {
    let normalized = version.trim().trim_start_matches('v');
    if normalized.is_empty() {
        return None;
    }
    for name in [
        format!("python{normalized}"),
        format!("python{}", normalized.replace('.', "")),
    ] {
        let candidate = OsString::from(name);
        if probe_python(&candidate) {
            return Some(candidate);
        }
    }
    let mut roots = Vec::new();
    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
        roots.push(home.join(".pyenv/versions"));
        roots.push(home.join(".local/share/uv/python"));
    }
    for root in roots {
        let Ok(entries) = fs::read_dir(&root) else {
            continue;
        };
        for entry in entries.flatten() {
            let dir_name = entry.file_name().to_string_lossy().into_owned();
            if !dir_name.contains(normalized) {
                continue;
            }
            for bin_name in ["bin/python3", "bin/python"] {
                let candidate = entry.path().join(bin_name);
                if candidate.is_file() {
                    let candidate_os = candidate.into_os_string();
                    if probe_python(&candidate_os) {
                        return Some(candidate_os);
                    }
                }
            }
        }
    }
    None
}

impl RuntimeBackend for PythonSession {
    fn execute_with_control(
        &mut self,
        source: &str,
        timeout: Option<Duration>,
        cancellation: Arc<AtomicBool>,
    ) -> Result<ExecutionResult, RuntimeError> {
        self.execute_inner(source, timeout, cancellation, None)
    }

    fn execute_with_events(
        &mut self,
        source: &str,
        timeout: Option<Duration>,
        cancellation: Arc<AtomicBool>,
        events: &mut dyn FnMut(StreamEvent),
    ) -> Result<ExecutionResult, RuntimeError> {
        self.execute_inner(source, timeout, cancellation, Some(events))
    }

    fn reset(&mut self) -> Result<(), RuntimeError> {
        self.kill_child();
        self.execution_count = 0;
        Ok(())
    }

    fn execution_count(&self) -> u64 {
        self.execution_count
    }

    fn cargo_lock(&self) -> Result<Option<String>, RuntimeError> {
        Ok(None)
    }

    fn inspect_variables(&mut self) -> Result<Vec<VariableInfo>, RuntimeError> {
        self.ensure_spawn()?;
        let id = self.next_request_id();
        let response = self.transact(
            json!({"type": "inspect", "id": id}),
            "inspection",
            CONTROL_TIMEOUT,
        )?;
        let variables = response
            .get("variables")
            .and_then(Value::as_array)
            .map(|entries| {
                entries
                    .iter()
                    .filter_map(|entry| {
                        Some(VariableInfo {
                            name: entry.get("name")?.as_str()?.to_owned(),
                            type_name: entry.get("type").and_then(Value::as_str).map(str::to_owned),
                            value: entry
                                .get("value")
                                .and_then(Value::as_str)
                                .map(str::to_owned),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
        Ok(variables)
    }

    fn diagnose(&mut self, source: &str) -> Result<Vec<CellDiagnostic>, RuntimeError> {
        self.ensure_spawn()?;
        let id = self.next_request_id();
        let response = self.transact(
            json!({"type": "check", "id": id, "source": source}),
            "check_result",
            CONTROL_TIMEOUT,
        )?;
        let Some(error) = response.get("error").filter(|value| !value.is_null()) else {
            return Ok(Vec::new());
        };
        let line = error.get("lineno").and_then(Value::as_u64).unwrap_or(1) as u32;
        let column = error.get("offset").and_then(Value::as_u64).unwrap_or(1) as u32;
        Ok(vec![CellDiagnostic {
            cell_index: None,
            line,
            column,
            end_line: line,
            end_column: column,
            severity: "error".to_owned(),
            code: error.get("name").and_then(Value::as_str).map(str::to_owned),
            message: error
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned(),
        }])
    }

    fn handle_comm(
        &mut self,
        comm_id: &str,
        comm_type: &str,
        data: &serde_json::Value,
    ) -> Result<Vec<mars_core::CommMessage>, RuntimeError> {
        if self.executing.load(Ordering::Relaxed) {
            return Err(RuntimeError::Backend(
                "cannot deliver comm while a cell is executing".to_owned(),
            ));
        }
        self.ensure_spawn()?;
        let id = self.next_request_id();
        let request_type = if comm_type == "comm_close" {
            "comm_close"
        } else {
            "comm_msg"
        };
        self.send(&json!({
            "type": request_type,
            "id": id,
            "comm_id": comm_id,
            "data": data,
        }))?;
        let Some(receiver) = self.lines.as_ref() else {
            return Err(RuntimeError::Backend(
                "python driver stdout unavailable".to_owned(),
            ));
        };
        let started = Instant::now();
        let mut comms = Vec::new();
        loop {
            if started.elapsed() >= CONTROL_TIMEOUT {
                return Err(RuntimeError::Backend("comm request timed out".to_owned()));
            }
            match receiver.recv_timeout(Duration::from_millis(20)) {
                Ok(line) => {
                    let Ok(message) = serde_json::from_str::<Value>(&line) else {
                        continue;
                    };
                    match message.get("type").and_then(Value::as_str) {
                        Some("comm") => {
                            if let Some(comm) = parse_comm_message(&message) {
                                comms.push(comm);
                            }
                        }
                        Some("comm_ack")
                            if message.get("id").and_then(Value::as_u64) == Some(id) =>
                        {
                            return Ok(comms);
                        }
                        _ => {}
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if let Some(child) = self.child.as_mut()
                        && !matches!(child.try_wait(), Ok(None))
                    {
                        return Err(self.driver_exit_error());
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    return Err(self.driver_exit_error());
                }
            }
        }
    }

    fn write_stdin(&mut self, text: &str) -> Result<(), RuntimeError> {
        self.send(&json!({"type": "input_response", "text": text}))
    }

    fn stdin_writer(&self) -> Option<StdinWriter> {
        let stdin = Arc::clone(&self.stdin);
        Some(StdinWriter(Arc::new(move |text: Option<&str>| {
            let payload = match serde_json::to_vec(&json!({
                "type": "input_response",
                "text": text,
            })) {
                Ok(payload) => payload,
                Err(source) => {
                    return Err(RuntimeError::Backend(format!(
                        "failed to encode input response: {source}"
                    )));
                }
            };
            let mut guard = match stdin.lock() {
                Ok(guard) => guard,
                Err(_) => {
                    return Err(RuntimeError::Backend(
                        "python driver stdin lock poisoned".to_owned(),
                    ));
                }
            };
            let Some(child_stdin) = guard.as_mut() else {
                return Err(RuntimeError::Backend(
                    "python driver stdin unavailable".to_owned(),
                ));
            };
            let write = child_stdin
                .write_all(&payload)
                .and_then(|()| child_stdin.write_all(b"\n"))
                .and_then(|()| child_stdin.flush());
            if let Err(source) = write {
                return Err(RuntimeError::Backend(format!(
                    "failed to write to python driver: {source}"
                )));
            }
            Ok(())
        })))
    }
}

impl Drop for PythonSession {
    fn drop(&mut self) {
        self.kill_child();
        let _ = fs::remove_dir_all(&self.workdir);
    }
}
