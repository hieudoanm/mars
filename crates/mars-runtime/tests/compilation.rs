use mars_runtime::compilation::{CompileOptions, compile_command, compile_program};
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::atomic::AtomicBool;

#[test]
fn compile_command_uses_rustc_without_dependencies() {
    let workdir = std::env::temp_dir();
    let (compiler, args) = compile_command(
        &workdir,
        Path::new("session.rs"),
        Path::new("session-bin"),
        &BTreeMap::new(),
        None,
        false,
        false,
    )
    .expect("compile command");

    assert_eq!(compiler, "rustc");
    assert!(args.contains(&"--edition=2024".to_owned()));
    assert!(args.contains(&"--error-format=json".to_owned()));
    assert!(args.contains(&"session.rs".to_owned()));
    assert!(args.contains(&"-o".to_owned()));
    assert!(args.contains(&"session-bin".to_owned()));
}

#[test]
fn compile_command_uses_cargo_with_dependencies() {
    let dir = std::env::temp_dir().join(format!("mars-compile-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    let source = dir.join("session.rs");
    std::fs::write(&source, "fn main() {}\n").expect("source");

    let mut dependencies = BTreeMap::new();
    dependencies.insert("serde".to_owned(), "1".to_owned());
    let (compiler, args) = compile_command(
        &dir,
        &source,
        &dir.join("session-bin"),
        &dependencies,
        Some("# lockfile\n"),
        true,
        true,
    )
    .expect("compile command");

    assert_eq!(compiler, "cargo");
    assert!(args.contains(&"build".to_owned()));
    assert!(args.contains(&"--offline".to_owned()));
    assert!(args.contains(&"--locked".to_owned()));
    assert!(dir.join("Cargo.toml").exists());
    assert_eq!(
        std::fs::read_to_string(dir.join("Cargo.lock")).expect("lock"),
        "# lockfile\n"
    );
    assert_eq!(
        std::fs::read_to_string(dir.join("src/main.rs")).expect("main"),
        "fn main() {}\n"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn compile_program_builds_and_runs_rustc_output() {
    let dir = std::env::temp_dir().join(format!("mars-compile-run-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    let source = dir.join("session.rs");
    let binary = dir.join("session-bin");
    let cancellation = AtomicBool::new(false);
    let options = CompileOptions {
        dependencies: &BTreeMap::new(),
        cargo_lock: None,
        offline: false,
        locked: false,
        timeout: None,
        cancellation: &cancellation,
    };

    let output = compile_program(
        &dir,
        &source,
        &binary,
        "fn main() { println!(\"compiled\"); }\n".to_owned(),
        &options,
    )
    .expect("compile");

    assert!(output.status.is_some_and(|status| status.success()));
    assert!(binary.exists());

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn compile_command_reports_io_errors_for_cargo_projects() {
    let dir = std::env::temp_dir().join(format!("mars-compile-err-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);

    // Workdir that is an existing file: create_dir_all must fail.
    let file_path = dir.with_extension("not-a-dir");
    std::fs::create_dir_all(&dir).expect("temp dir");
    std::fs::write(&file_path, b"occupied").expect("marker file");
    let mut dependencies = BTreeMap::new();
    dependencies.insert("serde".to_owned(), "1".to_owned());
    let result = compile_command(
        &file_path,
        Path::new("session.rs"),
        Path::new("session-bin"),
        &dependencies,
        None,
        false,
        false,
    );
    assert!(result.is_err());

    // Missing source file: manifest writes succeed, fs::copy must fail.
    let missing_source = dir.join("does-not-exist.rs");
    let result = compile_command(
        &dir,
        &missing_source,
        Path::new("session-bin"),
        &dependencies,
        Some("# lock\n"),
        true,
        false,
    );
    assert!(result.is_err());

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn compile_program_reports_unwritable_source_path() {
    let dir = std::env::temp_dir().join(format!("mars-compile-prog-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    let cancellation = AtomicBool::new(false);
    let options = CompileOptions {
        dependencies: &BTreeMap::new(),
        cargo_lock: None,
        offline: false,
        locked: false,
        timeout: None,
        cancellation: &cancellation,
    };
    // Parent directory does not exist, so writing the source fails.
    let bad_source = dir.join("missing").join("session.rs");
    let result = compile_program(
        &dir,
        &bad_source,
        &dir.join("session-bin"),
        "fn main() {}\n".to_owned(),
        &options,
    );
    assert!(result.is_err());
    let _ = std::fs::remove_dir_all(&dir);
}
