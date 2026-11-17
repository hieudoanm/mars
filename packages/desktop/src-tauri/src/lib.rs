pub mod checkpoints;
pub mod kernel;
pub mod kernel_exec;
pub mod kernel_query;
pub mod lab;
pub mod lsp;
pub mod lsp_cmds;
pub mod notebook_cmds;
pub mod notebook_files;
pub mod open;

use lab::{
    lab_algorithms, lab_dataset_insights, lab_dataset_preview, lab_export_artefact, lab_get_job,
    lab_list_datasets, lab_list_jobs, lab_list_jobs_page, lab_train, lab_upload_dataset,
    read_table_preview,
};
use mars_kernel::{Kernel, RuntimeBackendKind};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, atomic::AtomicBool};
use tauri::Manager;

pub struct AppState {
    kernels: Mutex<HashMap<(String, String), Kernel>>,
    input_writers: Mutex<HashMap<(String, String), mars_runtime::StdinWriter>>,
    python_options: Mutex<HashMap<String, mars_kernel::PythonOptions>>,
    sql_sources: Mutex<HashMap<String, Vec<mars_runtime::SqlSource>>>,
    cancellations: Mutex<HashMap<String, Arc<AtomicBool>>>,
    lsp: lsp::LspSessions,
    backend: RuntimeBackendKind,
}

impl AppState {
    pub fn new(backend: RuntimeBackendKind) -> Self {
        Self {
            kernels: Mutex::new(HashMap::new()),
            input_writers: Mutex::new(HashMap::new()),
            python_options: Mutex::new(HashMap::new()),
            sql_sources: Mutex::new(HashMap::new()),
            cancellations: Mutex::new(HashMap::new()),
            lsp: lsp::LspSessions::new(),
            backend,
        }
    }
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
        .manage(AppState::new(backend))
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            kernel_exec::execute_cell,
            kernel_exec::execute_cell_with_timeout,
            kernel_exec::cancel_execution,
            kernel_exec::reset_kernel,
            kernel_exec::kernel_status,
            kernel_query::inspect_variables,
            kernel_query::diagnose_cell,
            kernel_query::send_kernel_input,
            kernel_query::send_kernel_comm,
            kernel_exec::configure_kernel,
            lsp_cmds::lsp_sync,
            lsp_cmds::lsp_hover,
            lsp_cmds::lsp_completion,
            lsp_cmds::lsp_signature_help,
            lsp_cmds::lsp_code_actions,
            lsp_cmds::lsp_definition,
            lsp_cmds::lsp_inlay_hints,
            lsp_cmds::lsp_semantic_tokens,
            lsp_cmds::lsp_diagnostics,
            eval_output_script,
            notebook_cmds::open_notebook,
            notebook_cmds::save_notebook,
            notebook_cmds::notebook_file_mtime,
            notebook_cmds::rename_notebook_file,
            notebook_cmds::list_notebook_files,
            notebook_cmds::create_notebook_file,
            notebook_cmds::create_directory,
            notebook_cmds::delete_path,
            notebook_cmds::write_text_file,
            notebook_cmds::open_in_terminal,
            notebook_cmds::open_path_external,
            notebook_cmds::export_notebook_markdown,
            notebook_cmds::export_notebook_html,
            notebook_cmds::export_notebook_pdf,
            notebook_cmds::export_notebook_slides,
            notebook_cmds::present_notebook_slides,
            notebook_cmds::import_notebook_ipynb,
            notebook_cmds::export_notebook_ipynb,
            notebook_cmds::create_notebook_checkpoint,
            notebook_cmds::list_notebook_checkpoints,
            notebook_cmds::read_notebook_checkpoint,
            notebook_cmds::restore_notebook_checkpoint,
            lab_algorithms,
            lab_upload_dataset,
            lab_list_datasets,
            lab_dataset_insights,
            lab_list_jobs,
            lab_get_job,
            lab_train,
            lab_list_jobs_page,
            lab_dataset_preview,
            read_table_preview,
            lab_export_artefact
        ])
        .run(tauri::generate_context!())
}
