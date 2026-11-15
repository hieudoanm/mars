use crate::lab::{
    lab_algorithms, lab_dataset_insights, lab_dataset_preview, lab_export_artefact, lab_get_job,
    lab_list_datasets, lab_list_jobs, lab_list_jobs_page, lab_train, lab_upload_dataset,
};
use mars_core::CellType;
use mars_document::{Cell, Notebook, from_json, load, save_atomic, to_json_pretty};
use mars_kernel::{Kernel, RuntimeBackendKind};
use mars_protocol::{KernelRequest, KernelResponse, RequestEnvelope};
use mars_renderer::{render_markdown_notebook, render_notebook, render_pdf_notebook};
use mars_runtime::StreamEvent;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;
use tauri::{Emitter, Manager};

mod lab;
mod lsp;
mod notebook_files;

struct AppState {
    kernels: Mutex<HashMap<(String, String), Kernel>>,
    input_writers: Mutex<HashMap<(String, String), mars_runtime::StdinWriter>>,
    python_options: Mutex<HashMap<String, mars_kernel::PythonOptions>>,
    cancellations: Mutex<HashMap<String, Arc<AtomicBool>>>,
    lsp: lsp::LspSessions,
    backend: RuntimeBackendKind,
}

#[derive(Clone, Serialize)]
struct StreamMessage<'a> {
    session: &'a str,
    language: &'a str,
    #[serde(flatten)]
    event: &'a StreamEvent,
}

fn stream_sink<'a>(
    app: &'a tauri::AppHandle,
    session: &'a str,
    language: &'a str,
) -> impl FnMut(StreamEvent) + 'a {
    move |event| {
        let _ = app.emit(
            "kernel-stream",
            StreamMessage {
                session,
                language,
                event: &event,
            },
        );
    }
}

fn kernel_key(session: &str, language: &str) -> (String, String) {
    (session.to_owned(), language.to_owned())
}

fn kernel_for<'a>(
    state: &'a AppState,
    kernels: &'a mut HashMap<(String, String), Kernel>,
    session: &str,
    language: &str,
    backend: RuntimeBackendKind,
) -> Result<&'a mut Kernel, String> {
    let key = kernel_key(session, language);
    if !kernels.contains_key(&key) {
        let kernel = if language == "python" {
            let options = state
                .python_options
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .get(session)
                .cloned()
                .unwrap_or_default();
            Kernel::for_language_with_python_options(language, backend, options)
        } else {
            Kernel::for_language(language, backend)
        }
        .map_err(|error| error.to_string())?;
        kernels.insert(key.clone(), kernel);
    }
    kernels
        .get_mut(&key)
        .ok_or_else(|| "kernel session unavailable".to_owned())
}

/// Registers the kernel's stdin writer so interactive prompts can be
/// answered while `execute_*` holds the kernels mutex.
fn register_input_writer(state: &AppState, session: &str, language: &str, kernel: &Kernel) {
    if let Some(writer) = kernel.stdin_writer()
        && let Ok(mut writers) = state.input_writers.lock()
    {
        writers.insert(kernel_key(session, language), writer);
    }
}

fn cancellation_for(state: &AppState, session: &str) -> Arc<AtomicBool> {
    let mut cancellations = state
        .cancellations
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    Arc::clone(
        cancellations
            .entry(session.to_owned())
            .or_insert_with(|| Arc::new(AtomicBool::new(false))),
    )
}

#[tauri::command]
fn execute_cell(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    session: String,
    source: String,
    language: String,
) -> Result<KernelResponse, String> {
    let mut kernels = state
        .kernels
        .lock()
        .map_err(|_| "kernel lock poisoned".to_owned())?;
    let kernel = kernel_for(&state, &mut kernels, &session, &language, state.backend)?;
    register_input_writer(&state, &session, &language, kernel);
    let cancellation = cancellation_for(&state, &session);
    cancellation.store(false, Ordering::Relaxed);
    let mut sink = stream_sink(&app, &session, &language);
    Ok(kernel.execute_with_events(&source, None, cancellation, &mut sink))
}

#[tauri::command]
fn execute_cell_with_timeout(
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
    let mut kernels = state
        .kernels
        .lock()
        .map_err(|_| "kernel lock poisoned".to_owned())?;
    let kernel = kernel_for(&state, &mut kernels, &session, &language, state.backend)?;
    register_input_writer(&state, &session, &language, kernel);
    let cancellation = cancellation_for(&state, &session);
    cancellation.store(false, Ordering::Relaxed);
    let mut sink = stream_sink(&app, &session, &language);
    Ok(kernel.execute_with_events(
        &source,
        Some(Duration::from_millis(timeout_millis)),
        cancellation,
        &mut sink,
    ))
}

#[tauri::command]
fn cancel_execution(state: tauri::State<'_, AppState>, session: String) -> Result<(), String> {
    cancellation_for(&state, &session).store(true, Ordering::Relaxed);
    Ok(())
}

#[tauri::command]
fn reset_kernel(
    state: tauri::State<'_, AppState>,
    session: String,
    language: String,
) -> Result<KernelResponse, String> {
    let mut kernels = state
        .kernels
        .lock()
        .map_err(|_| "kernel lock poisoned".to_owned())?;
    let kernel = kernel_for(&state, &mut kernels, &session, &language, state.backend)?;
    Ok(kernel
        .handle(RequestEnvelope::new(KernelRequest::Reset))
        .response)
}

#[tauri::command]
fn inspect_variables(
    state: tauri::State<'_, AppState>,
    session: String,
    language: String,
) -> Result<Vec<mars_runtime::VariableInfo>, String> {
    let mut kernels = state
        .kernels
        .lock()
        .map_err(|_| "kernel lock poisoned".to_owned())?;
    let kernel = kernel_for(&state, &mut kernels, &session, &language, state.backend)?;
    kernel
        .inspect_variables()
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn diagnose_cell(
    state: tauri::State<'_, AppState>,
    session: String,
    source: String,
    language: String,
) -> Result<Vec<mars_runtime::CellDiagnostic>, String> {
    let mut kernels = state
        .kernels
        .lock()
        .map_err(|_| "kernel lock poisoned".to_owned())?;
    let kernel = kernel_for(&state, &mut kernels, &session, &language, state.backend)?;
    kernel.diagnose(&source).map_err(|error| error.to_string())
}

#[tauri::command]
fn send_kernel_input(
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
    let mut kernels = state
        .kernels
        .lock()
        .map_err(|_| "kernel lock poisoned".to_owned())?;
    let kernel = kernels
        .get_mut(&kernel_key(&session, &language))
        .ok_or_else(|| "kernel session unavailable".to_owned())?;
    kernel.write_stdin(&text).map_err(|error| error.to_string())
}

#[tauri::command]
fn send_kernel_comm(
    state: tauri::State<'_, AppState>,
    session: String,
    comm_id: String,
    comm_type: String,
    data: serde_json::Value,
    language: String,
) -> Result<Vec<mars_core::CommMessage>, String> {
    let mut kernels = state
        .kernels
        .lock()
        .map_err(|_| "kernel lock poisoned".to_owned())?;
    let kernel = kernels
        .get_mut(&kernel_key(&session, &language))
        .ok_or_else(|| "kernel session unavailable".to_owned())?;
    kernel
        .handle_comm(&comm_id, &comm_type, &data)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn kernel_status(
    state: tauri::State<'_, AppState>,
    session: String,
    language: String,
) -> Result<KernelResponse, String> {
    let mut kernels = state
        .kernels
        .lock()
        .map_err(|_| "kernel lock poisoned".to_owned())?;
    let kernel = kernel_for(&state, &mut kernels, &session, &language, state.backend)?;
    Ok(kernel
        .handle(RequestEnvelope::new(KernelRequest::Status))
        .response)
}

#[tauri::command]
fn configure_kernel(
    state: tauri::State<'_, AppState>,
    session: String,
    content: String,
) -> Result<(), String> {
    let notebook = from_json(&content).map_err(|error| error.to_string())?;
    let python_options = mars_kernel::python_options_from_metadata(&notebook.metadata.extra);
    if let Ok(mut options) = state.python_options.lock() {
        options.insert(session.clone(), python_options.clone());
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
    let mut kernels = state
        .kernels
        .lock()
        .map_err(|_| "kernel lock poisoned".to_owned())?;
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
    notebook_files::rename_path(&old_path, &new_path)
}

#[tauri::command]
fn list_notebook_files(path: String) -> Result<Vec<notebook_files::FileExplorerEntry>, String> {
    notebook_files::list_notebook_files(std::path::Path::new(&path))
}

#[tauri::command]
fn create_notebook_file(path: String) -> Result<(), String> {
    if std::path::Path::new(&path).exists() {
        return Err("A file with that name already exists".to_owned());
    }
    let mut notebook = Notebook::new();
    notebook
        .cells
        .push(Cell::new(CellType::Markdown, "# Untitled notebook"));
    save_atomic(&notebook, &path).map_err(|error| error.to_string())
}

#[tauri::command]
fn create_directory(path: String) -> Result<(), String> {
    if std::path::Path::new(&path).exists() {
        return Err("A file or folder with that name already exists".to_owned());
    }
    std::fs::create_dir_all(&path).map_err(|error| error.to_string())
}

#[tauri::command]
fn delete_path(path: String) -> Result<(), String> {
    notebook_files::delete_path(&path)
}

#[tauri::command]
fn write_text_file(path: String, content: String) -> Result<(), String> {
    std::fs::write(&path, content).map_err(|error| error.to_string())
}

#[tauri::command]
fn open_in_terminal(path: String) -> Result<(), String> {
    notebook_files::open_in_terminal(&path)
}

#[tauri::command]
fn open_path_external(path: String) -> Result<(), String> {
    notebook_files::open_path_external(&path)
}

#[tauri::command]
fn export_notebook_markdown(path: String, content: String) -> Result<(), String> {
    let notebook = from_json(&content).map_err(|error| error.to_string())?;
    let markdown = render_markdown_notebook(&notebook);
    std::fs::write(&path, format!("{markdown}\n")).map_err(|error| error.to_string())
}

#[tauri::command]
fn export_notebook_html(path: String, content: String) -> Result<(), String> {
    let notebook = from_json(&content).map_err(|error| error.to_string())?;
    std::fs::write(&path, render_notebook(&notebook)).map_err(|error| error.to_string())
}

#[tauri::command]
fn export_notebook_pdf(path: String, content: String) -> Result<(), String> {
    let notebook = from_json(&content).map_err(|error| error.to_string())?;
    let bytes = render_pdf_notebook(&notebook, notebook.metadata.page_size);
    std::fs::write(&path, bytes).map_err(|error| error.to_string())
}

#[tauri::command]
fn import_notebook_ipynb(path: String) -> Result<String, String> {
    let input = std::fs::read_to_string(&path).map_err(|error| error.to_string())?;
    let notebook = mars_jupyter::import_ipynb(&input).map_err(|error| error.to_string())?;
    to_json_pretty(&notebook).map_err(|error| error.to_string())
}

#[tauri::command]
fn create_notebook_checkpoint(path: String) -> Result<String, String> {
    notebook_files::create_checkpoint(&path)
}

#[tauri::command]
fn list_notebook_checkpoints(path: String) -> Result<Vec<String>, String> {
    notebook_files::list_checkpoints(&path)
}

#[tauri::command]
fn read_notebook_checkpoint(path: String, checkpoint_path: String) -> Result<String, String> {
    notebook_files::read_checkpoint(&path, &checkpoint_path)
}

#[tauri::command]
fn restore_notebook_checkpoint(path: String, checkpoint_path: String) -> Result<(), String> {
    notebook_files::restore_checkpoint(&path, &checkpoint_path)
}

#[tauri::command]
fn export_notebook_ipynb(path: String, content: String) -> Result<(), String> {
    let notebook = from_json(&content).map_err(|error| error.to_string())?;
    let json = mars_jupyter::export_ipynb(&notebook).map_err(|error| error.to_string())?;
    std::fs::write(&path, json).map_err(|error| error.to_string())
}

#[tauri::command]
fn lsp_sync(state: tauri::State<AppState>, request: lsp::SyncRequest) -> Result<(), String> {
    state
        .lsp
        .sync(&request.session, request.cells, request.dependencies)
}

#[tauri::command]
fn lsp_hover(
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
fn lsp_completion(
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
fn lsp_signature_help(
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
fn lsp_code_actions(
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
fn lsp_definition(
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
fn lsp_inlay_hints(
    state: tauri::State<AppState>,
    session: String,
) -> Result<Vec<mars_lsp::CellInlayHint>, String> {
    state.lsp.inlay_hints(&session)
}

#[tauri::command]
fn lsp_semantic_tokens(
    state: tauri::State<AppState>,
    session: String,
) -> Result<Vec<mars_lsp::CellSemanticToken>, String> {
    state.lsp.semantic_tokens(&session)
}

#[tauri::command]
fn lsp_diagnostics(
    state: tauri::State<AppState>,
    session: String,
) -> Result<Vec<mars_lsp::CellDiagnostic>, String> {
    state.lsp.diagnostics(&session)
}

#[tauri::command]
fn eval_output_script(app: tauri::AppHandle, script: String) -> Result<(), String> {
    let window = app
        .webview_windows()
        .into_values()
        .next()
        .ok_or_else(|| "no webview window available".to_owned())?;
    window.eval(&script).map_err(|error| error.to_string())
}

pub fn run() -> tauri::Result<()> {
    let backend = RuntimeBackendKind::from_environment();
    tauri::Builder::default()
        .manage(AppState {
            kernels: Mutex::new(HashMap::new()),
            input_writers: Mutex::new(HashMap::new()),
            python_options: Mutex::new(HashMap::new()),
            cancellations: Mutex::new(HashMap::new()),
            lsp: lsp::LspSessions::new(),
            backend,
        })
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            execute_cell,
            execute_cell_with_timeout,
            cancel_execution,
            reset_kernel,
            kernel_status,
            inspect_variables,
            diagnose_cell,
            send_kernel_input,
            send_kernel_comm,
            lsp_sync,
            lsp_hover,
            lsp_completion,
            lsp_signature_help,
            lsp_code_actions,
            lsp_definition,
            lsp_inlay_hints,
            lsp_semantic_tokens,
            lsp_diagnostics,
            eval_output_script,
            configure_kernel,
            open_notebook,
            save_notebook,
            notebook_file_mtime,
            rename_notebook_file,
            list_notebook_files,
            create_notebook_file,
            create_directory,
            delete_path,
            write_text_file,
            open_in_terminal,
            open_path_external,
            export_notebook_markdown,
            export_notebook_html,
            export_notebook_pdf,
            import_notebook_ipynb,
            export_notebook_ipynb,
            create_notebook_checkpoint,
            list_notebook_checkpoints,
            read_notebook_checkpoint,
            restore_notebook_checkpoint,
            lab_algorithms,
            lab_upload_dataset,
            lab_list_datasets,
            lab_dataset_insights,
            lab_list_jobs,
            lab_get_job,
            lab_train,
            lab_list_jobs_page,
            lab_dataset_preview,
            lab_export_artefact
        ])
        .run(tauri::generate_context!())
}
