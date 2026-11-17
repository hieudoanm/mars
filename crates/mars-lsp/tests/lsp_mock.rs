//! Integration test for `LspClient`/`LspSession` against a mock LSP server.
//! `MARS_RUST_ANALYZER` is pointed at a small Python script that speaks
//! Content-Length framed JSON-RPC, so no real rust-analyzer is required.

use mars_lsp::session::{CellSource, LspSession};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Tests in this binary mutate `MARS_RUST_ANALYZER`; serialize them so the
/// env var cannot be overwritten mid-test by a parallel test.
static ENV_LOCK: Mutex<()> = Mutex::new(());

const MOCK_SERVER: &str = r#"#!/usr/bin/env python3
import sys, json

doc_uri = None

def send(msg):
    body = json.dumps(msg).encode()
    sys.stdout.buffer.write(b"Content-Length: " + str(len(body)).encode() + b"\r\n\r\n" + body)
    sys.stdout.buffer.flush()

def read_msg():
    length = 0
    while True:
        line = sys.stdin.buffer.readline()
        if not line:
            return None
        line = line.strip()
        if not line:
            break
        if line.lower().startswith(b"content-length:"):
            length = int(line.split(b":", 1)[1].strip())
    if length <= 0:
        return None
    return json.loads(sys.stdin.buffer.read(length))

while True:
    msg = read_msg()
    if msg is None:
        break
    method = msg.get("method")
    if method == "exit":
        break
    mid = msg.get("id")
    if mid is None:
        # Capture the document uri from didOpen, then publish diagnostics.
        if method == "textDocument/didOpen":
            doc_uri = msg["params"]["textDocument"]["uri"]
            send({"jsonrpc": "2.0", "method": "textDocument/publishDiagnostics",
                  "params": {"uri": doc_uri, "diagnostics": [
                      {"range": {"start": {"line": 1, "character": 0}, "end": {"line": 1, "character": 5}},
                       "severity": 1, "message": "mock error"}]}})
        continue
    if method == "initialize":
        result = {"capabilities": {"hoverProvider": True}, "serverInfo": {"name": "mock-lsp", "version": "0"}}
    elif method == "shutdown":
        result = None
    elif method == "textDocument/hover":
        result = {"contents": {"kind": "markdown", "value": "fn helper(x: i32) -> i32"}}
    elif method == "textDocument/completion":
        result = {"isIncomplete": False, "items": [{"label": "helper", "kind": 3}]}
    elif method == "textDocument/signatureHelp":
        result = {"signatures": [{"label": "helper(x: i32) -> i32", "parameters": []}], "activeSignature": 0, "activeParameter": 0}
    elif method == "textDocument/definition":
        result = [{"uri": doc_uri, "range": {"start": {"line": 1, "character": 4}, "end": {"line": 1, "character": 9}}}]
    elif method == "textDocument/inlayHint":
        result = [{"position": {"line": 1, "character": 4}, "label": [{"value": "i32"}], "kind": 1, "paddingLeft": False, "paddingRight": True}]
    elif method == "textDocument/semanticTokens/full":
        result = {"data": [1, 0, 4, 9, 0, 0, 5, 3, 12, 0]}
    elif method == "textDocument/codeAction":
        result = [{"title": "Remove unused variable", "kind": "quickfix",
                   "edit": {"changes": {doc_uri: [{"range": {"start": {"line": 1, "character": 0}, "end": {"line": 1, "character": 10}}, "newText": ""}]}}}]
    else:
        result = None
    send({"jsonrpc": "2.0", "id": mid, "result": result})
"#;

fn python3_available() -> bool {
    std::process::Command::new("python3")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn install_mock_server(dir: &Path) -> Option<PathBuf> {
    if !python3_available() {
        return None;
    }
    let script = dir.join("mock_lsp.py");
    std::fs::write(&script, MOCK_SERVER).expect("write mock server");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).expect("chmod");
    }
    Some(script)
}

fn cells() -> Vec<CellSource> {
    vec![
        CellSource {
            id: "md1".to_owned(),
            cell_type: "markdown".to_owned(),
            source: "# Title".to_owned(),
        },
        CellSource {
            id: "c1".to_owned(),
            cell_type: "code".to_owned(),
            source: "let x = 1;".to_owned(),
        },
        CellSource {
            id: "c2".to_owned(),
            cell_type: "code".to_owned(),
            source: "let y = 2;".to_owned(),
        },
    ]
}

#[test]
fn session_round_trips_against_mock_server() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let base = std::env::temp_dir().join(format!("mars-lsp-mock-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).expect("temp dir");
    let Some(server) = install_mock_server(&base) else {
        eprintln!("python3 unavailable; skipping");
        return;
    };

    // SAFETY: test-only env mutation; no other test in this binary reads it.
    unsafe { std::env::set_var("MARS_RUST_ANALYZER", &server) };
    let mut session = LspSession::open(&base, "mock").expect("open session");
    session
        .sync(&cells(), &HashMap::new())
        .expect("sync document");

    // Diagnostics published by the mock on didOpen map back to cell c1.
    // The notification may arrive slightly after sync returns; poll briefly.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    let mut diagnostics = Vec::new();
    while std::time::Instant::now() < deadline {
        diagnostics = session.poll_diagnostics();
        if diagnostics.iter().any(|d| d.cell_id == "c1") {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    assert!(
        diagnostics
            .iter()
            .any(|d| d.cell_id == "c1" && d.message == "mock error"),
        "diagnostics: {diagnostics:?}"
    );

    // Requests map cell-local positions into the document.
    let hover = session.hover("c1", 0, 4).expect("hover");
    assert!(hover.is_some());
    assert!(
        session
            .completion("c1", 0, 4)
            .expect("completion")
            .is_some()
    );
    assert!(
        session
            .signature_help("c1", 0, 8)
            .expect("signature")
            .is_some()
    );

    // Definition resolves into cell c1.
    let definition = session.definition("c1", 0, 4).expect("definition");
    assert!(definition.is_some(), "expected definition target");

    // Inlay hints and semantic tokens map back to cells.
    let hints = session.inlay_hints().expect("hints");
    assert!(
        hints.iter().any(|h| h.cell_id == "c1" && h.label == "i32"),
        "{hints:?}"
    );
    let tokens = session.semantic_tokens().expect("tokens");
    assert!(tokens.iter().any(|t| t.cell_id == "c1"), "{tokens:?}");

    // Code actions map edits back to cell-local coordinates.
    let actions = session.code_actions("c1", 0, 2).expect("actions");
    assert!(!actions.is_empty(), "expected code actions");
    assert_eq!(actions[0].edits[0].cell_id, "c1");

    // A second sync sends didChange (not didOpen) and regenerates the
    // manifest with real dependency entries.
    let mut deps = std::collections::HashMap::new();
    deps.insert("serde".to_owned(), "1".to_owned());
    deps.insert("anyhow".to_owned(), "1".to_owned());
    session.sync(&cells(), &deps).expect("re-sync");
    let manifest = std::fs::read_to_string(base.join("mock").join("Cargo.toml")).expect("manifest");
    assert!(manifest.contains("serde"), "{manifest}");
    assert!(manifest.contains("anyhow"), "{manifest}");

    // Non-code cells have no document span: position mapping short-circuits.
    assert!(session.hover("md1", 0, 0).expect("md hover").is_none());
    assert!(
        session
            .completion("md1", 0, 0)
            .expect("md completion")
            .is_none()
    );
    // Out-of-range line inside a real cell also maps to None.
    assert!(session.hover("c1", 999, 0).expect("oob hover").is_none());

    // Unknown cells short-circuit to empty results without server calls.
    assert!(session.hover("nope", 0, 0).expect("hover").is_none());
    assert!(
        session
            .completion("nope", 0, 0)
            .expect("completion")
            .is_none()
    );
    assert!(session.definition("nope", 0, 0).expect("def").is_none());
    assert!(session.signature_help("nope", 0, 0).expect("sig").is_none());
    assert!(
        session
            .code_actions("nope", 0, 0)
            .expect("actions")
            .is_empty()
    );

    drop(session);
    unsafe { std::env::remove_var("MARS_RUST_ANALYZER") };
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn spawn_fails_for_missing_binary() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let base = std::env::temp_dir().join(format!("mars-lsp-missing-{}", std::process::id()));
    std::fs::create_dir_all(&base).expect("temp dir");
    // SAFETY: test-only env mutation.
    unsafe { std::env::set_var("MARS_RUST_ANALYZER", "/nonexistent/mars-lsp-xyz") };
    let result = LspSession::open(&base, "missing");
    unsafe { std::env::remove_var("MARS_RUST_ANALYZER") };
    assert!(result.is_err());
    let _ = std::fs::remove_dir_all(&base);
}
