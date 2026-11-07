use serde::Serialize;
use std::path::Path;

const MAX_EXPLORER_DEPTH: usize = 4;

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileExplorerEntry {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub children: Vec<FileExplorerEntry>,
}

pub fn list_notebook_files(path: &Path) -> Result<Vec<FileExplorerEntry>, String> {
    read_directory(path, 0)
}

fn read_directory(path: &Path, depth: usize) -> Result<Vec<FileExplorerEntry>, String> {
    let mut entries = std::fs::read_dir(path)
        .map_err(|error| error.to_string())?
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            if (name.starts_with('.') && name != ".cargo")
                || name.ends_with(".checkpoints")
                || matches!(name.as_str(), "target" | "node_modules")
            {
                return None;
            }
            let file_type = entry.file_type().ok()?;
            let entry_path = entry.path();
            if file_type.is_symlink() || (!file_type.is_dir() && !is_explorer_file(&entry_path)) {
                return None;
            }
            let children = if file_type.is_dir() && depth < MAX_EXPLORER_DEPTH {
                read_directory(&entry_path, depth + 1).unwrap_or_default()
            } else {
                Vec::new()
            };
            Some(FileExplorerEntry {
                name,
                path: entry_path.to_string_lossy().into_owned(),
                is_directory: file_type.is_dir(),
                children,
            })
        })
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| {
        right
            .is_directory
            .cmp(&left.is_directory)
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
    });
    Ok(entries)
}

pub fn rename_path(old_path: &str, new_path: &str) -> Result<(), String> {
    if old_path == new_path {
        return Ok(());
    }
    let destination = Path::new(new_path);
    if destination.exists() && !is_case_only_rename(old_path, new_path) {
        return Err("A file with that name already exists".to_owned());
    }
    std::fs::rename(old_path, new_path).map_err(|error| error.to_string())
}

pub fn delete_path(path: &str) -> Result<(), String> {
    let target = Path::new(path);
    let metadata = std::fs::symlink_metadata(target).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink() || metadata.is_file() {
        return std::fs::remove_file(target).map_err(|error| error.to_string());
    }
    if metadata.is_dir() {
        return std::fs::remove_dir_all(target).map_err(|error| error.to_string());
    }
    Err("Unsupported file type".to_owned())
}

pub fn checkpoints_directory(notebook_path: &str) -> Result<String, String> {
    let notebook = Path::new(notebook_path);
    let file_name = notebook
        .file_name()
        .ok_or_else(|| "Invalid notebook path".to_owned())?
        .to_string_lossy();
    let parent = notebook.parent().unwrap_or_else(|| Path::new("."));
    Ok(parent
        .join(format!("{file_name}.checkpoints"))
        .to_string_lossy()
        .into_owned())
}

pub fn create_checkpoint(notebook_path: &str) -> Result<String, String> {
    let source = Path::new(notebook_path);
    if !source.is_file() {
        return Err("Notebook file does not exist".to_owned());
    }
    let directory_path = checkpoints_directory(notebook_path)?;
    let directory = Path::new(&directory_path);
    std::fs::create_dir_all(directory).map_err(|error| error.to_string())?;
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_millis();
    let checkpoint = directory.join(format!("checkpoint-{timestamp}.mars"));
    std::fs::copy(source, &checkpoint).map_err(|error| error.to_string())?;
    Ok(checkpoint.to_string_lossy().into_owned())
}

pub fn list_checkpoints(notebook_path: &str) -> Result<Vec<String>, String> {
    let directory_path = checkpoints_directory(notebook_path)?;
    let directory = Path::new(&directory_path);
    if !directory.is_dir() {
        return Ok(Vec::new());
    }
    let mut checkpoints = std::fs::read_dir(directory)
        .map_err(|error| error.to_string())?
        .filter_map(Result::ok)
        .map(|entry| entry.path().to_string_lossy().into_owned())
        .filter(|path| path.ends_with(".mars"))
        .collect::<Vec<_>>();
    checkpoints.sort();
    Ok(checkpoints)
}

pub fn read_checkpoint(notebook_path: &str, checkpoint: &str) -> Result<String, String> {
    let allowed = checkpoints_directory(notebook_path)?;
    let checkpoint_path = Path::new(checkpoint);
    if !checkpoint_path.starts_with(Path::new(&allowed)) || !checkpoint_path.is_file() {
        return Err("Checkpoint is outside the notebook checkpoint directory".to_owned());
    }
    std::fs::read_to_string(checkpoint_path).map_err(|error| error.to_string())
}

pub fn restore_checkpoint(notebook_path: &str, checkpoint: &str) -> Result<(), String> {
    let allowed = checkpoints_directory(notebook_path)?;
    let checkpoint_path = Path::new(checkpoint);
    if !checkpoint_path.starts_with(Path::new(&allowed)) || !checkpoint_path.is_file() {
        return Err("Checkpoint is outside the notebook checkpoint directory".to_owned());
    }
    std::fs::copy(checkpoint_path, notebook_path)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn is_explorer_file(path: &Path) -> bool {
    if path.file_name().is_some_and(|name| name == "Cargo.lock") {
        return true;
    }
    path.extension().is_some_and(|extension| {
        extension.eq_ignore_ascii_case("mars")
            || extension.eq_ignore_ascii_case("rs")
            || extension.eq_ignore_ascii_case("toml")
    })
}

fn is_case_only_rename(old_path: &str, new_path: &str) -> bool {
    if old_path == new_path || !old_path.eq_ignore_ascii_case(new_path) {
        return false;
    }
    match (
        std::fs::canonicalize(old_path),
        std::fs::canonicalize(new_path),
    ) {
        (Ok(source), Ok(destination)) => source == destination,
        _ => false,
    }
}

/// Open the containing directory of `path` in the platform's terminal
/// emulator. Best-effort: returns an error only when no candidate exists.
pub fn open_in_terminal(path: &str) -> Result<(), String> {
    let target = std::path::Path::new(path)
        .parent()
        .map(|p| p.to_path_buf())
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    open_directory_in_terminal(&target)
}

#[cfg(target_os = "macos")]
fn open_directory_in_terminal(dir: &std::path::Path) -> Result<(), String> {
    std::process::Command::new("open")
        .arg("-a")
        .arg("Terminal")
        .arg(dir)
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[cfg(target_os = "windows")]
fn open_directory_in_terminal(dir: &std::path::Path) -> Result<(), String> {
    std::process::Command::new("cmd")
        .args(["/c", "start", ""])
        .arg(dir)
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[cfg(all(unix, not(target_os = "macos")))]
fn open_directory_in_terminal(dir: &std::path::Path) -> Result<(), String> {
    for terminal in [
        "x-terminal-emulator",
        "gnome-terminal",
        "konsole",
        "alacritty",
        "kitty",
        "xterm",
    ] {
        if std::process::Command::new(terminal)
            .arg(dir)
            .spawn()
            .is_ok()
        {
            return Ok(());
        }
    }
    Err("No terminal emulator found".to_owned())
}

pub fn open_path_external(path: &str) -> Result<(), String> {
    let target = Path::new(path);
    if !target.exists() {
        return Err("File does not exist".to_owned());
    }
    open_external(target)
}

#[cfg(target_os = "macos")]
fn open_external(path: &Path) -> Result<(), String> {
    std::process::Command::new("open")
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[cfg(target_os = "windows")]
fn open_external(path: &Path) -> Result<(), String> {
    std::process::Command::new("cmd")
        .args(["/c", "start", ""])
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[cfg(all(unix, not(target_os = "macos")))]
fn open_external(path: &Path) -> Result<(), String> {
    std::process::Command::new("xdg-open")
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEMPORARY_DIRECTORY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    fn temporary_directory() -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let sequence = TEMPORARY_DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("mars-tauri-{nonce}-{sequence}"));
        std::fs::create_dir_all(&path).expect("creates temporary directory");
        path
    }

    #[test]
    fn lists_notebooks_and_rust_project_files() {
        let directory = temporary_directory();
        std::fs::create_dir(directory.join("notebooks")).expect("creates child directory");
        std::fs::write(directory.join("analysis.mars"), "{}").expect("creates notebook");
        std::fs::write(directory.join("Cargo.toml"), "[package]").expect("creates manifest");
        std::fs::write(directory.join("Cargo.lock"), "").expect("creates lockfile");
        std::fs::write(directory.join("main.rs"), "fn main() {}").expect("creates source");
        std::fs::write(directory.join("notes.txt"), "ignored").expect("creates text file");
        std::fs::create_dir(directory.join("target")).expect("creates target directory");
        std::fs::write(directory.join("target").join("build.rs"), "").expect("creates build file");

        let entries = list_notebook_files(&directory).expect("lists directory");
        let names = entries
            .iter()
            .map(|entry| entry.name.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            names,
            [
                "notebooks",
                "analysis.mars",
                "Cargo.lock",
                "Cargo.toml",
                "main.rs"
            ]
        );
        assert!(!names.contains(&"target"));
        assert!(!names.contains(&"notes.txt"));
        std::fs::remove_dir_all(directory).expect("removes temporary directory");
    }

    #[test]
    fn ignores_hidden_entries() {
        let directory = temporary_directory();
        std::fs::create_dir(directory.join(".git")).expect("creates hidden directory");
        std::fs::create_dir(directory.join("visible")).expect("creates visible directory");
        std::fs::write(directory.join(".hidden.mars"), "{}").expect("creates hidden notebook");

        let entries = list_notebook_files(&directory).expect("lists directory");

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "visible");
        std::fs::remove_dir_all(directory).expect("removes temporary directory");
    }

    #[test]
    fn refuses_to_overwrite_an_existing_distinct_file() {
        let directory = temporary_directory();
        let source = directory.join("source.mars");
        let destination = directory.join("destination.mars");
        std::fs::write(&source, "source").expect("creates source");
        std::fs::write(&destination, "destination").expect("creates destination");

        let error = rename_path(&source.to_string_lossy(), &destination.to_string_lossy())
            .expect_err("does not overwrite destination");

        assert_eq!(error, "A file with that name already exists");
        assert_eq!(
            std::fs::read_to_string(destination).expect("reads destination"),
            "destination"
        );
        std::fs::remove_dir_all(directory).expect("removes temporary directory");
    }
}
