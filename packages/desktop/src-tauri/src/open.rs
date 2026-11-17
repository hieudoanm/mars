use std::path::Path;

/// Open the containing directory of `path` in the platform's terminal
/// emulator. Best-effort: returns an error only when no candidate exists.
pub(crate) fn open_in_terminal(path: &str) -> Result<(), String> {
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

pub(crate) fn open_path_external(path: &str) -> Result<(), String> {
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
