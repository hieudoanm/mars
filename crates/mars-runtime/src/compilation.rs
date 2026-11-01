use crate::error::RuntimeError;
use crate::execution::configured_command;
use crate::output::BinaryOutput;
use std::{
    collections::BTreeMap, fs, path::Path, process::Stdio, sync::atomic::AtomicBool, time::Duration,
};

pub struct CompileOptions<'a> {
    pub dependencies: &'a BTreeMap<String, String>,
    pub cargo_lock: Option<&'a str>,
    pub offline: bool,
    pub locked: bool,
    pub timeout: Option<Duration>,
    pub cancellation: &'a AtomicBool,
}

pub fn compile_command(
    workdir: &Path,
    source_path: &Path,
    binary_path: &Path,
    dependencies: &BTreeMap<String, String>,
    cargo_lock: Option<&str>,
    offline: bool,
    locked: bool,
) -> Result<(String, Vec<String>), RuntimeError> {
    if dependencies.is_empty() {
        return Ok((
            "rustc".to_owned(),
            vec![
                "--edition=2024".to_owned(),
                "--error-format=json".to_owned(),
                source_path.to_string_lossy().into_owned(),
                "-o".to_owned(),
                binary_path.to_string_lossy().into_owned(),
            ],
        ));
    }

    let source_directory = workdir.join("src");
    fs::create_dir_all(&source_directory).map_err(|source| RuntimeError::Io {
        path: source_directory.clone(),
        source,
    })?;
    fs::write(workdir.join("Cargo.toml"), cargo_manifest(dependencies)).map_err(|source| {
        RuntimeError::Io {
            path: workdir.join("Cargo.toml"),
            source,
        }
    })?;
    if let Some(lock) = cargo_lock {
        fs::write(workdir.join("Cargo.lock"), lock).map_err(|source| RuntimeError::Io {
            path: workdir.join("Cargo.lock"),
            source,
        })?;
    }
    fs::copy(source_path, source_directory.join("main.rs")).map_err(|source| RuntimeError::Io {
        path: source_directory.join("main.rs"),
        source,
    })?;
    let mut args = vec![
        "build".to_owned(),
        "--manifest-path".to_owned(),
        workdir.join("Cargo.toml").to_string_lossy().into_owned(),
    ];
    if offline {
        args.push("--offline".to_owned());
    }
    if locked {
        args.push("--locked".to_owned());
    }
    Ok(("cargo".to_owned(), args))
}

pub fn compile_program(
    workdir: &Path,
    source_path: &Path,
    binary_path: &Path,
    program: String,
    options: &CompileOptions<'_>,
) -> Result<BinaryOutput, RuntimeError> {
    fs::write(source_path, program).map_err(|source| RuntimeError::Io {
        path: source_path.to_path_buf(),
        source,
    })?;
    let (compiler, compiler_args) = compile_command(
        workdir,
        source_path,
        binary_path,
        options.dependencies,
        options.cargo_lock,
        options.offline,
        options.locked,
    )?;
    let child = configured_command(compiler)
        .args(compiler_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(RuntimeError::CompilerUnavailable)?;
    crate::execution::run_child(child, options.timeout, options.cancellation)
}

pub use crate::diagnostics::format_compiler_diagnostics;
pub use crate::manifest::{cargo_manifest, validate_dependencies};
