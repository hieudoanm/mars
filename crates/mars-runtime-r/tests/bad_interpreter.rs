//! A `MARS_RSCRIPT` override pointing at a missing binary must surface a
//! spawn error on first execution rather than panicking.

use mars_runtime::RuntimeBackend;
use mars_runtime_r::RSession;
use std::sync::{Arc, atomic::AtomicBool};

#[test]
fn missing_override_binary_is_a_backend_error() {
    unsafe { std::env::set_var("MARS_RSCRIPT", "/nonexistent/mars-no-rscript") };
    let mut session = RSession::new().expect("session spawns lazily");
    let result = session.execute_with_control("1 + 1", None, Arc::new(AtomicBool::new(false)));
    unsafe { std::env::remove_var("MARS_RSCRIPT") };
    assert!(result.is_err());
}
