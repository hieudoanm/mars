use crate::error::RuntimeError;
use std::{
    process::{Child, Command},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant},
};

use crate::output::BinaryOutput;

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
