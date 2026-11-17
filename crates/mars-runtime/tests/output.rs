use mars_core::Output;
use mars_runtime::output::{
    annotate_source_locations, execution_outputs, extract_standalone_svg, has_visible_text,
    interrupted_result, normalize_stdout, parse_evcxr_content, strip_prompt_protocol,
};
use std::time::Duration;

#[test]
fn annotates_panic_locations_with_cell_hashes() {
    let program = "fn main() {\n    // Project Mars cell 0 111\n    let a = 1;\n    // Project Mars cell 1 222\n    panic!(\"boom\");\n}\n";
    let stderr = "thread 'main' panicked at src/main.rs:5:5:\nboom\n";
    let annotated = annotate_source_locations(stderr, program);
    // Line 5 is inside cell 1's region (marker at line 4, content starts line 5).
    assert!(annotated.contains("[mars:cell=222,line=1]"));
}

#[test]
fn leaves_unmapped_locations_untouched() {
    let program = "fn main() {\n    // Project Mars cell 0 111\n    let a = 1;\n}\n";
    let stderr = "note: run with RUST_BACKTRACE=1\n";
    assert_eq!(annotate_source_locations(stderr, program), stderr);
}

#[test]
fn extracts_table_marker_lines() {
    let stdout = b"before\n__MARS_TABLE__={\"columns\":[\"a\",\"b\"],\"rows\":[[\"1\",\"2\"],[\"3\",\"4\"]]}\nafter\n";
    let outputs = execution_outputs(stdout, b"", "RuntimeError");
    assert_eq!(outputs.len(), 2);
    match &outputs[1] {
        Output::Table { columns, rows } => {
            assert_eq!(columns, &vec!["a".to_owned(), "b".to_owned()]);
            assert_eq!(rows.len(), 2);
        }
        other => panic!("expected table, got {other:?}"),
    }
}

#[test]
fn extracts_display_marker_lines() {
    let stdout = concat!(
        "__MARS_HTML__\"<b>hi</b>\"\n",
        "__MARS_LATEX__\"x^2\"\n",
        "__MARS_IMAGE__={\"mime_type\":\"image/png\",\"data\":\"AQID\"}\n",
        "__MARS_CHART__{\"mark\":\"bar\"}\n"
    );
    let outputs = execution_outputs(stdout.as_bytes(), b"", "RuntimeError");
    assert_eq!(outputs.len(), 4);
    assert!(matches!(&outputs[0], Output::Html { html } if html == "<b>hi</b>"));
    assert!(matches!(&outputs[1], Output::Latex { latex } if latex == "x^2"));
    assert!(
        matches!(&outputs[2], Output::Image { mime_type, data } if mime_type == "image/png" && data == "AQID")
    );
    assert!(matches!(&outputs[3], Output::Chart { spec } if spec["mark"] == "bar"));
}

#[test]
fn ignores_malformed_table_marker() {
    let stdout = b"__MARS_TABLE__={not json}\n";
    let outputs = execution_outputs(stdout, b"", "RuntimeError");
    assert!(matches!(outputs[0], Output::Text { .. }));
}

#[test]
fn strips_prompt_protocol_lines() {
    let stripped = strip_prompt_protocol("a\n__MARS_PROMPT__Pick one\nb\n");
    assert_eq!(stripped, "a\n\nb\n");
    assert!(!stripped.contains("__MARS_PROMPT__"));
}

#[test]
fn normalize_stdout_removes_result_marker_prefix() {
    let normalized = normalize_stdout("__MARS_RESULT__=42\nplain\n");
    assert_eq!(normalized, "42\nplain\n");
}

#[test]
fn parse_evcxr_content_extracts_rich_blocks() {
    let stdout = concat!(
        "hello\n",
        "EVCXR_BEGIN_CONTENT text/plain\n",
        "plain output\n",
        "EVCXR_END_CONTENT\n",
        "EVCXR_BEGIN_CONTENT text/html\n",
        "<svg><path/></svg>\n",
        "EVCXR_END_CONTENT\n",
        "EVCXR_BEGIN_CONTENT application/x-custom\n",
        "custom\n",
        "EVCXR_END_CONTENT\n",
        "world\n"
    );
    let (text, outputs) = parse_evcxr_content(stdout);
    assert_eq!(text, "hello\nworld\n");
    assert_eq!(outputs.len(), 3);
    assert!(matches!(&outputs[0], Output::Text { text } if text == "plain output"));
    assert!(matches!(&outputs[1], Output::MimeBundle { data, .. }
            if data.get("image/svg+xml").and_then(|v| v.as_str()).is_some()
                && data.get("text/plain").and_then(|v| v.as_str()) == Some("Figure (SVG)")));
    assert!(matches!(&outputs[2], Output::MimeBundle { data, .. }
            if data.get("application/x-custom").and_then(|v| v.as_str()) == Some("custom")));
}

#[test]
fn standalone_svg_requires_no_surrounding_text() {
    assert_eq!(extract_standalone_svg("<svg></svg>"), Some("<svg></svg>"));
    assert_eq!(extract_standalone_svg("x<svg></svg>"), None);
    assert!(has_visible_text("<b>bold</b>"));
    assert!(!has_visible_text("<b></b>"));
}

#[test]
fn execution_outputs_combines_stdout_and_stderr() {
    let outputs = execution_outputs(b"out\n", b"err\n", "RuntimeError");
    assert_eq!(outputs.len(), 2);
    assert!(matches!(&outputs[0], Output::Text { text } if text == "out\n"));
    assert!(
        matches!(&outputs[1], Output::Error { name, message, traceback }
            if name == "RuntimeError" && message == "err\n" && traceback.is_empty())
    );
}

#[test]
fn interrupted_result_marks_execution_interrupted() {
    let result = interrupted_result(Duration::from_millis(5), 7, "cancelled");
    assert!(result.interrupted);
    assert_eq!(result.execution_count, 7);
    assert!(
        matches!(&result.outputs[0], Output::Error { name, message, .. }
        if name == "Interrupted" && message == "cancelled")
    );
}

#[test]
fn extracts_mime_marker_and_keeps_malformed_lines() {
    let stdout = concat!(
        "__MARS_MIME__={\"text/html\":\"<i>x</i>\",\"image/png\":\"aGk=\"}\n",
        "__MARS_MIME__=[1,2]\n",
        "__MARS_IMAGE__={\"mime_type\":\"image/png\"}\n",
        "__MARS_HTML__{bad json\n",
    );
    let outputs = execution_outputs(stdout.as_bytes(), b"", "RuntimeError");
    assert_eq!(outputs.len(), 2);
    match &outputs[0] {
        Output::Text { text } => {
            assert!(text.contains("__MARS_MIME__=[1,2]"));
            assert!(text.contains("__MARS_IMAGE__="));
            assert!(text.contains("__MARS_HTML__"));
        }
        other => panic!("expected text, got {other:?}"),
    }
    match &outputs[1] {
        Output::MimeBundle { data, .. } => {
            assert_eq!(data.len(), 2);
            assert_eq!(data["text/html"], serde_json::json!("<i>x</i>"));
        }
        other => panic!("expected mime bundle, got {other:?}"),
    }
}
