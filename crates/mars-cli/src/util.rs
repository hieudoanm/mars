use std::process::Command;

pub fn usage() -> String {
    "usage: mars notebook <new|inspect|open|run|import|export> ... | mars kernel [list|restart|serve] | mars mcp serve ... | mars lab <algorithms|dataset|datasets|train|job|jobs|export> ... | mars tools [list|call <name> [json] [--server <cmd>]]".to_owned()
}

pub fn notebook_usage() -> String {
    "usage: mars notebook new <file.mars> [language] | mars notebook inspect <file.mars> | mars notebook open <file.mars> | mars notebook run <file.mars> [--runtime replay|evcxr] [--timeout-ms <milliseconds>] [--offline] [--locked] [--deterministic] [--no-save] [--report] | mars notebook import <file.ipynb> [file.mars] | mars notebook export <file.mars> [ipynb|html|md|pdf] [--page-size=<a4|letter|legal|...>] [output]".to_owned()
}

pub fn open_with_system(path: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let mut command = Command::new("open");
    #[cfg(target_os = "linux")]
    let mut command = Command::new("xdg-open");
    #[cfg(target_os = "windows")]
    let mut command = {
        let mut command = Command::new("cmd");
        command.arg("/C").arg("start");
        command
    };
    command
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("failed to open {path}: {error}"))
}

pub fn default_mars_path(input: &str) -> std::path::PathBuf {
    std::path::Path::new(input).with_extension("mars")
}

pub fn default_ipynb_path(input: &str) -> std::path::PathBuf {
    std::path::Path::new(input).with_extension("ipynb")
}

pub fn default_export_path(input: &str, format: &str) -> std::path::PathBuf {
    match format {
        "html" => std::path::Path::new(input).with_extension("html"),
        "md" | "markdown" => std::path::Path::new(input).with_extension("md"),
        "pdf" => std::path::Path::new(input).with_extension("pdf"),
        _ => default_ipynb_path(input),
    }
}
