use std::path::Path;

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
    let mut timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_millis();
    // Two checkpoints created within the same millisecond would collide, so
    // bump the timestamp until the target name is free.
    let checkpoint = loop {
        let candidate = directory.join(format!("checkpoint-{timestamp}.mars"));
        if !candidate.exists() {
            break candidate;
        }
        timestamp += 1;
    };
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
