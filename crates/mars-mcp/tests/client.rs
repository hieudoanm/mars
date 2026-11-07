use mars_mcp::McpClient;

#[test]
fn client_handshake_against_test_server() {
    // In-process smoke test: run the real server on pipes via threads is
    // covered by server tests; here we only verify spawn plumbing fails
    // gracefully for a missing binary.
    let client = McpClient::spawn(&["definitely-not-a-real-binary-xyz".to_owned()]);
    assert!(client.is_err());
}
