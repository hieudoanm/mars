use crate::compilation::{compile_command, render_program};
use crate::diagnostics::parse_compiler_diagnostics;
use crate::manifest::generate_conanfile;
use mars_core::Output;
use mars_runtime::{error::RuntimeError, output::ExecutionResult, RuntimeBackend};
use std::{
    collections::HashMap,
    fs,
    path::Path,
    process::{Command, Stdio},
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};
use tempfile::TempDir;

pub struct CppSession {
    workdir: TempDir,
    cells: Vec<String>,
    execution_count: u64,
    dependencies: HashMap<String, String>,
    conan_lock: Option<String>,
    offline: bool,
    locked: bool,
    compiler: String,
}

impl CppSession {
    pub fn new(
        dependencies: HashMap<String, String>,
        conan_lock: Option<String>,
        offline: bool,
        locked: bool,
    ) -> Result<Self, RuntimeError> {
        let workdir = TempDir::new()
            .map_err(|e| RuntimeError::Backend(format!("Failed to create tempdir: {e}")))?;

        let compiler = find_compiler()?;

        Ok(Self {
            workdir,
            cells: Vec::new(),
            execution_count: 0,
            dependencies,
            conan_lock,
            offline,
            locked,
            compiler,
        })
    }

    fn workdir_path(&self) -> &Path {
        self.workdir.path()
    }
}

impl RuntimeBackend for CppSession {
    fn execute_with_control(
        &mut self,
        source: &str,
        timeout: Option<Duration>,
        cancellation: Arc<AtomicBool>,
    ) -> Result<ExecutionResult, RuntimeError> {
        let start = std::time::Instant::now();

        // Add the new cell to accumulated sources
        self.cells.push(source.to_string());

        // Generate the complete program
        let program = render_program(&self.cells);

        // Write program to source file
        let source_path = self.workdir_path().join("main.cpp");
        fs::write(&source_path, program)
            .map_err(|e| RuntimeError::Backend(format!("Failed to write source: {e}")))?;

        // Handle dependencies with Conan
        if !self.dependencies.is_empty() {
            let conanfile = generate_conanfile(&self.dependencies);
            let conanfile_path = self.workdir_path().join("conanfile.txt");
            fs::write(&conanfile_path, conanfile)
                .map_err(|e| RuntimeError::Backend(format!("Failed to write conanfile: {e}")))?;

            // Run conan install
            let mut conan_cmd = Command::new("conan");
            conan_cmd
                .arg("install")
                .arg(".")
                .arg("--build=missing")
                .current_dir(self.workdir_path());

            if self.offline {
                conan_cmd.arg("--offline");
            }

            if self.locked {
                if let Some(lock) = &self.conan_lock {
                    let lock_path = self.workdir_path().join("conan.lock");
                    fs::write(&lock_path, lock)
                        .map_err(|e| RuntimeError::Backend(format!("Failed to write lock: {e}")))?;
                    conan_cmd.arg("--lockfile=conan.lock");
                    conan_cmd.arg("--lockfile-out=conan.lock");
                } else {
                    return Err(RuntimeError::Backend(
                        "--locked requires metadata.conan_lock".to_owned(),
                    ));
                }
            }

            let conan_output = conan_cmd
                .output()
                .map_err(|e| RuntimeError::Backend(format!("Conan install failed: {e}")))?;

            if !conan_output.status.success() {
                let stderr = String::from_utf8_lossy(&conan_output.stderr);
                return Err(RuntimeError::Backend(format!(
                    "Conan install failed: {}",
                    stderr
                )));
            }

            // Save lockfile if this was the first successful install
            if self.conan_lock.is_none() {
                let lock_path = self.workdir_path().join("conan.lock");
                if lock_path.exists() {
                    let lock_content = fs::read_to_string(&lock_path)
                        .map_err(|e| RuntimeError::Backend(format!("Failed to read lock: {e}")))?;
                    self.conan_lock = Some(lock_content);
                }
            }
        }

        // Compile the program
        let mut compile_cmd = compile_command(
            &self.compiler,
            &source_path,
            self.workdir_path(),
            !self.dependencies.is_empty(),
        );

        let compile_output = compile_cmd
            .output()
            .map_err(|e| RuntimeError::Backend(format!("Compilation failed: {e}")))?;

        if !compile_output.status.success() {
            let stderr = String::from_utf8_lossy(&compile_output.stderr);
            let diagnostics = parse_compiler_diagnostics(&stderr);
            let error_msg = if diagnostics.is_empty() {
                stderr.to_string()
            } else {
                diagnostics.join("\n")
            };
            return Err(RuntimeError::Backend(format!(
                "Compilation error:\n{}",
                error_msg
            )));
        }

        // Execute the compiled binary in its own process group so timeout
        // and cancellation can kill the whole tree.
        let binary_path = self.workdir_path().join("main");
        let mut exec_cmd = mars_runtime::execution::configured_command(&binary_path);
        exec_cmd.stdin(Stdio::piped());
        exec_cmd.stdout(Stdio::piped());
        exec_cmd.stderr(Stdio::piped());

        let mut child = exec_cmd
            .spawn()
            .map_err(|e| RuntimeError::Backend(format!("Execution failed: {e}")))?;

        // Handle timeout and cancellation
        let start_time = std::time::Instant::now();
        loop {
            if let Some(to) = timeout {
                if start_time.elapsed() >= to {
                    terminate_process_tree(child.id())?;
                    return Err(RuntimeError::Backend("Execution timeout".to_owned()));
                }
            }

            if cancellation.load(std::sync::atomic::Ordering::Relaxed) {
                terminate_process_tree(child.id())?;
                return Err(RuntimeError::Backend("Execution interrupted".to_owned()));
            }

            match child.try_wait() {
                Ok(Some(status)) => {
                    if status.success() {
                        break;
                    } else {
                        return Err(RuntimeError::Backend(format!(
                            "Execution failed with status: {}",
                            status
                        )));
                    }
                }
                Ok(None) => {
                    std::thread::sleep(Duration::from_millis(50));
                }
                Err(e) => {
                    return Err(RuntimeError::Backend(format!("Failed to wait: {e}")));
                }
            }
        }

        let output = child
            .wait_with_output()
            .map_err(|e| RuntimeError::Backend(format!("Failed to get output: {e}")))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let mut outputs = Vec::new();

        if !stdout.is_empty() {
            outputs.push(Output::Text { text: stdout });
        }

        if !stderr.is_empty() {
            outputs.push(Output::Text {
                text: format!("[stderr] {}", stderr),
            });
        }

        self.execution_count += 1;

        Ok(ExecutionResult {
            execution_count: self.execution_count,
            duration: start.elapsed(),
            outputs,
            interrupted: false,
        })
    }

    fn reset(&mut self) -> Result<(), RuntimeError> {
        self.cells.clear();
        self.execution_count = 0;
        Ok(())
    }

    fn execution_count(&self) -> u64 {
        self.execution_count
    }

    fn cargo_lock(&self) -> Result<Option<String>, RuntimeError> {
        // C++ uses conan.lock instead of Cargo.lock, but we return it via the same interface
        Ok(self.conan_lock.clone())
    }
}

fn find_compiler() -> Result<String, RuntimeError> {
    if let Ok(compiler) = std::env::var("MARS_CPP_COMPILER") {
        if which::which(&compiler).is_ok() {
            return Ok(compiler);
        }
    }

    // Prefer clang++ for better diagnostics
    if which::which("clang++").is_ok() {
        return Ok("clang++".to_string());
    }

    // Fall back to g++
    if which::which("g++").is_ok() {
        return Ok("g++".to_string());
    }

    Err(RuntimeError::Backend(
        "No C++ compiler found. Please install clang++ or g++".to_owned(),
    ))
}

#[cfg(unix)]
fn terminate_process_tree(pid: u32) -> Result<(), RuntimeError> {
    // The child runs in its own process group (pgid == pid), so a negative
    // pid targets the whole group.
    Command::new("kill")
        .arg("-KILL")
        .arg(format!("-{pid}"))
        .status()
        .map_err(|e| RuntimeError::Backend(format!("Failed to kill process tree: {e}")))?;

    Ok(())
}

#[cfg(windows)]
fn terminate_process_tree(pid: u32) -> Result<(), RuntimeError> {
    Command::new("taskkill")
        .args(["/F", "/T", "/PID", &pid.to_string()])
        .status()
        .map_err(|e| RuntimeError::Backend(format!("Failed to kill process tree: {e}")))?;
    Ok(())
}
