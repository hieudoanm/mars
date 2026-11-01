use std::process::Command;

pub fn usage() -> String {
    "usage: mars new <file.mars> [language] | mars open <file.mars> | mars inspect <file.mars> | mars run <file.mars> [--runtime replay|evcxr] [--timeout-ms <milliseconds>] [--offline] [--locked] [--deterministic] [--no-save] [--report] | mars import <file.ipynb> [file.mars] | mars export <file.mars> [ipynb|html|md|pdf] [--page-size=<a4|letter|legal|...>] [output] | mars kernel [list|restart|serve [--runtime replay|evcxr] [--notebook <file.mars>] [--language <lang>]] | mars mcp serve [--runtime replay|evcxr] [--offline] [--notebook <file.mars>] [--language <lang>] | mars tools [list|call <name> [json] [--server <cmd>]]".to_owned()
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
