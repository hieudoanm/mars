//! With no `MARS_JULIA` override and no `julia` on PATH, session creation
//! must fail with a clear backend error.

use mars_runtime_julia::JuliaSession;

#[test]
fn missing_interpreter_is_a_backend_error() {
    // This test binary never sets MARS_JULIA; if a real julia happens to be
    // installed the probe succeeds and there is nothing to assert.
    match JuliaSession::new() {
        Ok(_) => eprintln!("julia found on PATH; skipping assertion"),
        Err(error) => {
            let text = error.to_string();
            assert!(
                text.contains("Julia") || text.contains("julia"),
                "unexpected error: {text}"
            );
        }
    }
}
