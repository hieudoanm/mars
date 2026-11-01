use crate::output::ExecutionResult;
use std::sync::{Arc, atomic::AtomicBool};

pub trait RuntimeBackend: Send {
    fn execute_with_control(
        &mut self,
        source: &str,
        timeout: Option<std::time::Duration>,
        cancellation: Arc<AtomicBool>,
    ) -> Result<ExecutionResult, crate::error::RuntimeError>;

    fn reset(&mut self) -> Result<(), crate::error::RuntimeError>;

    fn execution_count(&self) -> u64;

    fn cargo_lock(&self) -> Result<Option<String>, crate::error::RuntimeError>;
}
