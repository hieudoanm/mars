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
