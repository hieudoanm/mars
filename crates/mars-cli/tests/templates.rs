use mars::changes::first_changed_code_index;
use mars::templates::{list_templates_text, notebook_from_template};
use mars_core::CellType;
use mars_document::{Cell, Notebook};

fn notebook_with_cells(cells: &[(&str, &str)]) -> Notebook {
    let mut notebook = Notebook::new();
    for (id, source) in cells {
        let mut cell = Cell::new(CellType::Code, *source);
        cell.id = mars_core::CellId::from_string(*id).expect("cell id");
        notebook.add_cell(cell).expect("add cell");
    }
    notebook
}

#[test]
fn builtin_templates_produce_valid_notebooks() {
    for name in ["rust", "python", "ml", "dashboard"] {
        let notebook = notebook_from_template(name).expect("template");
        assert!(
            notebook
                .cells
                .iter()
                .any(|cell| cell.cell_type == CellType::Code),
            "{name} should contain code cells"
        );
        assert_eq!(
            notebook.metadata.language,
            match name {
                "python" => "python",
                _ => "rust",
            }
        );
    }
}

#[test]
fn unknown_template_is_an_error() {
    let error = notebook_from_template("nope").expect_err("unknown template");
    assert!(error.contains("unknown template"));
}

#[test]
fn templates_listing_mentions_all_builtins() {
    let text = list_templates_text();
    for name in ["rust", "python", "ml", "dashboard"] {
        assert!(text.contains(name), "missing {name}");
    }
}

#[test]
fn first_changed_code_index_detects_new_and_modified_cells() {
    let baseline = notebook_with_cells(&[("a", "1 + 1"), ("b", "2 + 2"), ("c", "3 + 3")]);

    // Identical -> nothing changed.
    let same = notebook_with_cells(&[("a", "1 + 1"), ("b", "2 + 2"), ("c", "3 + 3")]);
    assert_eq!(first_changed_code_index(&same, &baseline), None);

    // Modified middle cell -> its index.
    let modified = notebook_with_cells(&[("a", "1 + 1"), ("b", "2 + 3"), ("c", "3 + 3")]);
    assert_eq!(first_changed_code_index(&modified, &baseline), Some(1));

    // New cell appended -> its index.
    let appended = notebook_with_cells(&[
        ("a", "1 + 1"),
        ("b", "2 + 2"),
        ("c", "3 + 3"),
        ("d", "4 + 4"),
    ]);
    assert_eq!(first_changed_code_index(&appended, &baseline), Some(3));

    // Deleted cell: remaining cells keep their ids and sources, so no
    // *current* code cell differs -> None. Documented limitation: deletions
    // alone do not trigger re-execution.
    let deleted = notebook_with_cells(&[("a", "1 + 1"), ("c", "3 + 3")]);
    assert_eq!(first_changed_code_index(&deleted, &baseline), None);
}

#[test]
fn diff_cells_classifies_changes() {
    use mars::changes::{CellChange, diff_cells, source_diff};
    let baseline = notebook_with_cells(&[("a", "one"), ("b", "two")]);
    let current = notebook_with_cells(&[("a", "one"), ("b", "two!"), ("c", "three")]);
    let diff = diff_cells(&current, &baseline);
    let kinds: Vec<CellChange> = diff.iter().map(|(change, _)| *change).collect();
    assert_eq!(
        kinds,
        vec![
            CellChange::Unchanged,
            CellChange::Modified,
            CellChange::Added,
        ]
    );
    // dissimilar diffs at character granularity; assert markers, not exact lines.
    let rendered = source_diff("alpha\nbeta\n", "alpha\ngamma\n");
    assert!(rendered.contains('-'));
    assert!(rendered.contains('+'));
}
