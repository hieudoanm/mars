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
            if name.starts_with('.') {
                return None;
            }
            let file_type = entry.file_type().ok()?;
            let entry_path = entry.path();
            if file_type.is_symlink() || (!file_type.is_dir() && !is_notebook(&entry_path)) {
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

pub fn rename_notebook_file(old_path: &str, new_path: &str) -> Result<(), String> {
    if old_path == new_path {
        return Ok(());
    }
    let destination = Path::new(new_path);
    if destination.exists() && !is_case_only_rename(old_path, new_path) {
        return Err("A file with that name already exists".to_owned());
    }
    std::fs::rename(old_path, new_path).map_err(|error| error.to_string())
}

fn is_notebook(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("mars"))
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
    fn lists_directories_and_mars_notebooks_only() {
        let directory = temporary_directory();
        std::fs::create_dir(directory.join("notebooks")).expect("creates child directory");
        std::fs::write(directory.join("analysis.mars"), "{}").expect("creates notebook");
        std::fs::write(directory.join("notes.txt"), "ignored").expect("creates text file");

        let entries = list_notebook_files(&directory).expect("lists directory");

        assert_eq!(entries.len(), 2);
        assert!(entries[0].is_directory);
        assert_eq!(entries[1].name, "analysis.mars");
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

        let error = rename_notebook_file(&source.to_string_lossy(), &destination.to_string_lossy())
            .expect_err("does not overwrite destination");

        assert_eq!(error, "A file with that name already exists");
        assert_eq!(
            std::fs::read_to_string(destination).expect("reads destination"),
            "destination"
        );
        std::fs::remove_dir_all(directory).expect("removes temporary directory");
    }
}
