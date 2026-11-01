use mars_core::Output;
use mars_runtime::backend::StdinWriter;
use mars_runtime::{ExecutionResult, RuntimeBackend, RuntimeError};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

struct StubBackend {
    calls: u32,
}

impl RuntimeBackend for StubBackend {
    fn execute_with_control(
        &mut self,
        _source: &str,
        _timeout: Option<Duration>,
        _cancellation: Arc<AtomicBool>,
    ) -> Result<ExecutionResult, RuntimeError> {
        self.calls += 1;
        Ok(ExecutionResult {
            outputs: vec![Output::Text {
                text: "stub".to_owned(),
            }],
            duration: Duration::ZERO,
            execution_count: self.calls as u64,
            interrupted: false,
        })
    }

    fn reset(&mut self) -> Result<(), RuntimeError> {
        self.calls = 0;
        Ok(())
    }

    fn execution_count(&self) -> u64 {
        self.calls as u64
    }

    fn cargo_lock(&self) -> Result<Option<String>, RuntimeError> {
        Ok(None)
    }
}

#[test]
fn stdin_writer_invokes_the_underlying_closure() {
    let seen = Arc::new(std::sync::Mutex::new(Vec::<Option<String>>::new()));
    let sink = Arc::clone(&seen);
    let writer = StdinWriter(Arc::new(move |text: Option<&str>| {
        sink.lock().expect("lock").push(text.map(str::to_owned));
        Ok(())
    }));

    writer.write("hello").expect("write");
    writer.write_eof().expect("eof");
    assert_eq!(
        seen.lock().expect("lock").as_slice(),
        [Some("hello".to_owned()), None]
    );
}

#[test]
fn stdin_writer_propagates_errors() {
    let writer = StdinWriter(Arc::new(|_| Err(RuntimeError::Backend("nope".to_owned()))));
    assert!(writer.write("x").is_err());
    assert!(writer.write_eof().is_err());
}

#[test]
fn trait_defaults_cover_inspection_and_input() {
    let mut backend = StubBackend { calls: 0 };

    // Default execute_with_events delegates to execute_with_control.
    let mut events = Vec::new();
    let result = backend
        .execute_with_events(
            "src",
            None,
            Arc::new(AtomicBool::new(false)),
            &mut |event| events.push(event),
        )
        .expect("execute");
    assert_eq!(result.execution_count, 1);
    assert!(events.is_empty());

    assert!(backend.inspect_variables().expect("inspect").is_empty());
    assert!(backend.diagnose("src").expect("diagnose").is_empty());
    assert!(backend.write_stdin("line").is_err());
    assert!(backend.stdin_writer().is_none());
}
