use crate::AppState;
use crate::kernel::{
    cancellation_for, kernel_for, kernel_key, kernels_lock, register_input_writer, stream_sink,
};
use mars_document::from_json;
use mars_kernel::Kernel;
use mars_protocol::{KernelRequest, KernelResponse, RequestEnvelope};
use std::sync::atomic::Ordering;
use std::time::Duration;

/// Decides whether a kernel result should notify and returns the body,
/// updating the per-`(session, language)` dedup set. Host-side `Err`
/// (runtime cannot start) and `KernelResponse::Error` (backend process
/// died) notify once per failure streak; ordinary cell errors arrive
/// inside `Execution.outputs` and never notify. A successful `Execution`
/// clears the streak so the next failure pings again.
pub fn kernel_failure_body(
    notified: &mut std::collections::HashSet<(String, String)>,
    session: &str,
    language: &str,
    result: &Result<KernelResponse, String>,
) -> Option<String> {
    let key = kernel_key(session, language);
    match result {
        Ok(KernelResponse::Execution { .. }) => {
            notified.remove(&key);
            None
        }
        Ok(KernelResponse::Error { message }) => notified
            .insert(key)
            .then(|| format!("{language} kernel error: {message}")),
        Err(message) => notified
            .insert(key)
            .then(|| format!("{language} kernel unavailable: {message}")),
        _ => None,
    }
}

/// Thin AppHandle wrapper around `kernel_failure_body`.
pub fn notify_kernel_result<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    state: &AppState,
    session: &str,
    language: &str,
    result: &Result<KernelResponse, String>,
) {
    let body = state
        .notified_kernel_errors
        .lock()
        .ok()
        .and_then(|mut notified| kernel_failure_body(&mut notified, session, language, result));
    if let Some(body) = body {
        crate::notify::notify_unfocused(app, "Mars Notebook", body);
    }
}

#[tauri::command]
pub fn execute_cell(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    session: String,
    source: String,
    language: String,
) -> Result<KernelResponse, String> {
    let mut sink = stream_sink(&app, &session, &language);
    let result = execute_cell_inner(&state, &session, &source, &language, None, &mut sink);
    notify_kernel_result(&app, &state, &session, &language, &result);
    result
}

#[tauri::command]
pub fn execute_cell_with_timeout(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    session: String,
    source: String,
    timeout_millis: u64,
    language: String,
) -> Result<KernelResponse, String> {
    if timeout_millis == 0 {
        return Err("timeout_millis must be greater than zero".to_owned());
    }
    let mut sink = stream_sink(&app, &session, &language);
    let result = execute_cell_inner(
        &state,
        &session,
        &source,
        &language,
        Some(Duration::from_millis(timeout_millis)),
        &mut sink,
    );
    notify_kernel_result(&app, &state, &session, &language, &result);
    result
}

/// Shared execution path with the stream sink injected, so tests can drive it
/// without a Tauri `AppHandle`.
pub fn execute_cell_inner(
    state: &AppState,
    session: &str,
    source: &str,
    language: &str,
    timeout: Option<Duration>,
    sink: &mut dyn FnMut(mars_runtime::StreamEvent),
) -> Result<KernelResponse, String> {
    let mut kernels = kernels_lock(state)?;
    let kernel = kernel_for(state, &mut kernels, session, language, state.backend)?;
    register_input_writer(state, session, language, kernel);
    let cancellation = cancellation_for(state, session);
    cancellation.store(false, Ordering::Relaxed);
    Ok(kernel.execute_with_events(source, timeout, cancellation, sink))
}

#[tauri::command]
pub fn cancel_execution(state: tauri::State<'_, AppState>, session: String) -> Result<(), String> {
    cancellation_for(&state, &session).store(true, Ordering::Relaxed);
    Ok(())
}

#[tauri::command]
pub fn reset_kernel(
    state: tauri::State<'_, AppState>,
    session: String,
    language: String,
) -> Result<KernelResponse, String> {
    let mut kernels = kernels_lock(&state)?;
    let kernel = kernel_for(&state, &mut kernels, &session, &language, state.backend)?;
    Ok(kernel
        .handle(RequestEnvelope::new(KernelRequest::Reset))
        .response)
}

#[tauri::command]
pub fn kernel_status(
    state: tauri::State<'_, AppState>,
    session: String,
    language: String,
) -> Result<KernelResponse, String> {
    let mut kernels = kernels_lock(&state)?;
    let kernel = kernel_for(&state, &mut kernels, &session, &language, state.backend)?;
    Ok(kernel
        .handle(RequestEnvelope::new(KernelRequest::Status))
        .response)
}

#[tauri::command]
pub fn configure_kernel(
    state: tauri::State<'_, AppState>,
    session: String,
    content: String,
    base_dir: Option<String>,
) -> Result<(), String> {
    let notebook = from_json(&content).map_err(|error| error.to_string())?;
    let python_options = mars_kernel::python_options_from_metadata(&notebook.metadata.extra);
    if let Ok(mut options) = state.python_options.lock() {
        options.insert(session.clone(), python_options.clone());
    }
    let base = base_dir
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let sql_sources = mars_kernel::sql_sources_from_metadata(&notebook.metadata.extra, &base);
    if let Ok(mut sources) = state.sql_sources.lock() {
        sources.insert(session.clone(), sql_sources);
    }
    let kernel = if notebook.metadata.language == "python" {
        Kernel::for_language_with_python_options(
            &notebook.metadata.language,
            state.backend,
            python_options,
        )
    } else {
        Kernel::for_language_with_configuration(
            &notebook.metadata.language,
            notebook.metadata.dependencies,
            notebook.metadata.cargo_lock,
            false,
            false,
            state.backend,
        )
    }
    .map_err(|error| error.to_string())?;
    let mut kernels = kernels_lock(&state)?;
    kernels.insert(kernel_key(&session, &notebook.metadata.language), kernel);
    register_input_writer(
        &state,
        &session,
        &notebook.metadata.language,
        kernels
            .get(&kernel_key(&session, &notebook.metadata.language))
            .ok_or_else(|| "kernel session unavailable".to_owned())?,
    );
    Ok(())
}
