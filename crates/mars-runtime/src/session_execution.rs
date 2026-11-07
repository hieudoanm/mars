use crate::ExecutionResult;
use crate::RustSession;
use crate::backend::RuntimeBackend;
use crate::compilation::CompileOptions;
use crate::error::RuntimeError;
use crate::execution::{
    configured_command, interruption_message, remaining_timeout, run_child_streaming,
};
use crate::expression::{render_program, render_program_with_last_output};
use crate::output::{
    ExpressionOutput, StreamEvent, StreamPhase, execution_outputs, interrupted_result,
};
use mars_core::Output;
use std::process::Stdio;
use std::sync::{
    Arc, Mutex,
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
        self.execute_inner(source, timeout, cancellation, None)
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
        if let Some(sink) = events.as_deref_mut() {
            sink(StreamEvent::Phase {
                phase: StreamPhase::Compiling,
            });
        }
        let mut program = render_program(&candidate_cells);
        let mut compile = crate::compilation::compile_program(
            &self.workdir,
            &source_path,
            &binary_path,
            program.clone(),
            &compile_options,
        )?;
        if !compile.status.is_some_and(|status| status.success())
            && crate::expression::split_final_expression(source).is_some()
        {
            program = render_program_with_last_output(&candidate_cells, ExpressionOutput::Display);
            let display_compile = crate::compilation::compile_program(
                &self.workdir,
                &source_path,
                &binary_path,
                program.clone(),
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
        let mut execution_child = configured_command(&execution_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(RuntimeError::Execution)?;
        self.stdin = execution_child
            .stdin
            .take()
            .map(|stdin| Arc::new(Mutex::new(stdin)));
        if let Some(sink) = events.as_deref_mut() {
            sink(StreamEvent::Phase {
                phase: StreamPhase::Running,
            });
        }
        let run = run_child_streaming(
            execution_child,
            remaining_timeout(started, timeout),
            &cancellation,
            events,
        )?;
        self.stdin = None;
        if run.interrupted {
            return Ok(interrupted_result(
                started.elapsed(),
                self.execution_count,
                interruption_message(&cancellation, "execution"),
            ));
        }
        if !run.status.is_some_and(|status| status.success()) {
            let stderr = crate::output::annotate_source_locations(
                &String::from_utf8_lossy(&run.stderr),
                &program,
            );
            return Ok(ExecutionResult {
                outputs: execution_outputs(&run.stdout, stderr.as_bytes(), "RuntimeError"),
                duration: started.elapsed(),
                execution_count: self.execution_count,
                interrupted: false,
            });
        }

        self.cells.push(source.to_owned());
        self.execution_count += 1;
        let stderr = crate::output::annotate_source_locations(
            &String::from_utf8_lossy(&run.stderr),
            &program,
        );
        Ok(ExecutionResult {
            outputs: execution_outputs(&run.stdout, stderr.as_bytes(), "RuntimeError"),
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

    fn execute_with_events(
        &mut self,
        source: &str,
        timeout: Option<std::time::Duration>,
        cancellation: Arc<AtomicBool>,
        events: &mut dyn FnMut(StreamEvent),
    ) -> Result<ExecutionResult, crate::error::RuntimeError> {
        self.execute_inner(source, timeout, cancellation, Some(events))
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

    fn inspect_variables(
        &mut self,
    ) -> Result<Vec<crate::inspect::VariableInfo>, crate::error::RuntimeError> {
        RustSession::inspect_variables(self)
    }

    fn diagnose(
        &mut self,
        source: &str,
    ) -> Result<Vec<crate::inspect::CellDiagnostic>, crate::error::RuntimeError> {
        RustSession::diagnose(self, source)
    }

    fn write_stdin(&mut self, text: &str) -> Result<(), crate::error::RuntimeError> {
        RustSession::write_stdin(self, text)
    }
}
