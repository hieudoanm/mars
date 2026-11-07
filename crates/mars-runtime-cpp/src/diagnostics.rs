/// Parses compiler error/warning messages and maps them to notebook cells.
pub fn parse_compiler_diagnostics(stderr: &str) -> Vec<String> {
    let mut diagnostics = Vec::new();

    for line in stderr.lines() {
        // Parse clang++/g++ error format: file:line:column: error: message
        if line.contains("error:") || line.contains("warning:") {
            diagnostics.push(line.to_string());
        }
    }

    // If no structured errors found, return the raw stderr
    if diagnostics.is_empty() && !stderr.is_empty() {
        diagnostics.push(stderr.to_string());
    }

    diagnostics
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
