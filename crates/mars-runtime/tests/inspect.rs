use mars_runtime::inspect::{let_bindings, render_inspect_program};

#[test]
fn extracts_simple_let_bindings() {
    let cells = vec![
        "let numbers = vec![1, 2, 3];\nlet mut count = 0;".to_owned(),
        "let (a, b) = (1, 2);\nlet name: String = \"x\".into();".to_owned(),
    ];
    assert_eq!(let_bindings(&cells), vec!["numbers", "count", "name"]);
}

#[test]
fn renders_type_only_inspect_lines() {
    let program = render_inspect_program(&["let x = 1;".to_owned()], &["x".to_owned()], false);
    assert!(program.contains("std::any::type_name_of_val(&x)"));
    assert!(program.contains("__MARS_VAR__"));
}
