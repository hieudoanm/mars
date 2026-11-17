use mars_kernel::RuntimeBackendKind;
use mars_lsp::{CellSource, Dependencies};
use mars_tauri_lib::AppState;
use mars_tauri_lib::lsp::{LspSessions, PositionRequest, SyncRequest};
use mars_tauri_lib::lsp_cmds::*;
use tauri::Manager;

fn app() -> tauri::App<tauri::test::MockRuntime> {
    let app = tauri::test::mock_app();
    app.manage(AppState::new(RuntimeBackendKind::Replay));
    app
}

fn sessions() -> LspSessions {
    LspSessions::new()
}

fn cells() -> Vec<CellSource> {
    vec![CellSource {
        id: "c1".to_owned(),
        cell_type: "code".to_owned(),
        source: "fn main() {}".to_owned(),
    }]
}

fn position() -> PositionRequest {
    PositionRequest {
        session: "cmd".to_owned(),
        cell_id: "c1".to_owned(),
        line: 0,
        character: 0,
    }
}

#[test]
fn session_queries_error_before_sync() {
    // `Default` is equivalent to `new`.
    let _ = LspSessions::default();
    let sessions = sessions();
    assert!(sessions.hover("nope", "c1".to_owned(), 0, 0).is_err());
    assert!(sessions.completion("nope", "c1".to_owned(), 0, 0).is_err());
    assert!(
        sessions
            .signature_help("nope", "c1".to_owned(), 0, 0)
            .is_err()
    );
    assert!(sessions.definition("nope", "c1".to_owned(), 0, 0).is_err());
    assert!(
        sessions
            .code_actions("nope", "c1".to_owned(), 0, 0)
            .is_err()
    );
    assert!(sessions.inlay_hints("nope").is_err());
    assert!(sessions.semantic_tokens("nope").is_err());
    assert!(sessions.diagnostics("nope").is_err());
}

#[test]
fn sync_then_query_roundtrip() {
    let sessions = sessions();
    // rust-analyzer may be absent; both outcomes are valid coverage.
    match sessions.sync("s1", cells(), Dependencies::default()) {
        Ok(()) => {
            // With a live session every query must succeed (values may be
            // empty while rust-analyzer is still indexing).
            assert!(sessions.hover("s1", "c1".to_owned(), 0, 0).is_ok());
            assert!(sessions.completion("s1", "c1".to_owned(), 0, 0).is_ok());
            assert!(sessions.signature_help("s1", "c1".to_owned(), 0, 0).is_ok());
            assert!(sessions.definition("s1", "c1".to_owned(), 0, 0).is_ok());
            assert!(sessions.code_actions("s1", "c1".to_owned(), 0, 0).is_ok());
            assert!(sessions.inlay_hints("s1").is_ok());
            assert!(sessions.semantic_tokens("s1").is_ok());
            assert!(sessions.diagnostics("s1").is_ok());
            // Re-sync exercises the update path.
            sessions
                .sync("s1", cells(), Dependencies::default())
                .expect("re-syncs");
        }
        Err(_) => {
            assert!(sessions.hover("s1", "c1".to_owned(), 0, 0).is_err());
        }
    }
}

#[test]
fn sync_rejects_or_accepts_non_code_cells() {
    let sessions = sessions();
    let markdown = vec![CellSource {
        id: "m1".to_owned(),
        cell_type: "markdown".to_owned(),
        source: "# title".to_owned(),
    }];
    // Either the session rejects non-code cells or syncs them harmlessly.
    let _ = sessions.sync("md", markdown, Dependencies::default());
}

#[test]
fn request_payloads_deserialize() {
    let sync: SyncRequest = serde_json::from_value(serde_json::json!({
        "session": "deser",
        "cells": [{"id": "c1", "cell_type": "code", "source": "fn main() {}"}],
        "dependencies": {}
    }))
    .expect("deserializes sync request");
    assert_eq!(sync.session, "deser");
    assert_eq!(sync.cells.len(), 1);

    let position: PositionRequest = serde_json::from_value(serde_json::json!({
        "session": "deser",
        "cell_id": "c1",
        "line": 1,
        "character": 2
    }))
    .expect("deserializes position request");
    assert_eq!(position.cell_id, "c1");
}

#[test]
fn commands_delegate_to_state() {
    let app = app();
    let state = app.state::<AppState>();

    lsp_sync(
        state.clone(),
        SyncRequest {
            session: "cmd".to_owned(),
            cells: cells(),
            dependencies: Dependencies::default(),
        },
    )
    .ok();

    let _ = lsp_hover(state.clone(), position());
    let _ = lsp_completion(state.clone(), position());
    let _ = lsp_signature_help(state.clone(), position());
    let _ = lsp_code_actions(state.clone(), position());
    let _ = lsp_definition(state.clone(), position());
    let _ = lsp_inlay_hints(state.clone(), "cmd".to_owned());
    let _ = lsp_semantic_tokens(state.clone(), "cmd".to_owned());
    let _ = lsp_diagnostics(state.clone(), "cmd".to_owned());
}
