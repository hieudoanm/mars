use mars_runtime_cpp::{generate_conanfile, parse_compiler_diagnostics};
use std::collections::HashMap;

#[test]
fn test_generate_conanfile() {
    let mut deps = HashMap::new();
    deps.insert("nlohmann_json".to_string(), "3.11.2".to_string());
    deps.insert("eigen".to_string(), "3.4.0".to_string());

    let conanfile = generate_conanfile(&deps);

    assert!(conanfile.contains("[requires]"));
    assert!(conanfile.contains("nlohmann_json/3.11.2"));
    assert!(conanfile.contains("eigen/3.4.0"));
    assert!(conanfile.contains("[generators]"));
    assert!(conanfile.contains("CMakeDeps"));
}

#[test]
fn test_empty_dependencies() {
    let deps = HashMap::new();
    let conanfile = generate_conanfile(&deps);

    assert!(conanfile.contains("[requires]"));
    assert!(conanfile.contains("[generators]"));
}

#[test]
fn test_parse_clang_error() {
    let stderr = "main.cpp:10:5: error: use of undeclared identifier 'x'";
    let result = parse_compiler_diagnostics(stderr);
    assert_eq!(result.len(), 1);
    assert!(result[0].contains("error"));
}

#[test]
fn test_parse_gcc_error() {
    let stderr = "main.cpp:10:5: error: 'x' was not declared in this scope";
    let result = parse_compiler_diagnostics(stderr);
    assert_eq!(result.len(), 1);
    assert!(result[0].contains("error"));
}

#[test]
fn test_parse_warnings() {
    let stderr = "main.cpp:10:5: warning: unused variable 'x'";
    let result = parse_compiler_diagnostics(stderr);
    assert_eq!(result.len(), 1);
    assert!(result[0].contains("warning"));
}

#[test]
fn test_empty_stderr() {
    let stderr = "";
    let result = parse_compiler_diagnostics(stderr);
    assert!(result.is_empty());
}

#[test]
fn test_raw_stderr() {
    let stderr = "Some unstructured error message";
    let result = parse_compiler_diagnostics(stderr);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], stderr);
}

#[test]
fn test_render_program_accumulates_and_dedups_includes() {
    let cells = vec![
        "#include <iostream>\n#include <numeric>\nint x = 1;".to_string(),
        "#include <numeric>\n#include <set>\nstd::cout << x;".to_string(),
    ];
    let program = mars_runtime_cpp::render_program(&cells);
    assert!(program.contains("#include <iostream>"));
    assert!(program.contains("#include <numeric>"));
    assert!(program.contains("#include <set>"));
    // Duplicated include appears once in the header block (plus the std set).
    assert_eq!(program.matches("#include <numeric>").count(), 1);
    assert!(program.contains("int main()"));
    assert!(program.contains("return 0;"));
}

#[test]
fn test_render_program_empty_cells() {
    let program = mars_runtime_cpp::render_program(&[]);
    assert!(program.contains("int main()"));
    assert!(!program.contains("#include <numeric>"));
}

#[test]
fn test_compile_command_without_dependencies() {
    let dir = std::env::temp_dir().join("mars-cpp-cmd-test");
    let cmd = mars_runtime_cpp::compile_command("clang++", &dir.join("main.cpp"), &dir, false);
    let args: Vec<String> = cmd
        .get_args()
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    assert!(args.iter().any(|a| a == "-std=c++17"));
    assert!(args.iter().any(|a| a == "-O2"));
    assert!(!args.iter().any(|a| a.contains("profile:build")));
}

#[test]
fn test_compile_command_with_dependencies_adds_conan_profiles() {
    let dir = std::env::temp_dir().join("mars-cpp-cmd-test");
    let cmd = mars_runtime_cpp::compile_command("clang++", &dir.join("main.cpp"), &dir, true);
    let args: Vec<String> = cmd
        .get_args()
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    assert!(args.iter().any(|a| a.contains("--profile:build=")));
    assert!(args.iter().any(|a| a.contains("--profile:host=")));
    assert!(args.iter().any(|a| a.contains("conan_profile")));
}
