use crate::{checkpoints, notebook_files, open};
use mars_core::CellType;
use mars_document::{Cell, Notebook, from_json, load, save_atomic, to_json_pretty};
use mars_renderer::{
    render_markdown_notebook, render_notebook, render_pdf_notebook, render_slides_notebook,
};

#[tauri::command]
pub fn open_notebook(path: String) -> Result<String, String> {
    load(&path)
        .and_then(|notebook| to_json_pretty(&notebook))
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn save_notebook(path: String, content: String) -> Result<(), String> {
    let notebook = from_json(&content).map_err(|error| error.to_string())?;
    save_atomic(&notebook, &path).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn notebook_file_mtime(path: String) -> Result<Option<u64>, String> {
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
pub fn rename_notebook_file(old_path: String, new_path: String) -> Result<(), String> {
    notebook_files::rename_path(&old_path, &new_path)
}

#[tauri::command]
pub fn list_notebook_files(path: String) -> Result<Vec<notebook_files::FileExplorerEntry>, String> {
    notebook_files::list_notebook_files(std::path::Path::new(&path))
}

#[tauri::command]
pub fn create_notebook_file(path: String) -> Result<(), String> {
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
pub fn create_directory(path: String) -> Result<(), String> {
    if std::path::Path::new(&path).exists() {
        return Err("A file or folder with that name already exists".to_owned());
    }
    std::fs::create_dir_all(&path).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn delete_path(path: String) -> Result<(), String> {
    notebook_files::delete_path(&path)
}

#[tauri::command]
pub fn write_text_file(path: String, content: String) -> Result<(), String> {
    std::fs::write(&path, content).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn open_in_terminal(path: String) -> Result<(), String> {
    open::open_in_terminal(&path)
}

#[tauri::command]
pub fn open_path_external(path: String) -> Result<(), String> {
    open::open_path_external(&path)
}

#[tauri::command]
pub fn export_notebook_markdown(path: String, content: String) -> Result<(), String> {
    let notebook = from_json(&content).map_err(|error| error.to_string())?;
    let markdown = render_markdown_notebook(&notebook);
    std::fs::write(&path, format!("{markdown}\n")).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn export_notebook_html(path: String, content: String) -> Result<(), String> {
    let notebook = from_json(&content).map_err(|error| error.to_string())?;
    std::fs::write(&path, render_notebook(&notebook)).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn export_notebook_slides(path: String, content: String) -> Result<(), String> {
    let notebook = from_json(&content).map_err(|error| error.to_string())?;
    std::fs::write(&path, render_slides_notebook(&notebook)).map_err(|error| error.to_string())
}

/// Renders the notebook as a slides deck, writes it to a temp file, and opens
/// it in the system browser for presenting.
#[tauri::command]
pub fn present_notebook_slides(content: String) -> Result<String, String> {
    let notebook = from_json(&content).map_err(|error| error.to_string())?;
    let html = render_slides_notebook(&notebook);
    let path = std::env::temp_dir().join(format!(
        "mars-present-{}-{}.html",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    ));
    std::fs::write(&path, html).map_err(|error| error.to_string())?;
    open::open_path_external(path.to_string_lossy().as_ref())?;
    Ok(path.to_string_lossy().into_owned())
}

#[tauri::command]
pub fn export_notebook_pdf(path: String, content: String) -> Result<(), String> {
    let notebook = from_json(&content).map_err(|error| error.to_string())?;
    let bytes = render_pdf_notebook(&notebook, notebook.metadata.page_size);
    std::fs::write(&path, bytes).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn import_notebook_ipynb(path: String) -> Result<String, String> {
    let input = std::fs::read_to_string(&path).map_err(|error| error.to_string())?;
    let notebook = mars_jupyter::import_ipynb(&input).map_err(|error| error.to_string())?;
    to_json_pretty(&notebook).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn create_notebook_checkpoint(path: String) -> Result<String, String> {
    checkpoints::create_checkpoint(&path)
}

#[tauri::command]
pub fn list_notebook_checkpoints(path: String) -> Result<Vec<String>, String> {
    checkpoints::list_checkpoints(&path)
}

#[tauri::command]
pub fn read_notebook_checkpoint(path: String, checkpoint_path: String) -> Result<String, String> {
    checkpoints::read_checkpoint(&path, &checkpoint_path)
}

#[tauri::command]
pub fn restore_notebook_checkpoint(path: String, checkpoint_path: String) -> Result<(), String> {
    checkpoints::restore_checkpoint(&path, &checkpoint_path)
}

#[tauri::command]
pub fn export_notebook_ipynb(path: String, content: String) -> Result<(), String> {
    let notebook = from_json(&content).map_err(|error| error.to_string())?;
    let json = mars_jupyter::export_ipynb(&notebook).map_err(|error| error.to_string())?;
    std::fs::write(&path, json).map_err(|error| error.to_string())
}
