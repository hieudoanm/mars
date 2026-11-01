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
}
