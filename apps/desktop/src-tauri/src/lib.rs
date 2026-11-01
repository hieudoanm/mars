use mars_document::{from_json, load, save_atomic, to_json_pretty};
use mars_kernel::{Kernel, RuntimeBackendKind};
use mars_protocol::{KernelRequest, KernelResponse, RequestEnvelope};
use mars_renderer::render_markdown_notebook;
use mars_runtime::StreamEvent;
use tauri::Emitter;
mod notebook_files;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

struct AppState {
    kernel: Mutex<Kernel>,
    cancellation: Arc<AtomicBool>,
    backend: RuntimeBackendKind,
}

fn stream_sink(app: &tauri::AppHandle) -> impl FnMut(StreamEvent) + '_ {
    move |event| {
        let _ = app.emit("kernel-stream", event);
    }
}

#[tauri::command]
fn execute_cell(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    source: String,
) -> Result<KernelResponse, String> {
    let mut kernel = state
        .kernel
        .lock()
        .map_err(|_| "kernel lock poisoned".to_owned())?;
    state.cancellation.store(false, Ordering::Relaxed);
    let mut sink = stream_sink(&app);
    Ok(kernel.execute_with_events(&source, None, Arc::clone(&state.cancellation), &mut sink))
}

#[tauri::command]
fn execute_cell_with_timeout(
    app: tauri::AppHandle,
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
    let mut sink = stream_sink(&app);
    Ok(kernel.execute_with_events(
        &source,
        Some(Duration::from_millis(timeout_millis)),
        Arc::clone(&state.cancellation),
        &mut sink,
    ))
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
fn notebook_file_mtime(path: String) -> Result<Option<u64>, String> {
    let metadata = match std::fs::metadata(&path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
    };
    let modified = metadata
        .modified()
        .map_err(|error| error.to_string())?
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| error.to_string())?;
    Ok(Some(modified.as_millis() as u64))
}

#[tauri::command]
fn rename_notebook_file(old_path: String, new_path: String) -> Result<(), String> {
    notebook_files::rename_notebook_file(&old_path, &new_path)
}

#[tauri::command]
fn list_notebook_files(path: String) -> Result<Vec<notebook_files::FileExplorerEntry>, String> {
    notebook_files::list_notebook_files(std::path::Path::new(&path))
}

#[tauri::command]
fn export_notebook_markdown(path: String, content: String) -> Result<(), String> {
    let notebook = from_json(&content).map_err(|error| error.to_string())?;
    let markdown = render_markdown_notebook(&notebook);
    std::fs::write(&path, format!("{markdown}\n")).map_err(|error| error.to_string())
}

#[tauri::command]
fn import_notebook_ipynb(path: String) -> Result<String, String> {
    let input = std::fs::read_to_string(&path).map_err(|error| error.to_string())?;
    let notebook = mars_jupyter::import_ipynb(&input).map_err(|error| error.to_string())?;
    to_json_pretty(&notebook).map_err(|error| error.to_string())
}

#[tauri::command]
fn export_notebook_ipynb(path: String, content: String) -> Result<(), String> {
    let notebook = from_json(&content).map_err(|error| error.to_string())?;
    let json = mars_jupyter::export_ipynb(&notebook).map_err(|error| error.to_string())?;
    std::fs::write(&path, json).map_err(|error| error.to_string())
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
            notebook_file_mtime,
            rename_notebook_file,
            list_notebook_files,
            export_notebook_markdown,
            import_notebook_ipynb,
            export_notebook_ipynb
        ])
        .run(tauri::generate_context!())
}
