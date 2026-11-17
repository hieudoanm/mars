use crate::{AppState, lsp};

#[tauri::command]
pub(crate) fn lsp_sync(
    state: tauri::State<AppState>,
    request: lsp::SyncRequest,
) -> Result<(), String> {
    state
        .lsp
        .sync(&request.session, request.cells, request.dependencies)
}

#[tauri::command]
pub(crate) fn lsp_hover(
    state: tauri::State<AppState>,
    request: lsp::PositionRequest,
) -> Result<Option<serde_json::Value>, String> {
    state.lsp.hover(
        &request.session,
        request.cell_id,
        request.line,
        request.character,
    )
}

#[tauri::command]
pub(crate) fn lsp_completion(
    state: tauri::State<AppState>,
    request: lsp::PositionRequest,
) -> Result<Option<serde_json::Value>, String> {
    state.lsp.completion(
        &request.session,
        request.cell_id,
        request.line,
        request.character,
    )
}

#[tauri::command]
pub(crate) fn lsp_signature_help(
    state: tauri::State<AppState>,
    request: lsp::PositionRequest,
) -> Result<Option<serde_json::Value>, String> {
    state.lsp.signature_help(
        &request.session,
        request.cell_id,
        request.line,
        request.character,
    )
}

#[tauri::command]
pub(crate) fn lsp_code_actions(
    state: tauri::State<AppState>,
    request: lsp::PositionRequest,
) -> Result<Vec<mars_lsp::LspCodeAction>, String> {
    state.lsp.code_actions(
        &request.session,
        request.cell_id,
        request.line,
        request.character,
    )
}

#[tauri::command]
pub(crate) fn lsp_definition(
    state: tauri::State<AppState>,
    request: lsp::PositionRequest,
) -> Result<Option<mars_lsp::DefinitionTarget>, String> {
    state.lsp.definition(
        &request.session,
        request.cell_id,
        request.line,
        request.character,
    )
}

#[tauri::command]
pub(crate) fn lsp_inlay_hints(
    state: tauri::State<AppState>,
    session: String,
) -> Result<Vec<mars_lsp::CellInlayHint>, String> {
    state.lsp.inlay_hints(&session)
}

#[tauri::command]
pub(crate) fn lsp_semantic_tokens(
    state: tauri::State<AppState>,
    session: String,
) -> Result<Vec<mars_lsp::CellSemanticToken>, String> {
    state.lsp.semantic_tokens(&session)
}

#[tauri::command]
pub(crate) fn lsp_diagnostics(
    state: tauri::State<AppState>,
    session: String,
) -> Result<Vec<mars_lsp::CellDiagnostic>, String> {
    state.lsp.diagnostics(&session)
}
