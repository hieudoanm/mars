use mars_runtime::inspect::{
    VariableInfo, cell_line_map, diagnostic_object, let_bindings, map_diagnostics, parse_variables,
    render_inspect_program,
};
use serde_json::json;
use std::path::Path;

#[test]
fn extracts_simple_let_bindings() {
    let cells = vec![
        "let numbers = vec![1, 2, 3];\nlet mut count = 0;".to_owned(),
        "let (a, b) = (1, 2);\nlet name: String = \"x\".into();".to_owned(),
    ];
    assert_eq!(let_bindings(&cells), vec!["numbers", "count", "name"]);
}

#[test]
fn let_bindings_handles_edge_cases() {
    // `let` must be preceded by a boundary and followed by a valid ident.
    let cells = vec![
        "outlet = 1;\nlet _x9 = 2;\nlet mut reusable = 3;\nlet reusable = 4;".to_owned(),
        "let 9bad = 1;\nlet _ = 5;\nlet mut = 6;".to_owned(),
        "fn helper() { let inner = 1; }".to_owned(),
    ];
    let names = let_bindings(&cells);
    assert!(names.contains(&"_x9".to_owned()));
    assert!(names.contains(&"reusable".to_owned()));
    assert!(names.contains(&"inner".to_owned()));
    assert!(
        !names
            .iter()
            .any(|name| name == "9bad" || name == "_" || name == "mut")
    );
    assert!(!names.iter().any(|name| name == "outlet"));
}

#[test]
fn renders_type_only_inspect_lines() {
    let program = render_inspect_program(&["let x = 1;".to_owned()], &["x".to_owned()], false);
    assert!(program.contains("std::any::type_name_of_val(&x)"));
    assert!(program.contains("__MARS_VAR__"));
}

#[test]
fn renders_value_inspect_lines_with_debug_preview() {
    let program = render_inspect_program(&["let x = 1;".to_owned()], &["x".to_owned()], true);
    assert!(program.contains("format!(\"{:?}\", &x)"));
    assert!(program.contains("take(160)"));
    assert!(program.contains("// Project Mars cell 0"));
}

#[test]
fn parse_variables_reads_marker_lines_only() {
    let stdout = concat!(
        "noise\n",
        "__MARS_VAR__\tanswer\ti32\t42\n",
        "__MARS_VAR__\tname\t&str\n",
        "__MARS_VAR__\tbroken\n",
        "__MARS_VAR__\t\t\n",
        "trailing\n",
    );
    let variables = parse_variables(stdout.as_bytes());
    assert_eq!(
        variables,
        vec![
            VariableInfo {
                name: "answer".to_owned(),
                type_name: Some("i32".to_owned()),
                value: Some("42".to_owned()),
            },
            VariableInfo {
                name: "name".to_owned(),
                type_name: Some("&str".to_owned()),
                value: None,
            },
            VariableInfo {
                name: "".to_owned(),
                type_name: Some("".to_owned()),
                value: None,
            },
        ]
    );
}

#[test]
fn cell_line_map_reads_index_before_hash() {
    let program = "fn main() {\n    // Project Mars cell 0 123456\n    let x = 1;\n    // Project Mars cell 2 654321\n}\n";
    assert_eq!(cell_line_map(program), vec![(2, 0), (4, 2)]);
}

#[test]
fn diagnostic_object_accepts_both_envelopes() {
    let flat = json!({ "$message_type": "diagnostic", "level": "error" });
    assert!(diagnostic_object(&flat).is_some());
    let wrapped = json!({
        "reason": "compiler-message",
        "message": { "level": "warning", "message": "careful" }
    });
    assert_eq!(
        diagnostic_object(&wrapped)
            .and_then(|d| d.get("message"))
            .and_then(|m| m.as_str()),
        Some("careful")
    );
    assert!(diagnostic_object(&json!({ "reason": "compiler-artifact" })).is_none());
}

#[test]
fn map_diagnostics_translates_rustc_spans_to_cells() {
    let program = concat!(
        "fn main() {\n",
        "    // Project Mars cell 0 111\n",
        "    let fine = 1;\n",
        "    // Project Mars cell 1 222\n",
        "    let broken = ;\n",
        "}\n",
    );
    let stderr = concat!(
        r#"{"reason":"compiler-message","message":{"message":"expected expression","level":"error","code":{"code":"E0425"},"spans":[{"file_name":"session.rs","line_start":5,"column_start":18,"line_end":5,"column_end":19,"is_primary":true,"label":"expected expression"}]}}"#,
        "\n",
        r#"{"reason":"compiler-message","message":{"message":"just a note","level":"note","spans":[{"file_name":"session.rs","line_start":5,"is_primary":true}]}}"#,
        "\n",
        r#"{"reason":"compiler-message","message":{"message":"secondary only","level":"warning","spans":[{"file_name":"session.rs","line_start":3,"is_primary":false}]}}"#,
        "\n",
        r#"{"reason":"compiler-message","message":{"message":"other file","level":"error","spans":[{"file_name":"lib.rs","line_start":9,"is_primary":true}]}}"#,
        "\n",
        "not json at all\n",
    );

    let diagnostics = map_diagnostics(program, stderr.as_bytes(), Path::new("session.rs"));
    assert_eq!(diagnostics.len(), 1);
    let diagnostic = &diagnostics[0];
    assert_eq!(diagnostic.cell_index, Some(1));
    assert_eq!(diagnostic.line, 1);
    assert_eq!(diagnostic.column, 18);
    assert_eq!(diagnostic.end_line, 1);
    assert_eq!(diagnostic.end_column, 19);
    assert_eq!(diagnostic.severity, "error");
    assert_eq!(diagnostic.code.as_deref(), Some("E0425"));
    assert_eq!(diagnostic.message, "expected expression");
}

#[test]
fn map_diagnostics_falls_back_for_spanless_errors() {
    let stderr = r#"{"$message_type":"diagnostic","level":"error","message":"aborting due to previous error","spans":[]}"#;
    let diagnostics = map_diagnostics("fn main() {}\n", stderr.as_bytes(), Path::new("session.rs"));
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].cell_index, None);
    assert_eq!(diagnostics[0].severity, "error");
}
