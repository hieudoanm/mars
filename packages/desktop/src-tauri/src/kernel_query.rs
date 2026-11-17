use crate::AppState;
use crate::kernel::{kernel_for, kernel_key, kernels_lock};

#[tauri::command]
pub fn inspect_variables(
    state: tauri::State<'_, AppState>,
    session: String,
    language: String,
) -> Result<Vec<mars_runtime::VariableInfo>, String> {
    let mut kernels = kernels_lock(&state)?;
    let kernel = kernel_for(&state, &mut kernels, &session, &language, state.backend)?;
    kernel
        .inspect_variables()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn diagnose_cell(
    state: tauri::State<'_, AppState>,
    session: String,
    source: String,
    language: String,
) -> Result<Vec<mars_runtime::CellDiagnostic>, String> {
    let mut kernels = kernels_lock(&state)?;
    let kernel = kernel_for(&state, &mut kernels, &session, &language, state.backend)?;
    kernel.diagnose(&source).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn send_kernel_input(
    state: tauri::State<'_, AppState>,
    session: String,
    text: String,
    language: String,
) -> Result<(), String> {
    // Prefer the stdin writer handle: it does not need the kernels mutex,
    // which execute_cell holds for the whole cell duration.
    let writer = state
        .input_writers
        .lock()
        .map_err(|_| "kernel lock poisoned".to_owned())?
        .get(&kernel_key(&session, &language))
        .cloned();
    if let Some(writer) = writer {
        return writer.write(&text).map_err(|error| error.to_string());
    }
    let mut kernels = kernels_lock(&state)?;
    let kernel = kernels
        .get_mut(&kernel_key(&session, &language))
        .ok_or_else(|| "kernel session unavailable".to_owned())?;
    kernel.write_stdin(&text).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn send_kernel_comm(
    state: tauri::State<'_, AppState>,
    session: String,
    comm_id: String,
    comm_type: String,
    data: serde_json::Value,
    language: String,
) -> Result<Vec<mars_core::CommMessage>, String> {
    let mut kernels = kernels_lock(&state)?;
    let kernel = kernels
        .get_mut(&kernel_key(&session, &language))
        .ok_or_else(|| "kernel session unavailable".to_owned())?;
    kernel
        .handle_comm(&comm_id, &comm_type, &data)
        .map_err(|error| error.to_string())
}
