use crate::control::{interrupted_result, terminate_evaluator};
use crate::session::EvcxrSession;
use evcxr::Error as EvcxrError;
use mars_core::Output;
use mars_runtime::{ExecutionResult, RuntimeBackend, RuntimeError};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::{Duration, Instant};

impl RuntimeBackend for EvcxrSession {
    fn execute_with_control(
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
                "execution cancelled before evaluation",
            ));
        }

        let started = Instant::now();
        let context = Arc::clone(&self.context);
        let process_handle = context
            .lock()
            .map_err(|_| RuntimeError::Backend("Evcxr context lock is poisoned".to_owned()))?
            .process_handle();
        let (sender, receiver) = crossbeam_channel::bounded(1);
        let source = source.to_owned();
        std::thread::Builder::new()
            .name("mars-evcxr-controller".to_owned())
            .spawn(move || {
                let result: Result<Result<evcxr::EvalOutputs, EvcxrError>, String> =
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        let mut context = context
                            .lock()
                            .map_err(|_| "Evcxr context lock is poisoned".to_owned())?;
                        Ok(context.execute(&source))
                    }))
                    .unwrap_or_else(|_| Err("Evcxr evaluation thread panicked".to_owned()));
                let _ = sender.send(result);
            })
            .map_err(|error| {
                RuntimeError::Backend(format!("could not start Evcxr controller thread: {error}"))
            })?;

        let evaluation = loop {
            match receiver.recv_timeout(Duration::from_millis(5)) {
                Ok(Ok(result)) => break result,
                Ok(Err(message)) => break Err(EvcxrError::Message(message)),
                Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                    break Err(EvcxrError::Message(
                        "Evcxr evaluation thread stopped".to_owned(),
                    ));
                }
                Err(crossbeam_channel::RecvTimeoutError::Timeout) => {}
            }
            let message = if cancellation.load(Ordering::Relaxed) {
                Some("execution cancelled; Evcxr session was restarted")
            } else if timeout.is_some_and(|limit| started.elapsed() >= limit) {
                Some("execution timed out; Evcxr session was restarted")
            } else {
                None
            };
            if let Some(message) = message {
                terminate_evaluator(&process_handle);
                self.recreate()?;
                return Ok(interrupted_result(
                    started.elapsed(),
                    self.execution_count,
                    message,
                ));
            }
        };

        let failed = evaluation.is_err();
        let outputs = self.collect_outputs(evaluation);
        if failed {
            let lost_process = outputs.iter().any(
                |output| matches!(output, Output::Error { name, .. } if name == "RuntimeError"),
            );
            if lost_process {
                self.recreate()?;
                return Ok(ExecutionResult {
                    outputs,
                    duration: started.elapsed(),
                    execution_count: self.execution_count,
                    interrupted: false,
                });
            }
            return Ok(ExecutionResult {
                outputs,
                duration: started.elapsed(),
                execution_count: self.execution_count,
                interrupted: false,
            });
        }

        self.execution_count += 1;
        Ok(ExecutionResult {
            outputs,
            duration: started.elapsed(),
            execution_count: self.execution_count,
            interrupted: false,
        })
    }

    fn reset(&mut self) -> Result<(), RuntimeError> {
        self.recreate()
    }

    fn execution_count(&self) -> u64 {
        self.execution_count
    }

    fn cargo_lock(&self) -> Result<Option<String>, RuntimeError> {
        Ok(None)
    }
}
