use mars::output::{print_execution_result, print_indented, print_indented_error};
use mars_core::Output;
use mars_runtime::ExecutionResult;
use serde_json::json;
use std::time::Duration;

fn result(outputs: Vec<Output>, interrupted: bool) -> ExecutionResult {
    ExecutionResult {
        outputs,
        duration: Duration::from_millis(12),
        execution_count: 3,
        interrupted,
    }
}

#[test]
fn prints_successful_text_output() {
    print_execution_result(
        0,
        &result(
            vec![Output::Text {
                text: "hello\nworld\n".to_owned(),
            }],
            false,
        ),
    );
}

#[test]
fn prints_failed_and_interrupted_results() {
    print_execution_result(
        1,
        &result(
            vec![Output::Error {
                name: "RuntimeError".to_owned(),
                message: "boom\ntrace".to_owned(),
                traceback: vec!["line".to_owned()],
            }],
            false,
        ),
    );
    print_execution_result(2, &result(Vec::new(), true));
}

#[test]
fn prints_every_output_variant() {
    let mut bundle = serde_json::Map::new();
    bundle.insert("image/svg+xml".to_owned(), json!("<svg></svg>"));
    bundle.insert("text/plain".to_owned(), json!("fallback"));
    let mut plain_bundle = serde_json::Map::new();
    plain_bundle.insert("application/json".to_owned(), json!("{\"a\":1}"));
    plain_bundle.insert("text/csv".to_owned(), json!("a,b"));

    print_execution_result(
        0,
        &result(
            vec![
                Output::Html {
                    html: "<b>hi</b>".to_owned(),
                },
                Output::Image {
                    mime_type: "image/png".to_owned(),
                    data: "aGVsbG8=".to_owned(),
                },
                Output::Image {
                    mime_type: "image/svg+xml".to_owned(),
                    data: "<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>".to_owned(),
                },
                Output::Table {
                    columns: vec!["a".to_owned(), "b".to_owned()],
                    rows: vec![vec!["1".to_owned(), "2".to_owned()]],
                },
                Output::Latex {
                    latex: "x^2".to_owned(),
                },
                Output::Chart {
                    spec: json!({"mark": "bar"}),
                },
                Output::MimeBundle {
                    data: bundle,
                    metadata: serde_json::Map::new(),
                },
                Output::MimeBundle {
                    data: plain_bundle,
                    metadata: serde_json::Map::new(),
                },
            ],
            false,
        ),
    );
}

#[test]
fn formats_large_base64_images_as_kilobytes() {
    let large = "A".repeat(4096);
    print_execution_result(
        0,
        &result(
            vec![Output::Image {
                mime_type: "image/png".to_owned(),
                data: large,
            }],
            false,
        ),
    );
}

#[test]
fn indented_helpers_emit_lines() {
    print_indented("one\ntwo\n");
    print_indented_error("err1\nerr2\n");
    print_indented("");
    print_indented_error("");
}
