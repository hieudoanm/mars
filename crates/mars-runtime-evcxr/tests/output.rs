use mars_core::Output;
use mars_runtime_evcxr::output::append_mime_output;
use serde_json::{Map, Value};

#[test]
fn promotes_plotters_svg_html_to_a_mime_bundle() {
    let mut outputs = Vec::new();
    append_mime_output(
        &mut outputs,
        Map::from_iter([(
            "text/html".to_owned(),
            Value::String(
                "<div style=\"\"><svg xmlns=\"http://www.w3.org/2000/svg\"><circle cx=\"4\" cy=\"4\" r=\"4\"/></svg></div>"
                    .to_owned(),
            ),
        )]),
    );

    assert!(matches!(
        outputs.as_slice(),
        [Output::MimeBundle { data, .. }]
            if data.contains_key("image/svg+xml")
                && data.contains_key("text/plain")
                && !data.contains_key("text/html")
    ));
    let Output::MimeBundle { data, .. } = &outputs[0] else {
        return;
    };
    assert!(
        data["image/svg+xml"]
            .as_str()
            .is_some_and(|svg| svg.starts_with("<svg"))
    );
}

#[test]
fn keeps_image_mime_bundles_untouched() {
    let mut outputs = Vec::new();
    append_mime_output(
        &mut outputs,
        Map::from_iter([
            (
                "image/png".to_owned(),
                Value::String("iVBORw0KGgo=".to_owned()),
            ),
            (
                "text/html".to_owned(),
                Value::String("<svg><circle/></svg>".to_owned()),
            ),
        ]),
    );

    assert!(matches!(
        outputs.as_slice(),
        [Output::MimeBundle { data, .. }]
            if data.contains_key("image/png") && data.contains_key("text/html")
    ));
}

#[test]
fn leaves_html_with_surrounding_text_as_html() {
    let mut outputs = Vec::new();
    append_mime_output(
        &mut outputs,
        Map::from_iter([(
            "text/html".to_owned(),
            Value::String("<p>Inline icon</p><svg><circle/></svg>".to_owned()),
        )]),
    );

    assert!(matches!(
        outputs.as_slice(),
        [Output::Html { html }] if html.contains("Inline icon")
    ));
}

#[test]
fn still_collapses_single_text_and_html_outputs() {
    let mut outputs = Vec::new();
    append_mime_output(
        &mut outputs,
        Map::from_iter([("text/plain".to_owned(), Value::String("42".to_owned()))]),
    );
    assert!(matches!(outputs.as_slice(), [Output::Text { text }] if text == "42"));

    let mut outputs = Vec::new();
    append_mime_output(
        &mut outputs,
        Map::from_iter([(
            "text/html".to_owned(),
            Value::String("<b>safe</b>".to_owned()),
        )]),
    );
    assert!(matches!(outputs.as_slice(), [Output::Html { html }] if html == "<b>safe</b>"));
}

#[test]
fn drain_channel_joins_lines_and_handles_empty() {
    use mars_runtime_evcxr::output::drain_channel;
    let (tx, rx) = crossbeam_channel::unbounded::<String>();
    assert_eq!(drain_channel(&rx), "");
    tx.send("one".to_owned()).expect("send");
    tx.send("two".to_owned()).expect("send");
    drop(tx);
    assert_eq!(drain_channel(&rx), "one\ntwo\n");
}

#[test]
fn append_mime_output_edge_cases() {
    use mars_runtime_evcxr::output::{append_mime_output, promote_html_svg};
    // Empty bundle is dropped.
    let mut outputs = Vec::new();
    append_mime_output(&mut outputs, Map::new());
    assert!(outputs.is_empty());

    // Single text/plain collapses to Text.
    append_mime_output(
        &mut outputs,
        Map::from_iter([("text/plain".to_owned(), Value::String("hi".to_owned()))]),
    );
    assert!(matches!(outputs.last(), Some(Output::Text { text }) if text == "hi"));

    // Single text/html collapses to Html.
    append_mime_output(
        &mut outputs,
        Map::from_iter([("text/html".to_owned(), Value::String("<b>x</b>".to_owned()))]),
    );
    assert!(matches!(outputs.last(), Some(Output::Html { html }) if html == "<b>x</b>"));

    // Multiple keys stay a mime bundle.
    append_mime_output(
        &mut outputs,
        Map::from_iter([
            ("text/plain".to_owned(), Value::String("x".to_owned())),
            ("text/html".to_owned(), Value::String("<i>x</i>".to_owned())),
        ]),
    );
    assert!(matches!(outputs.last(), Some(Output::MimeBundle { .. })));

    // promote_html_svg leaves bundles with image/* untouched.
    let mut data = Map::from_iter([
        (
            "text/html".to_owned(),
            Value::String("<svg></svg>".to_owned()),
        ),
        ("image/png".to_owned(), Value::String("aGk=".to_owned())),
    ]);
    promote_html_svg(&mut data);
    assert!(data.contains_key("text/html"));
    assert!(!data.contains_key("image/svg+xml"));

    // Non-standalone-svg html is not promoted.
    let mut data = Map::from_iter([(
        "text/html".to_owned(),
        Value::String("<div>no svg</div>".to_owned()),
    )]);
    promote_html_svg(&mut data);
    assert!(data.contains_key("text/html"));
}
