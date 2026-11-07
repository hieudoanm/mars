use mars::mcp::{run_tools, serve_mcp};
use serde_json::json;

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

fn server_arg() -> String {
    format!("{} mcp serve", env!("CARGO_BIN_EXE_mars"))
}

#[test]
fn tools_requires_a_command() {
    assert!(run_tools(&[]).is_err());
}

#[test]
fn tools_rejects_bad_server_argument() {
    assert!(run_tools(&args(&["list", "--server"])).is_err());
    assert!(run_tools(&args(&["list", "--server", ""])).is_err());
    assert!(run_tools(&args(&["list", "--server", "definitely-not-a-binary-xyz"])).is_err());
}

#[test]
fn tools_lists_tools_from_real_server() {
    run_tools(&args(&["list", "--server", &server_arg()])).expect("tools list");
}

#[test]
fn tools_rejects_extra_list_arguments() {
    assert!(run_tools(&args(&["list", "unexpected", "--server", &server_arg()])).is_err());
}

#[test]
fn tools_call_reports_tool_errors() {
    // mars_notebook_open on a missing file returns a CallToolResult error,
    // which run_tools surfaces as a command error.
    let result = run_tools(&args(&[
        "call",
        "mars_notebook_open",
        r#"{"path":"/nonexistent/mars-cli-test.mars"}"#,
        "--server",
        &server_arg(),
    ]));
    assert!(result.is_err());
}

#[test]
fn tools_call_executes_and_prints_result() {
    let dir = std::env::temp_dir().join(format!("mars-cli-tools-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("tools.mars");
    mars_document::save_atomic(&mars_document::Notebook::new(), &path).expect("save");
    let path_string = path.to_string_lossy().to_string();

    let result = run_tools(&args(&[
        "call",
        "mars_cell_execute",
        &json!({
            "path": path_string,
            "source": "6 * 7",
            "language": "rust"
        })
        .to_string(),
        "--server",
        &server_arg(),
    ]));
    assert!(result.is_ok(), "cell execute failed: {result:?}");

    let missing_name = run_tools(&args(&["call", "--server", &server_arg()]));
    assert!(missing_name.is_err());

    let invalid_json = run_tools(&args(&[
        "call",
        "mars_kernel_status",
        "not-json",
        "--server",
        &server_arg(),
    ]));
    assert!(invalid_json.is_err());

    let extra_args = run_tools(&args(&[
        "call",
        "mars_kernel_status",
        "{}",
        "extra",
        "--server",
        &server_arg(),
    ]));
    assert!(extra_args.is_err());

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn serve_mcp_rejects_missing_notebook() {
    assert!(serve_mcp(&args(&["--notebook", "/nonexistent/mars-cli-mcp.mars"])).is_err());
}

#[test]
fn tools_rejects_unknown_command_after_connect() {
    assert!(run_tools(&args(&["bogus", "--server", &server_arg()])).is_err());
}

#[test]
fn serve_mcp_serves_until_stdin_eof() {
    // With no `--notebook`, serve_mcp enters the stdio loop; test stdin is
    // EOF so it should return promptly.
    serve_mcp(&[]).expect("serve until EOF");
}
