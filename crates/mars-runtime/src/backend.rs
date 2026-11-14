use crate::error::RuntimeError;
use crate::output::{ExecutionResult, StreamEvent};
use std::sync::{Arc, atomic::AtomicBool};

/// `None` signals end-of-file on the input channel (e.g. piped stdin closed).
pub type StdinWriterFn = Arc<dyn Fn(Option<&str>) -> Result<(), RuntimeError> + Send + Sync>;

/// Cloneable handle that writes a line to the running cell's stdin without
/// borrowing the backend. Unlike [`RuntimeBackend::write_stdin`], this can be
/// invoked while `execute_*` holds `&mut self`, which is required for
/// interactive prompts (`input()`, `mars_prompt`).
#[derive(Clone)]
pub struct StdinWriter(pub StdinWriterFn);

impl StdinWriter {
    pub fn write(&self, text: &str) -> Result<(), RuntimeError> {
        (self.0)(Some(text))
    }

    /// Signals end-of-file on the input channel.
    pub fn write_eof(&self) -> Result<(), RuntimeError> {
        (self.0)(None)
    }
}

pub trait RuntimeBackend: Send {
    fn execute_with_control(
        &mut self,
        source: &str,
        timeout: Option<std::time::Duration>,
        cancellation: Arc<AtomicBool>,
    ) -> Result<ExecutionResult, crate::error::RuntimeError>;

    fn execute_with_events(
        &mut self,
        source: &str,
        timeout: Option<std::time::Duration>,
        cancellation: Arc<AtomicBool>,
        events: &mut dyn FnMut(StreamEvent),
    ) -> Result<ExecutionResult, crate::error::RuntimeError> {
        let _ = events;
        self.execute_with_control(source, timeout, cancellation)
    }

    fn reset(&mut self) -> Result<(), crate::error::RuntimeError>;

    fn execution_count(&self) -> u64;

    fn cargo_lock(&self) -> Result<Option<String>, crate::error::RuntimeError>;

    /// Live bindings from the accumulated cells, when the backend can inspect them.
    fn inspect_variables(
        &mut self,
    ) -> Result<Vec<crate::inspect::VariableInfo>, crate::error::RuntimeError> {
        Ok(Vec::new())
    }

    /// Pre-execution diagnostics for `source` in the context of committed cells.
    fn diagnose(
        &mut self,
        _source: &str,
    ) -> Result<Vec<crate::inspect::CellDiagnostic>, crate::error::RuntimeError> {
        Ok(Vec::new())
    }

    /// Writes a line to the running cell program's stdin (`mars_prompt` answers).
    /// Delivers a Jupyter-style comm message (`comm_msg`/`comm_close`) to
    /// the runtime (widget state updates). Returns any comm messages the
    /// runtime publishes while handling it. Backends without comm support
    /// return an empty list.
    fn handle_comm(
        &mut self,
        _comm_id: &str,
        _comm_type: &str,
        _data: &serde_json::Value,
    ) -> Result<Vec<mars_core::CommMessage>, crate::error::RuntimeError> {
        Ok(Vec::new())
    }

    fn write_stdin(&mut self, _text: &str) -> Result<(), crate::error::RuntimeError> {
        Err(crate::error::RuntimeError::Backend(
            "kernel input is not supported by this backend".to_owned(),
        ))
    }

    /// Returns a handle for writing stdin while a cell executes, when the
    /// backend can provide one without borrowing `self`.
    fn stdin_writer(&self) -> Option<StdinWriter> {
        None
    }
}
