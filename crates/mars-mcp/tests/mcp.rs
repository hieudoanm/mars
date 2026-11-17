use mars_mcp::mcp::{CallToolResult, PROTOCOL_VERSION, SERVER_NAME, initialize_result};
use mars_mcp::tools;
use serde_json::json;

#[test]
fn tool_result_serializes_mcp_shape() {
    let result = CallToolResult::error("boom");
    let value = serde_json::to_value(&result).expect("serialize");
    assert_eq!(value["isError"], true);
    assert_eq!(value["content"][0]["type"], "text");
    assert_eq!(value["content"][0]["text"], "boom");
}

#[test]
fn tool_result_ok_omits_is_error() {
    let result = CallToolResult::ok("done");
    let value = serde_json::to_value(&result).expect("serialize");
    assert!(value.get("isError").is_none());
    assert_eq!(value["content"][0]["text"], "done");
}

#[test]
fn tool_result_ok_json_pretty_prints() {
    let result = CallToolResult::ok_json(&json!({ "a": 1 }));
    let text = &result.content[0].text;
    assert!(text.contains("\n"), "expected pretty JSON: {text}");
    let parsed: serde_json::Value = serde_json::from_str(text).expect("json");
    assert_eq!(parsed["a"], 1);
}

#[test]
fn initialize_result_advertises_tools_capability() {
    let result = initialize_result();
    assert_eq!(result["protocolVersion"], PROTOCOL_VERSION);
    assert_eq!(result["serverInfo"]["name"], SERVER_NAME);
    assert_eq!(result["capabilities"]["tools"]["listChanged"], false);
}

#[test]
fn tool_definitions_use_camel_case_schema_key() {
    for tool in tools::definitions() {
        let value = serde_json::to_value(&tool).expect("serialize tool");
        assert!(value.get("inputSchema").is_some(), "{}", tool.name);
        assert!(value.get("input_schema").is_none());
    }
}

const EXPECTED_TOOLS: &[&str] = &[
    "mars_notebook_new",
    "mars_notebook_open",
    "mars_notebook_read",
    "mars_notebook_save",
    "mars_notebook_inspect",
    "mars_cell_add",
    "mars_cell_update",
    "mars_cell_delete",
    "mars_cell_move",
    "mars_cell_execute",
    "mars_notebook_execute",
    "mars_cell_diagnose",
    "mars_kernel_status",
    "mars_kernel_reset",
    "mars_kernel_variables",
    "mars_lab_algorithms",
    "mars_lab_dataset_upload",
    "mars_lab_datasets",
    "mars_lab_dataset",
    "mars_lab_train",
    "mars_lab_jobs",
    "mars_lab_job",
    "mars_lab_export",
];

#[test]
fn tool_definitions_cover_expected_catalog() {
    let definitions = tools::definitions();
    let mut names: Vec<&str> = definitions.iter().map(|tool| tool.name.as_str()).collect();
    names.sort_unstable();
    let mut expected = EXPECTED_TOOLS.to_vec();
    expected.sort_unstable();
    assert_eq!(names, expected);
}

#[test]
fn tool_definitions_serialize_all_four_annotation_hints() {
    for tool in tools::definitions() {
        let value = serde_json::to_value(&tool).expect("serialize tool");
        let annotations = &value["annotations"];
        for hint in [
            "readOnlyHint",
            "destructiveHint",
            "idempotentHint",
            "openWorldHint",
        ] {
            assert!(
                annotations.get(hint).is_some_and(|v| v.is_boolean()),
                "{} missing boolean annotation {hint}: {annotations}",
                tool.name
            );
        }
        // Every mars tool operates on local notebooks, kernels, and lab data.
        assert_eq!(annotations["openWorldHint"], false, "{}", tool.name);
    }
}
