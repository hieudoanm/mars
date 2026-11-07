use mars_lsp::session::{CellSource, build_document, decode_semantic_tokens};

fn cell(id: &str, cell_type: &str, source: &str) -> CellSource {
    CellSource {
        id: id.to_owned(),
        cell_type: cell_type.to_owned(),
        source: source.to_owned(),
    }
}

#[test]
fn document_wraps_code_cells_and_maps_lines() {
    let cells = vec![
        cell("m1", "markdown", "# Title"),
        cell("c1", "code", "let x = 1;"),
        cell("c2", "code", "let y = 2;\nprintln!(\"{y}\");"),
    ];
    let (document, spans) = build_document(&cells);
    assert!(document.starts_with("fn main() {\n"));
    assert!(document.ends_with("}\n"));
    assert!(!document.contains("# Title"));
    assert_eq!(spans.len(), 2);
    assert_eq!(spans[0].id, "c1");
    assert_eq!(spans[0].start_line, 1);
    assert_eq!(spans[1].start_line, 3);
    assert_eq!(spans[1].line_count, 2);
}

#[test]
fn semantic_token_delta_decoding() {
    // Two tokens on line 0, then a token on line 2 (deltaLine=2 resets
    // the character offset).
    let data = [
        0, 0, 2, 14, 0, // fn at char 0
        0, 3, 4, 12, 0, // main at char 3
        2, 5, 3, 9, 0, // let at line 2 char 5
    ];
    let tokens = decode_semantic_tokens(&data);
    assert_eq!(tokens, vec![(0, 0, 2, 14), (0, 3, 4, 12), (2, 5, 3, 9)]);
}

#[test]
fn semantic_token_decode_rejects_bad_length() {
    assert!(decode_semantic_tokens(&[1, 2, 3]).is_empty());
}
