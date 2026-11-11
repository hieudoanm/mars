//! With no `MARS_RSCRIPT` override and no Rscript on PATH, session creation
//! must fail with a clear backend error.

use mars_runtime_r::RSession;

#[test]
fn missing_interpreter_is_a_backend_error() {
    match RSession::new() {
        Ok(_) => eprintln!("Rscript found on PATH; skipping assertion"),
        Err(error) => {
            let text = error.to_string();
            assert!(
                text.contains("Rscript") || text.contains("R "),
                "unexpected error: {text}"
            );
        }
    }
}
