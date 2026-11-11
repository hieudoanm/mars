use std::path::Path;
use std::process::Command;

/// Renders the complete C++ program from accumulated cell sources.
pub fn render_program(cells: &[String]) -> String {
    let mut program = String::new();

    // Standard includes
    program.push_str("#include <iostream>\n");
    program.push_str("#include <vector>\n");
    program.push_str("#include <string>\n");
    program.push_str("#include <algorithm>\n");
    program.push_str("#include <map>\n");
    program.push_str("#include <optional>\n\n");

    // Accumulate includes from cells (simple heuristic: lines starting with #include)
    let mut includes = Vec::new();
    for cell in cells {
        for line in cell.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("#include") && !includes.contains(&trimmed.to_string()) {
                includes.push(trimmed.to_string());
            }
        }
    }

    // Add accumulated includes
    for include in &includes {
        program.push_str(include);
        program.push('\n');
    }

    program.push_str("\nint main() {\n");

    // Add cell code, stripping includes from cell bodies
    for cell in cells {
        for line in cell.lines() {
            let trimmed = line.trim();
            if !trimmed.starts_with("#include") {
                program.push_str("    ");
                program.push_str(line);
                program.push('\n');
            }
        }
    }

    program.push_str("    return 0;\n");
    program.push_str("}\n");

    program
}

/// Constructs the compilation command for C++.
pub fn compile_command(
    compiler: &str,
    source_path: &Path,
    workdir: &Path,
    has_dependencies: bool,
) -> Command {
    let mut cmd = Command::new(compiler);

    // C++17 standard (minimum)
    cmd.arg("-std=c++17");

    // Per-session clang module cache: concurrent sessions must not contend on
    // the shared user cache (a killed compiler can leave it locked).
    cmd.env(
        "CLANG_MODULE_CACHE_PATH",
        workdir.join("clang-module-cache"),
    );

    // Enable all warnings
    cmd.arg("-Wall");
    cmd.arg("-Wextra");

    // Output path
    let binary_path = workdir.join("main");
    cmd.arg("-o").arg(&binary_path);

    // Source file
    cmd.arg(source_path);

    // Conan integration
    if has_dependencies {
        // Add Conan include and library paths
        let conan_profile_path = workdir.join("conan_profile");
        let profile_args = vec![
            format!("--profile:build={}", conan_profile_path.display()),
            format!("--profile:host={}", conan_profile_path.display()),
        ];

        for arg in profile_args {
            cmd.arg(&arg);
        }
    }

    // Optimization level (could be configurable)
    cmd.arg("-O2");

    cmd
}
