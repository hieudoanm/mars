use crate::ExecutionResult;
use crate::RustSession;
use crate::backend::RuntimeBackend;
use crate::compilation::CompileOptions;
use crate::error::RuntimeError;
use crate::execution::{configured_command, interruption_message, remaining_timeout, run_child};
use crate::expression::{render_program, render_program_with_last_output};
use crate::output::{ExpressionOutput, execution_outputs, interrupted_result};
use mars_core::Output;
use std::process::Stdio;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::{Duration, Instant};

impl RustSession {
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
        let mut compile = crate::compilation::compile_program(
            &self.workdir,
            &source_path,
            &binary_path,
            render_program(&candidate_cells),
            &compile_options,
        )?;
        if !compile.status.is_some_and(|status| status.success())
            && crate::expression::split_final_expression(source).is_some()
        {
            let display_compile = crate::compilation::compile_program(
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
                    message: crate::compilation::format_compiler_diagnostics(&compile.stderr),
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
}

impl RuntimeBackend for RustSession {
    fn execute_with_control(
        &mut self,
        source: &str,
        timeout: Option<std::time::Duration>,
        cancellation: Arc<AtomicBool>,
    ) -> Result<ExecutionResult, crate::error::RuntimeError> {
        RustSession::execute_with_control(self, source, timeout, cancellation)
    }

    fn reset(&mut self) -> Result<(), crate::error::RuntimeError> {
        RustSession::reset(self);
        Ok(())
    }

    fn execution_count(&self) -> u64 {
        RustSession::execution_count(self)
    }

    fn cargo_lock(&self) -> Result<Option<String>, crate::error::RuntimeError> {
        RustSession::cargo_lock(self)
    }
}
