use mars_core::Output;
use mars_runtime::{
    CancellationToken, RuntimeError, RustSession, cargo_manifest, format_compiler_diagnostics,
    parse_evcxr_content,
};
use std::collections::BTreeMap;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::Duration;

#[test]
fn preserves_bindings_between_cells() {
    let mut session = RustSession::new().expect("runtime directory");
    session
        .execute(r#"let numbers = vec![1, 2, 3];"#)
        .expect("first cell");
    let result = session
        .execute(r#"println!("{}", numbers.iter().sum::<i32>());"#)
        .expect("second cell");

    assert_eq!(result.execution_count, 2);
    assert_eq!(
        result.outputs,
        vec![Output::Text {
            text: "6\n".to_owned()
        }]
    );
}

#[test]
fn captures_a_final_expression_result() {
    let mut session = RustSession::new().expect("runtime directory");
    let result = session.execute("let numbers = vec![1, 2, 3];\nnumbers.len()");

    assert_eq!(
        result.expect("execution result").outputs,
        vec![Output::Text {
            text: "3\n".to_owned()
        }]
    );
}

#[test]
fn captures_multiline_expression_results() {
    let mut session = RustSession::new().expect("runtime directory");
    let result = session
        .execute("let values = vec![1, 2, 3];\nvalues\n    .iter()\n    .sum::<i32>()")
        .expect("execution result");

    assert_eq!(
        result.outputs,
        vec![Output::Text {
            text: "6\n".to_owned()
        }]
    );
}

#[test]
fn ignores_trailing_comments_when_extracting_results() {
    let mut session = RustSession::new().expect("runtime directory");
    let result = session
        .execute("let answer = 42;\nanswer // show the result")
        .expect("execution result");

    assert_eq!(
        result.outputs,
        vec![Output::Text {
            text: "42\n".to_owned()
        }]
    );
}

#[test]
fn falls_back_to_display_for_non_debug_values() {
    let mut session = RustSession::new().expect("runtime directory");
    session
        .execute("vec![1, 2, 3]")
        .expect("debug-only prior value");
    let result = session
        .execute(
            r#"
struct DisplayOnly;
impl std::fmt::Display for DisplayOnly {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "display-only")
    }
}
DisplayOnly
"#,
        )
        .expect("execution result");

    assert_eq!(
        result.outputs,
        vec![Output::Text {
            text: "display-only\n".to_owned()
        }]
    );
}

#[test]
fn preserves_explicit_output_without_duplicate_expression_output() {
    let mut session = RustSession::new().expect("runtime directory");
    let result = session
        .execute(r#"println!("hello");"#)
        .expect("execution result");

    assert_eq!(
        result.outputs,
        vec![Output::Text {
            text: "hello\n".to_owned()
        }]
    );
}

#[test]
fn does_not_commit_failed_cells() {
    let mut session = RustSession::new().expect("runtime directory");
    let result = session.execute("let value: = 1;");

    assert_eq!(session.execution_count(), 0);
    assert!(matches!(
        result.expect("compiler result").outputs.first(),
        Some(Output::Error { name, .. }) if name == "CompilationError"
    ));
}

#[test]
fn formats_structured_diagnostics_with_all_labeled_spans() {
    let stderr = br#"{"reason":"compiler-message","message":{"message":"mismatched types","level":"error","code":{"code":"E0308"},"spans":[{"file_name":"session.rs","line_start":2,"column_start":5,"line_end":2,"column_end":8,"is_primary":true,"label":"expected integer"},{"file_name":"session.rs","line_start":3,"column_start":1,"line_end":3,"column_end":4,"is_primary":false,"label":"found here"}]}}"#;
    let formatted = format_compiler_diagnostics(stderr);

    assert!(formatted.contains("error [E0308]: mismatched types"));
    assert!(formatted.contains("[primary] expected integer"));
    assert!(formatted.contains("[secondary] found here"));
}

#[test]
fn interrupts_long_running_cells() {
    let mut session = RustSession::new().expect("runtime directory");
    let result = session
        .execute_with_timeout(
            "std::thread::sleep(std::time::Duration::from_millis(100));",
            Some(Duration::from_millis(10)),
        )
        .expect("execution result");

    assert!(result.interrupted);
    assert_eq!(session.execution_count(), 0);
    assert!(matches!(
        result.outputs.first(),
        Some(Output::Error { name, .. }) if name == "Interrupted"
    ));
}

#[test]
fn cancels_long_running_cells() {
    let mut session = RustSession::new().expect("runtime directory");
    let cancellation = Arc::new(AtomicBool::new(false));
    let signal = Arc::clone(&cancellation);
    let worker = thread::spawn(move || {
        session
            .execute_with_control(
                "std::thread::sleep(std::time::Duration::from_millis(500));",
                None,
                cancellation,
            )
            .expect("execution result")
    });

    thread::sleep(Duration::from_millis(25));
    signal.store(true, Ordering::Relaxed);
    let result = worker.join().expect("worker completes");

    assert!(result.interrupted);
    assert!(result.outputs.iter().any(|output| {
        matches!(
            output,
            Output::Error { name, message, .. }
                if name == "Interrupted" && message.contains("cancelled")
        )
    }));
}

#[test]
fn cancellation_token_can_cancel_before_start() {
    let token = CancellationToken::new();
    token.cancel();
    let mut session = RustSession::new().expect("runtime directory");
    let result = session
        .execute_with_control("println!(\"not run\");", None, token.flag())
        .expect("execution result");

    assert!(result.interrupted);
    assert!(result.outputs.iter().any(|output| {
        matches!(output, Output::Error { message, .. } if message.contains("before compilation"))
    }));
}

#[test]
fn replay_repeats_prior_side_effects_by_design() {
    let mut session = RustSession::new().expect("runtime directory");
    session
        .execute(r#"println!("side effect"); let value = 7;"#)
        .expect("first cell");
    let result = session.execute("value").expect("second cell");

    assert_eq!(
        result.outputs,
        vec![Output::Text {
            text: "side effect\n7\n".to_owned()
        }]
    );
}

#[test]
fn writes_a_cargo_manifest_for_declared_dependencies() {
    let mut dependencies = BTreeMap::new();
    dependencies.insert("serde".to_owned(), "1".to_owned());
    let session = RustSession::with_dependencies(dependencies).expect("runtime directory");
    let manifest = cargo_manifest(&session.dependencies);

    assert!(manifest.contains("[dependencies]"));
    assert!(manifest.contains("serde = \"1\""));
}

#[test]
fn writes_richer_cargo_dependency_specs() {
    let mut dependencies = BTreeMap::new();
    dependencies.insert(
        "serde".to_owned(),
        "{ version = \"1\", features = [\"derive\"] }".to_owned(),
    );
    assert!(
        cargo_manifest(&dependencies)
            .contains("serde = { version = \"1\", features = [\"derive\"] }")
    );
}

#[test]
fn rejects_invalid_dependency_names() {
    let mut dependencies = BTreeMap::new();
    dependencies.insert("bad name".to_owned(), "1".to_owned());
    assert!(matches!(
        RustSession::with_dependencies(dependencies),
        Err(RuntimeError::InvalidDependency { .. })
    ));
}

#[test]
fn parses_evcxr_svg_content_from_stdout() {
    let stdout = concat!(
        "before\n",
        "EVCXR_BEGIN_CONTENT text/html\n",
        "<div style=\"\"><svg xmlns=\"http://www.w3.org/2000/svg\"><circle cx=\"4\" cy=\"4\" r=\"4\"/></svg></div>\n",
        "EVCXR_END_CONTENT\n",
        "after\n",
    );

    let (text, outputs) = parse_evcxr_content(stdout);

    assert_eq!(text, "before\nafter\n");
    assert_eq!(outputs.len(), 1);
    let Output::MimeBundle { data, .. } = &outputs[0] else {
        panic!("expected a MIME bundle");
    };
    assert!(
        data["image/svg+xml"]
            .as_str()
            .is_some_and(|svg| svg.starts_with("<svg"))
    );
    assert!(!data.contains_key("text/html"));
    assert_eq!(data["text/plain"], "Figure (SVG)");
}

#[test]
fn keeps_evcxr_html_that_is_not_only_an_svg() {
    let stdout = concat!(
        "EVCXR_BEGIN_CONTENT text/html\n",
        "<p>Before</p><svg><circle/></svg>\n",
        "EVCXR_END_CONTENT\n",
    );

    let (text, outputs) = parse_evcxr_content(stdout);

    assert_eq!(text, "");
    assert!(matches!(
        outputs.as_slice(),
        [Output::Html { html }] if html.contains("Before")
    ));
}

#[test]
fn leaves_plain_stdout_untouched() {
    let (text, outputs) = parse_evcxr_content("hello\n");

    assert_eq!(text, "hello\n");
    assert!(outputs.is_empty());
}

#[test]
fn display_helpers_emit_typed_outputs_end_to_end() {
    let mut session = RustSession::new().expect("runtime directory");
    let result = session
        .execute(
            r##"
mars_display_html("<b>hi</b>");
mars_display_latex("x^2");
mars_display_image("image/png", &[1, 2, 3]);
mars_chart(r#"{"mark":"bar","data":{"values":[{"x":"a","y":1}]},"encoding":{"x":{"field":"x","type":"nominal"},"y":{"field":"y","type":"quantitative"}}}"#);
mars_table(&["name", "score"], &[vec!["alice".to_string(), "10".to_string()]]);
mars_csv("city,pop\nOslo,700000\n");
mars_display_markdown("**bold** text");
mars_display_mime("application/json", "{\"a\":1}");
"##,
        )
        .expect("execution result");

    let kinds: Vec<&str> = result
        .outputs
        .iter()
        .map(|output| match output {
            Output::Html { .. } => "html",
            Output::Latex { .. } => "latex",
            Output::Image { .. } => "image",
            Output::Chart { .. } => "chart",
            Output::Table { .. } => "table",
            _ => "other",
        })
        .collect();
    assert_eq!(
        kinds,
        [
            "html", "latex", "image", "chart", "table", "table", "other", "other"
        ]
    );
    assert!(
        matches!(&result.outputs[6], Output::MimeBundle { data, .. } if data["text/markdown"] == "**bold** text")
    );
    assert!(
        matches!(&result.outputs[7], Output::MimeBundle { data, .. } if data["application/json"] == "{\"a\":1}")
    );
    assert!(
        matches!(&result.outputs[2], Output::Image { mime_type, data } if mime_type == "image/png" && data == "AQID")
    );
    assert!(
        matches!(&result.outputs[4], Output::Table { columns, rows } if columns == &["name", "score"] && rows[0] == ["alice", "10"])
    );
    assert!(
        matches!(&result.outputs[5], Output::Table { columns, rows } if columns == &["city", "pop"] && rows[0] == ["Oslo", "700000"])
    );
}
