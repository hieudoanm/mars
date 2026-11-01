use mars_document::{from_json, load, save_atomic, to_json_pretty};
use mars_kernel::{Kernel, RuntimeBackendKind};
use mars_protocol::{KernelRequest, KernelResponse, RequestEnvelope};
use mars_renderer::render_markdown_notebook;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

struct AppState {
    kernel: Mutex<Kernel>,
    cancellation: Arc<AtomicBool>,
    backend: RuntimeBackendKind,
}

#[tauri::command]
fn execute_cell(
    state: tauri::State<'_, AppState>,
    source: String,
) -> Result<KernelResponse, String> {
    let mut kernel = state
        .kernel
        .lock()
        .map_err(|_| "kernel lock poisoned".to_owned())?;
    state.cancellation.store(false, Ordering::Relaxed);
    Ok(kernel
        .handle_with_cancellation(
            RequestEnvelope::new(KernelRequest::Execute {
                source,
                timeout_millis: None,
            }),
            Arc::clone(&state.cancellation),
        )
        .response)
}

#[tauri::command]
fn execute_cell_with_timeout(
    state: tauri::State<'_, AppState>,
    source: String,
    timeout_millis: u64,
) -> Result<KernelResponse, String> {
    if timeout_millis == 0 {
        return Err("timeout_millis must be greater than zero".to_owned());
    }
    let mut kernel = state
        .kernel
        .lock()
        .map_err(|_| "kernel lock poisoned".to_owned())?;
    state.cancellation.store(false, Ordering::Relaxed);
    Ok(kernel
        .handle_with_cancellation(
            RequestEnvelope::new(KernelRequest::Execute {
                source,
                timeout_millis: Some(timeout_millis),
            }),
            Arc::clone(&state.cancellation),
        )
        .response)
}

#[tauri::command]
fn cancel_execution(state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.cancellation.store(true, Ordering::Relaxed);
    Ok(())
}

#[tauri::command]
fn reset_kernel(state: tauri::State<'_, AppState>) -> Result<KernelResponse, String> {
    let mut kernel = state
        .kernel
        .lock()
        .map_err(|_| "kernel lock poisoned".to_owned())?;
    Ok(kernel
        .handle(RequestEnvelope::new(KernelRequest::Reset))
        .response)
}

#[tauri::command]
fn kernel_status(state: tauri::State<'_, AppState>) -> Result<KernelResponse, String> {
    let mut kernel = state
        .kernel
        .lock()
        .map_err(|_| "kernel lock poisoned".to_owned())?;
    Ok(kernel
        .handle(RequestEnvelope::new(KernelRequest::Status))
        .response)
}

#[tauri::command]
fn configure_kernel(state: tauri::State<'_, AppState>, content: String) -> Result<(), String> {
    let notebook = from_json(&content).map_err(|error| error.to_string())?;
    let kernel = Kernel::with_backend(notebook.metadata.dependencies, state.backend, false)
        .map_err(|error| error.to_string())?;
    let mut current = state
        .kernel
        .lock()
        .map_err(|_| "kernel lock poisoned".to_owned())?;
    *current = kernel;
    Ok(())
}

#[tauri::command]
fn open_notebook(path: String) -> Result<String, String> {
    load(&path)
        .and_then(|notebook| to_json_pretty(&notebook))
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn save_notebook(path: String, content: String) -> Result<(), String> {
    let notebook = from_json(&content).map_err(|error| error.to_string())?;
    save_atomic(&notebook, &path).map_err(|error| error.to_string())
}

#[tauri::command]
fn export_notebook_markdown(path: String, content: String) -> Result<(), String> {
    let notebook = from_json(&content).map_err(|error| error.to_string())?;
    let markdown = render_markdown_notebook(&notebook);
    std::fs::write(&path, format!("{markdown}\n")).map_err(|error| error.to_string())
}

pub fn run() -> tauri::Result<()> {
    let backend = RuntimeBackendKind::from_environment();
    let kernel = Kernel::with_backend(Default::default(), backend, false)
        .map_err(|error| tauri::Error::Anyhow(error.into()))?;
    tauri::Builder::default()
        .manage(AppState {
            kernel: Mutex::new(kernel),
            cancellation: Arc::new(AtomicBool::new(false)),
            backend,
        })
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            execute_cell,
            execute_cell_with_timeout,
            cancel_execution,
            reset_kernel,
            kernel_status,
            configure_kernel,
            open_notebook,
            save_notebook,
            export_notebook_markdown
        ])
        .run(tauri::generate_context!())
}
