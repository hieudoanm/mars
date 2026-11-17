//! LSP session failure paths. Runs in its own test process so the PATH
//! override cannot affect the other LSP integration tests.

use mars_lsp::{CellSource, Dependencies};
use mars_tauri_lib::lsp::LspSessions;

#[test]
fn sync_failure_maps_to_error_string() {
    // With no rust-analyzer on PATH the session cannot be opened; the error
    // must surface as a String, not a panic.
    let saved_path = std::env::var_os("PATH");
    unsafe { std::env::set_var("PATH", "/nonexistent-mars-lsp-path") };
    let sessions = LspSessions::new();
    let result = sessions.sync(
        "fail",
        vec![CellSource {
            id: "c1".to_owned(),
            cell_type: "code".to_owned(),
            source: "fn main() {}".to_owned(),
        }],
        Dependencies::default(),
    );
    unsafe {
        match saved_path {
            Some(path) => std::env::set_var("PATH", path),
            None => std::env::remove_var("PATH"),
        }
    }
    assert!(result.is_err());
}
