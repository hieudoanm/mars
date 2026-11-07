use mars_runtime::format_compiler_diagnostics;

#[test]
fn formats_flat_rustc_diagnostics() {
    let stderr = br#"{"$message_type":"diagnostic","message":"mismatched types","code":{"code":"E0308"},"level":"error","spans":[{"file_name":"main.rs","line_start":2,"column_start":18,"line_end":2,"column_end":24,"is_primary":true,"label":null}],"children":[],"rendered":"error[E0308]: mismatched types\n --> main.rs:2:18\n"}
{"$message_type":"diagnostic","message":"aborting due to 1 previous error","level":"error","spans":[],"children":[],"rendered":"error: aborting due to 1 previous error\n"}"#;
    let output = format_compiler_diagnostics(stderr);
    assert!(output.contains("error[E0308]: mismatched types"));
    assert!(output.contains("--> main.rs:2:18"));
    assert!(output.contains("aborting due to 1 previous error"));
    assert!(!output.contains("compiler diagnostic"));
}

#[test]
fn formats_wrapped_cargo_diagnostics() {
    let stderr = br#"{"reason":"compiler-message","message":{"message":"cannot find value `x`","level":"error","code":{"code":"E0425"},"spans":[],"children":[],"rendered":null}}"#;
    let output = format_compiler_diagnostics(stderr);
    assert!(output.contains("error [E0425]: cannot find value `x`"));
}

#[test]
fn falls_back_to_structured_output_without_rendered() {
    let stderr = br#"{"$message_type":"diagnostic","message":"some failure","level":"warning","spans":[{"file_name":"a.rs","line_start":1,"column_start":1,"line_end":1,"column_end":2,"is_primary":true,"label":"here"}],"children":[{"level":"note","message":"a note"}]}"#;
    let output = format_compiler_diagnostics(stderr);
    assert!(output.contains("warning: some failure"));
    assert!(output.contains("--> a.rs:1:1-1:2 [primary] here"));
    assert!(output.contains("note: a note"));
}

#[test]
fn skips_non_diagnostic_json_and_keeps_plain_text() {
    let stderr = br#"{"reason":"build-finished","success":false}
{"reason":"compiler-artifact","package_id":"x"}
error: could not compile `demo`"#;
    let output = format_compiler_diagnostics(stderr);
    assert_eq!(output, "error: could not compile `demo`");
}

#[test]
fn returns_raw_output_when_nothing_parses() {
    let stderr = b"plain rustc failure\nwith details";
    assert_eq!(
        format_compiler_diagnostics(stderr),
        "plain rustc failure\nwith details"
    );
}
