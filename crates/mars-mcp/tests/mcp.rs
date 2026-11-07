use mars_mcp::mcp::CallToolResult;

#[test]
fn tool_result_serializes_mcp_shape() {
    let result = CallToolResult::error("boom");
    let value = serde_json::to_value(&result).expect("serialize");
    assert_eq!(value["isError"], true);
    assert_eq!(value["content"][0]["type"], "text");
    assert_eq!(value["content"][0]["text"], "boom");
}
