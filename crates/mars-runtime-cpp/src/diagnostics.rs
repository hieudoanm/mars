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
