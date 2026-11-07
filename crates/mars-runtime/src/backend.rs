use crate::output::{ExecutionResult, StreamEvent};
use std::sync::{Arc, atomic::AtomicBool};

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
    fn write_stdin(&mut self, _text: &str) -> Result<(), crate::error::RuntimeError> {
        Err(crate::error::RuntimeError::Backend(
            "kernel input is not supported by this backend".to_owned(),
        ))
    }
}
