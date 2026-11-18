use mars_core::Output;
use mars_runtime::execution::{configured_command, interruption_message, terminate_process_tree};
use mars_runtime::{
    CellDiagnostic, ChunkStream, ExecutionResult, RuntimeBackend, RuntimeError, StreamEvent,
    StreamPhase, VariableInfo,
};
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

const DRIVER_SOURCE: &str = include_str!("driver.R");
const STDERR_TAIL_LIMIT: usize = 64 * 1024;
const CONTROL_TIMEOUT: Duration = Duration::from_secs(30);

/// Persistent R execution session backed by a long-lived `Rscript` driver
/// subprocess speaking newline-delimited JSON. Bindings persist in the
/// driver's `.GlobalEnv` across cells.
///
/// Known limitations:
/// - `readline()`/`scan()` consume kernel protocol lines; interactive input
///   is not supported.
/// - `cat(..., file = stderr())` bypasses stream capture.
/// - A timed-out or cancelled cell kills the driver; the next cell starts a
///   fresh R session and loses all state.
/// - Requires the `jsonlite` R package and a working `png` or `pdf` graphics
///   device for plot capture.
pub struct RSession {
    rscript: OsString,
    workdir: PathBuf,
    driver_path: PathBuf,
    child: Option<Child>,
    stdin: Option<ChildStdin>,
    lines: Option<Receiver<String>>,
    stderr_tail: Arc<Mutex<String>>,
    next_id: u64,
    execution_count: u64,
}

impl RSession {
    pub fn new() -> Result<Self, RuntimeError> {
        let rscript = find_rscript().ok_or_else(|| {
            RuntimeError::Backend(
                "no Rscript interpreter found; install R or set MARS_RSCRIPT".to_owned(),
            )
        })?;
        let workdir = std::env::temp_dir().join(format!("mars-r-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&workdir).map_err(|source| RuntimeError::Io {
            path: workdir.clone(),
            source,
        })?;
        let driver_path = workdir.join("driver.R");
        if let Err(source) = fs::write(&driver_path, DRIVER_SOURCE) {
            return Err(RuntimeError::Io {
                path: driver_path.clone(),
                source,
            });
        }
        Ok(Self {
            rscript,
            workdir,
            driver_path,
            child: None,
            stdin: None,
            lines: None,
            stderr_tail: Arc::new(Mutex::new(String::new())),
            next_id: 1,
            execution_count: 0,
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

        let mut command = configured_command(&self.rscript);
        command
            .arg("--vanilla")
            .arg(&self.driver_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = command.spawn().map_err(|source| {
            RuntimeError::Backend(format!("failed to start R driver: {source}"))
        })?;
        let Some(stdin) = child.stdin.take() else {
            return Err(RuntimeError::Backend(
                "R driver stdin unavailable".to_owned(),
            ));
        };
        let Some(stdout) = child.stdout.take() else {
            return Err(RuntimeError::Backend(
                "R driver stdout unavailable".to_owned(),
            ));
        };
        let stderr = child.stderr.take();

        if let Some(stderr) = stderr {
            let tail = Arc::clone(&self.stderr_tail);
            thread::spawn(move || drain_stderr(stderr, tail));
        }
        let (sender, receiver) = mpsc::channel::<String>();
        thread::spawn(move || read_stdout_lines(stdout, sender));

        self.child = Some(child);
        self.stdin = Some(stdin);
        self.lines = Some(receiver);
        Ok(())
    }

    fn send(&mut self, message: &Value) -> Result<(), RuntimeError> {
        self.ensure_spawn()?;
        let Some(stdin) = self.stdin.as_mut() else {
            return Err(RuntimeError::Backend(
                "R driver stdin unavailable".to_owned(),
            ));
        };
        let payload = match serde_json::to_vec(message) {
            Ok(payload) => payload,
            Err(source) => {
                return Err(RuntimeError::Backend(format!(
                    "failed to encode request: {source}"
                )));
            }
        };
        let write = stdin
            .write_all(&payload)
            .and_then(|()| stdin.write_all(b"\n"))
            .and_then(|()| stdin.flush());
        if let Err(source) = write {
            return Err(RuntimeError::Backend(format!(
                "failed to write to R driver: {source}"
            )));
        }
        Ok(())
    }

    fn kill_child(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = terminate_process_tree(&mut child);
            let _ = child.wait();
        }
        self.stdin = None;
        self.lines = None;
    }

    fn driver_exit_error(&self) -> RuntimeError {
        let tail = stderr_tail_lock(&self.stderr_tail).clone();
        let detail = if tail.trim().is_empty() {
            "process exited".to_owned()
        } else {
            format!("process exited: {}", tail.trim())
        };
        RuntimeError::Backend(format!("R driver {detail}"))
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
                "R driver stdout unavailable".to_owned(),
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
                    "R driver did not respond to {expected_type} request"
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
        let mut error: Option<(String, String, Vec<String>)> = None;

        let Some(receiver) = self.lines.as_ref() else {
            return Err(RuntimeError::Backend(
                "R driver stdout unavailable".to_owned(),
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
        if !stderr_text.is_empty() {
            outputs.push(Output::Error {
                name: "stderr".to_owned(),
                message: stderr_text,
                traceback: Vec::new(),
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

fn stderr_tail_lock(tail: &Mutex<String>) -> std::sync::MutexGuard<'_, String> {
    match tail.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
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

fn display_output(parts: &[(String, String)]) -> Option<Output> {
    for (mime, data) in parts {
        match mime.as_str() {
            "text/html" => return Some(Output::Html { html: data.clone() }),
            "image/png" | "image/jpeg" | "image/svg+xml" | "application/pdf" => {
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

fn find_rscript() -> Option<OsString> {
    if let Some(custom) = std::env::var_os("MARS_RSCRIPT") {
        return Some(custom);
    }
    let probe = configured_command("Rscript")
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    if matches!(probe, Ok(status) if status.success()) {
        return Some(OsString::from("Rscript"));
    }
    None
}

impl RuntimeBackend for RSession {
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

    fn write_stdin(&mut self, _text: &str) -> Result<(), RuntimeError> {
        Err(RuntimeError::Backend(
            "interactive input is not supported by the R backend".to_owned(),
        ))
    }
}

impl Drop for RSession {
    fn drop(&mut self) {
        self.kill_child();
        let _ = fs::remove_dir_all(&self.workdir);
    }
}
