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
