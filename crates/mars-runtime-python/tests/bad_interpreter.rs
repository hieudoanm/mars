//! A `MARS_PYTHON` override pointing at a missing binary must surface a
//! spawn error on first execution rather than panicking.

use mars_runtime::RuntimeBackend;
use mars_runtime_python::PythonSession;
use std::sync::{Arc, atomic::AtomicBool};

#[test]
fn missing_override_binary_is_a_backend_error() {
    unsafe { std::env::set_var("MARS_PYTHON", "/nonexistent/mars-no-python") };
    let mut session = PythonSession::new().expect("session spawns lazily");
    let result = session.execute_with_control("1 + 1", None, Arc::new(AtomicBool::new(false)));
    unsafe { std::env::remove_var("MARS_PYTHON") };
    assert!(result.is_err());
}
