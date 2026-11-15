use mars_runtime::expression::{cell_source_hash, render_program};

#[test]
fn renders_only_the_current_cell_with_automatic_output() {
    let cells = vec![
        "let numbers = vec![1, 2, 3];".to_owned(),
        "numbers.len()".to_owned(),
    ];
    let program = render_program(&cells);

    assert_eq!(program.matches("__MARS_RESULT__").count(), 1);
    assert!(program.contains("println!(\"__MARS_RESULT__={:?}\", (numbers.len()));"));
}

#[test]
fn evaluates_replayed_final_expressions_without_displaying() {
    let cells = vec!["1 + 1".to_owned(), "2 + 2".to_owned()];
    let program = render_program(&cells);

    let hash = cell_source_hash("1 + 1");
    assert!(program.contains(&format!(
        "    // Project Mars cell 0 {hash}\n    (1 + 1);\n"
    )));
    assert!(program.contains("println!(\"__MARS_RESULT__={:?}\", (2 + 2));"));
    assert_eq!(program.matches("__MARS_RESULT__").count(), 1);
}

#[test]
fn renders_empty_cells_without_panicking() {
    assert!(render_program(&[]).ends_with("fn main() {\n}\n"));
}
