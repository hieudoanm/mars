use mars_lsp::read_message;
use mars_lsp::session::{
    CellSource, build_document, decode_semantic_tokens, diagnostic_covers, diagnostic_range,
    first_location,
};
use serde_json::json;
use std::io::Cursor;

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

#[test]
fn first_location_handles_all_result_shapes() {
    // Single Location.
    let location = json!({
        "uri": "file:///a.rs",
        "range": {"start": {"line": 1, "character": 2}, "end": {"line": 1, "character": 5}}
    });
    assert_eq!(
        first_location(&location),
        Some(("file:///a.rs".to_owned(), (1, 2)))
    );

    // Array of Locations picks the first valid one.
    let array = json!([
        {"unexpected": true},
        {"uri": "file:///b.rs", "range": {"start": {"line": 3, "character": 4}, "end": {"line": 3, "character": 6}}}
    ]);
    assert_eq!(
        first_location(&array),
        Some(("file:///b.rs".to_owned(), (3, 4)))
    );

    // LocationLink uses targetUri + targetSelectionRange/targetRange.
    let link = json!({
        "targetUri": "file:///c.rs",
        "targetRange": {"start": {"line": 7, "character": 8}, "end": {"line": 7, "character": 9}},
        "targetSelectionRange": {"start": {"line": 9, "character": 1}, "end": {"line": 9, "character": 2}}
    });
    assert_eq!(
        first_location(&link),
        Some(("file:///c.rs".to_owned(), (9, 1)))
    );

    // Malformed shapes return None.
    assert_eq!(first_location(&json!({"uri": "file:///x"})), None);
    assert_eq!(first_location(&json!(null)), None);
    assert_eq!(first_location(&json!([])), None);
}

#[test]
fn diagnostic_range_and_covers() {
    let diagnostic = json!({
        "range": {
            "start": {"line": 2, "character": 3},
            "end": {"line": 4, "character": 5}
        }
    });
    assert_eq!(diagnostic_range(&diagnostic), Some(((2, 3), (4, 5))));
    assert!(diagnostic_covers(&diagnostic, 2, 3));
    assert!(diagnostic_covers(&diagnostic, 3, 0));
    assert!(diagnostic_covers(&diagnostic, 4, 5));
    assert!(!diagnostic_covers(&diagnostic, 2, 2));
    assert!(!diagnostic_covers(&diagnostic, 5, 0));
    assert!(!diagnostic_covers(&json!({}), 0, 0));
    assert!(!diagnostic_covers(
        &json!({"range": {"start": {"line": "x"}}}),
        0,
        0
    ));
}

#[test]
fn read_message_parses_content_length_framing() {
    let body = json!({"jsonrpc": "2.0", "id": 1, "result": {"ok": true}}).to_string();
    let framed = format!("Content-Length: {}\r\n\r\n{}", body.len(), body);
    let mut reader = Cursor::new(framed.into_bytes());
    let message = read_message(&mut reader).expect("read").expect("message");
    assert_eq!(message["result"]["ok"], true);
}

#[test]
fn read_message_handles_eof_and_bad_frames() {
    // EOF before headers -> None.
    let mut empty = Cursor::new(Vec::<u8>::new());
    assert!(read_message(&mut empty).expect("read").is_none());

    // Blank line only -> None (no content length).
    let mut blank = Cursor::new(b"\r\n".to_vec());
    assert!(read_message(&mut blank).expect("read").is_none());

    // Header without body -> read error surfaces as Err or None depending
    // on truncation; assert it does not panic.
    let mut truncated = Cursor::new(b"Content-Length: 50\r\n\r\n{\"a\":".to_vec());
    let _ = read_message(&mut truncated);

    // Invalid JSON body -> Err.
    let bad = b"not json".to_vec();
    let framed = format!("Content-Length: {}\r\n\r\n", bad.len());
    let mut combined = framed.into_bytes();
    combined.extend_from_slice(&bad);
    let mut reader = Cursor::new(combined);
    assert!(read_message(&mut reader).is_err());
}

#[test]
fn build_document_skips_non_code_cells_and_terminates_expressions() {
    let cells = vec![
        cell("m1", "markdown", "# Title"),
        cell("c1", "code", "let x = 1;"),
        cell("r1", "raw", "raw text"),
        cell("c2", "code", "x + 1"),
        cell("c3", "code", ""),
    ];
    let (document, spans) = build_document(&cells);
    assert!(document.starts_with("fn main() {\n"));
    assert!(document.ends_with("}\n"));
    assert!(!document.contains("# Title"));
    assert!(!document.contains("raw text"));
    // All three code cells produce spans; markdown/raw are skipped.
    assert_eq!(spans.len(), 3);
    assert_eq!(spans[0].id, "c1");
    assert_eq!(spans[1].id, "c2");
    assert_eq!(spans[2].id, "c3");
    // The trailing bare expression got a `;` appended.
    assert!(document.contains("x + 1;"));
}

#[test]
fn diagnostic_range_rejects_malformed_values() {
    use mars_lsp::session::diagnostic_range;
    assert!(diagnostic_range(&serde_json::json!({})).is_none());
    assert!(diagnostic_range(&serde_json::json!({"range": {}})).is_none());
    assert!(
        diagnostic_range(&serde_json::json!({
            "range": {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 1}}
        }))
        .is_some()
    );
}
