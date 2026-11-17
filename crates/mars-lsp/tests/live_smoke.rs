//! Opt-in smoke test against a real `rust-analyzer` binary.
//! Skipped automatically when rust-analyzer is not on PATH.

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use mars_lsp::session::{CellSource, LspSession};

fn rust_analyzer_available() -> bool {
    Command::new("rust-analyzer")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn temp_base() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("mars-lsp-smoke-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

#[test]
fn live_rust_analyzer_smoke() {
    if !rust_analyzer_available() {
        eprintln!("rust-analyzer not on PATH; skipping live smoke test");
        return;
    }
    let base = temp_base();
    let mut session = LspSession::open(&base, "smoke").expect("open lsp session");

    let cells = vec![
        CellSource {
            id: "c1".into(),
            cell_type: "code".into(),
            source: "fn helper(x: i32) -> i32 {\n    x * 2\n}\n\nlet value = helper(21);\nvalue"
                .into(),
        },
        CellSource {
            id: "c2".into(),
            cell_type: "code".into(),
            source: "let unused_smoke_variable = 1;".into(),
        },
    ];
    session
        .sync(&cells, &HashMap::new())
        .expect("sync document");

    // rust-analyzer needs a moment to load the workspace and publish
    // diagnostics for the intentionally broken second cell.
    let deadline = Instant::now() + Duration::from_secs(60);
    let mut diagnostics = Vec::new();
    while Instant::now() < deadline {
        diagnostics = session.poll_diagnostics();
        if diagnostics.iter().any(|d| d.cell_id == "c2") {
            break;
        }
        thread::sleep(Duration::from_millis(250));
    }
    assert!(
        diagnostics.iter().any(|d| d.cell_id == "c2"),
        "expected diagnostics for broken cell, got {diagnostics:?}"
    );

    // rust-analyzer answers lazily while it indexes; retry each request
    // until it produces a result or the deadline passes.
    let retry = |request: &mut dyn FnMut() -> bool| {
        let deadline = Instant::now() + Duration::from_secs(60);
        while Instant::now() < deadline {
            if request() {
                return true;
            }
            thread::sleep(Duration::from_millis(300));
        }
        false
    };

    // Hover over `helper` in cell c1 (line 4, col 13 in 0-based cell coords).
    let mut hover_text = String::new();
    let hover_ok = retry(&mut || match session.hover("c1", 4, 13) {
        Ok(Some(value)) => {
            hover_text = serde_json::to_string(&value).unwrap_or_default();
            hover_text.contains("helper")
        }
        _ => false,
    });
    assert!(hover_ok, "hover should mention helper, got {hover_text}");

    // Completion inside cell c1 after `hel`.
    let completion_ok = retry(&mut || matches!(session.completion("c1", 4, 8), Ok(Some(_))));
    assert!(completion_ok, "expected completion items");

    // Signature help inside the `helper(...)` call in cell c1.
    let signature_ok =
        retry(&mut || matches!(session.signature_help("c1", 4, 18), Ok(Some(_)) | Ok(None)));
    assert!(signature_ok, "signature_help request should not error");

    // Definition of `helper` call should resolve back into cell c1.
    let definition_ok = retry(&mut || matches!(session.definition("c1", 4, 13), Ok(Some(_))));
    assert!(definition_ok, "expected definition target for helper");

    // Code actions on the unused binding should offer quick fixes/assists
    // once diagnostics have been published.
    let actions_ok = retry(
        &mut || matches!(session.code_actions("c2", 0, 10), Ok(actions) if !actions.is_empty()),
    );
    assert!(actions_ok, "expected code actions for unused variable cell");

    // Inlay hints should annotate the `let value` binding once rust-analyzer
    // has finished indexing the scratch project.
    let hints_ok = retry(
        &mut || matches!(session.inlay_hints(), Ok(hints) if hints.iter().any(|h| h.cell_id == "c1" && !h.label.is_empty())),
    );
    assert!(hints_ok, "expected inlay hints mapped back to cell c1");

    // Semantic tokens should classify at least one token back into a cell.
    let tokens_ok = retry(
        &mut || matches!(session.semantic_tokens(), Ok(tokens) if tokens.iter().any(|t| !t.cell_id.is_empty())),
    );
    assert!(tokens_ok, "expected semantic tokens mapped back to cells");

    let _ = std::fs::remove_dir_all(&base);
}
