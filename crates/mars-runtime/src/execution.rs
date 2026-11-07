use crate::error::RuntimeError;
use std::{
    process::{Child, Command},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant},
};

use crate::output::{BinaryOutput, ChunkStream, StreamEvent};
use std::io::Read;
use std::sync::mpsc;

pub fn run_child(
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

/// Scans decoded stdout for `__MARS_PROMPT__<question>\n` protocol lines
/// emitted by the injected `mars_prompt` helper, returning cleaned text and
/// prompt messages. Incomplete markers are carried across chunk boundaries.
#[derive(Default)]
pub struct PromptScan {
    carry: String,
}

impl PromptScan {
    const MARKER: &'static str = "__MARS_PROMPT__";

    pub fn feed(&mut self, text: &str) -> (String, Vec<String>) {
        let combined = format!("{}{}", self.carry, text);
        self.carry.clear();
        let mut prompts = Vec::new();
        let mut cleaned = String::with_capacity(combined.len());
        let mut rest = combined.as_str();
        while let Some(index) = rest.find(Self::MARKER) {
            cleaned.push_str(&rest[..index]);
            let after_marker = &rest[index + Self::MARKER.len()..];
            match after_marker.find('\n') {
                Some(newline) => {
                    prompts.push(after_marker[..newline].trim_end().to_owned());
                    rest = &after_marker[newline + 1..];
                }
                None => {
                    // Marker without a terminating newline yet: hold from the
                    // marker start so the next chunk can complete it.
                    self.carry = rest[index..].to_owned();
                    rest = "";
                    break;
                }
            }
        }
        if rest.is_empty() && self.carry.is_empty() {
            // nothing held back
        } else if self.carry.is_empty() {
            cleaned.push_str(rest);
        }
        // Keep a tail that could be the start of a split marker.
        if self.carry.is_empty() {
            let keep = Self::MARKER.len() - 1;
            let mut start = cleaned.len().saturating_sub(keep);
            while start > 0 && !cleaned.is_char_boundary(start) {
                start -= 1;
            }
            self.carry = cleaned[start..].to_owned();
            cleaned.truncate(start);
        }
        (cleaned, prompts)
    }

    pub fn finish(&mut self) -> (String, Vec<String>) {
        let tail = std::mem::take(&mut self.carry);
        let (mut cleaned, mut prompts) = self.feed(&tail);
        if !self.carry.is_empty() {
            let rest = std::mem::take(&mut self.carry);
            if let Some(message) = rest.strip_prefix(Self::MARKER) {
                prompts.push(message.trim_end().to_owned());
            } else {
                cleaned.push_str(&rest);
            }
        }
        (cleaned, prompts)
    }
}

pub fn run_child_streaming(
    mut child: Child,
    timeout: Option<Duration>,
    cancellation: &AtomicBool,
    mut sink: Option<&mut dyn FnMut(StreamEvent)>,
) -> Result<BinaryOutput, RuntimeError> {
    let started = Instant::now();
    let (sender, receiver) = mpsc::channel::<(bool, Vec<u8>)>();
    let mut readers = Vec::new();
    for (is_stderr, pipe) in [
        (
            false,
            child
                .stdout
                .take()
                .map(|pipe| Box::new(pipe) as Box<dyn Read + Send>),
        ),
        (
            true,
            child
                .stderr
                .take()
                .map(|pipe| Box::new(pipe) as Box<dyn Read + Send>),
        ),
    ] {
        if let Some(mut pipe) = pipe {
            let sender = sender.clone();
            readers.push(thread::spawn(move || {
                let mut buffer = [0_u8; 8192];
                loop {
                    match pipe.read(&mut buffer) {
                        Ok(0) | Err(_) => break,
                        Ok(read) => {
                            if sender.send((is_stderr, buffer[..read].to_vec())).is_err() {
                                break;
                            }
                        }
                    }
                }
            }));
        }
    }
    drop(sender);

    let mut stdout_pending = Vec::new();
    let mut stderr_pending = Vec::new();
    let mut stdout_bytes = Vec::new();
    let mut stderr_bytes = Vec::new();
    let mut prompt_scan = PromptScan::default();

    loop {
        if let Some(status) = child.try_wait().map_err(RuntimeError::Execution)? {
            while let Ok((is_stderr, bytes)) = receiver.recv() {
                handle_stream_chunk(
                    is_stderr,
                    bytes,
                    &mut sink,
                    &mut stdout_pending,
                    &mut stderr_pending,
                    &mut stdout_bytes,
                    &mut stderr_bytes,
                    &mut prompt_scan,
                );
            }
            for reader in readers {
                let _ = reader.join();
            }
            let (tail, prompts) = prompt_scan.finish();
            if let Some(events) = sink.as_deref_mut() {
                for message in prompts {
                    events(StreamEvent::Prompt { message });
                }
                if !tail.is_empty() {
                    events(StreamEvent::Chunk {
                        stream: ChunkStream::Stdout,
                        data: tail,
                    });
                }
            }
            return Ok(BinaryOutput {
                status: Some(status),
                stdout: stdout_bytes,
                stderr: stderr_bytes,
                interrupted: false,
            });
        }

        while let Ok((is_stderr, bytes)) = receiver.try_recv() {
            handle_stream_chunk(
                is_stderr,
                bytes,
                &mut sink,
                &mut stdout_pending,
                &mut stderr_pending,
                &mut stdout_bytes,
                &mut stderr_bytes,
                &mut prompt_scan,
            );
        }

        if cancellation.load(Ordering::Relaxed)
            || timeout.is_some_and(|limit| started.elapsed() >= limit)
        {
            terminate_process_tree(&mut child)?;
            let _ = child.wait().map_err(RuntimeError::Execution)?;
            while let Ok((is_stderr, bytes)) = receiver.try_recv() {
                if is_stderr {
                    stderr_bytes.extend_from_slice(&bytes);
                } else {
                    stdout_bytes.extend_from_slice(&bytes);
                }
            }
            for reader in readers {
                let _ = reader.join();
            }
            let _ = prompt_scan.finish();
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

#[allow(clippy::too_many_arguments)]
fn handle_stream_chunk(
    is_stderr: bool,
    bytes: Vec<u8>,
    sink: &mut Option<&mut dyn FnMut(StreamEvent)>,
    stdout_pending: &mut Vec<u8>,
    stderr_pending: &mut Vec<u8>,
    stdout_bytes: &mut Vec<u8>,
    stderr_bytes: &mut Vec<u8>,
    prompt_scan: &mut PromptScan,
) {
    if is_stderr {
        stderr_bytes.extend_from_slice(&bytes);
    } else {
        stdout_bytes.extend_from_slice(&bytes);
    }
    let pending = if is_stderr {
        stderr_pending
    } else {
        stdout_pending
    };
    let text = decode_stream_chunk(pending, &bytes);
    if text.is_empty() {
        return;
    }
    let (text, prompts) = if is_stderr {
        (text, Vec::new())
    } else {
        prompt_scan.feed(&text)
    };
    if let Some(events) = sink.as_deref_mut() {
        for message in prompts {
            events(StreamEvent::Prompt { message });
        }
        if !text.is_empty() {
            events(StreamEvent::Chunk {
                stream: if is_stderr {
                    ChunkStream::Stderr
                } else {
                    ChunkStream::Stdout
                },
                data: text,
            });
        }
    }
}

fn decode_stream_chunk(pending: &mut Vec<u8>, bytes: &[u8]) -> String {
    pending.extend_from_slice(bytes);
    match std::str::from_utf8(pending) {
        Ok(text) => {
            let decoded = text.to_owned();
            pending.clear();
            decoded
        }
        Err(error) => {
            let valid = error.valid_up_to();
            let mut decoded = String::from_utf8_lossy(&pending[..valid]).into_owned();
            match error.error_len() {
                Some(invalid) => {
                    decoded.push('\u{FFFD}');
                    pending.drain(..valid + invalid);
                }
                None => {
                    pending.drain(..valid);
                }
            }
            decoded
        }
    }
}

pub fn terminate_process_tree(child: &mut Child) -> Result<(), RuntimeError> {
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

pub fn configured_command(program: impl AsRef<std::ffi::OsStr>) -> Command {
    let mut command = Command::new(program);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    command
}

pub fn interruption_message(cancellation: &AtomicBool, phase: &str) -> &'static str {
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

pub fn remaining_timeout(started: Instant, timeout: Option<Duration>) -> Option<Duration> {
    timeout.map(|limit| limit.saturating_sub(started.elapsed()))
}

#[cfg(test)]
mod prompt_scan_tests {
    use super::PromptScan;

    #[test]
    fn extracts_prompt_and_cleans_output() {
        let mut scan = PromptScan::default();
        let (_, prompts) = scan.feed("Hello\n__MARS_PROMPT__Name?\nWorld\n");
        assert_eq!(prompts, vec!["Name?"]);
        let (tail, _) = scan.finish();
        assert!(tail.contains("Hello"));
        assert!(tail.contains("World"));
        assert!(!tail.contains("__MARS_PROMPT__"));
    }

    #[test]
    fn carries_marker_across_chunks() {
        let mut scan = PromptScan::default();
        let (clean1, prompts1) = scan.feed("start __MARS_PROM");
        assert!(prompts1.is_empty());
        assert!(clean1.starts_with("sta"));
        let (_, prompts2) = scan.feed("PT__Pick one\nrest");
        assert_eq!(prompts2, vec!["Pick one"]);
        let (tail, _) = scan.finish();
        assert!(tail.contains("rest"));
        assert!(!tail.contains("__MARS_PROMPT__"));
    }
}
