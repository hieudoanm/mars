use mars_core::{CellId, CellType, CoreError, ExecutionStatus, Output, PaperSize};

#[test]
fn cell_id_new_generates_unique_displayable_ids() {
    let first = CellId::new();
    let second = CellId::new();

    assert_ne!(first, second);
    assert_eq!(first.as_str(), first.to_string());
    assert!(!first.as_str().is_empty());
}

#[test]
fn cell_id_from_string_rejects_blank_values() {
    assert!(matches!(
        CellId::from_string(""),
        Err(CoreError::InvalidCellId)
    ));
    assert!(matches!(
        CellId::from_string("   "),
        Err(CoreError::InvalidCellId)
    ));
}

#[test]
fn cell_id_from_string_accepts_non_empty_values() {
    let id = CellId::from_string("cell-1").expect("valid id");
    assert_eq!(id.as_str(), "cell-1");
}

#[test]
fn cell_id_serializes_as_plain_string() {
    let id = CellId::from_string("cell-1").expect("valid id");
    let json = serde_json::to_string(&id).expect("serializes");
    assert_eq!(json, "\"cell-1\"");

    let restored: CellId = serde_json::from_str(&json).expect("deserializes");
    assert_eq!(restored, id);
}

#[test]
fn cell_type_uses_lowercase_wire_names() {
    assert_eq!(
        serde_json::to_string(&CellType::Markdown).expect("serializes"),
        "\"markdown\""
    );
    assert_eq!(
        serde_json::to_string(&CellType::Code).expect("serializes"),
        "\"code\""
    );
    assert_eq!(
        serde_json::to_string(&CellType::Raw).expect("serializes"),
        "\"raw\""
    );

    let restored: CellType = serde_json::from_str("\"raw\"").expect("deserializes");
    assert_eq!(restored, CellType::Raw);
}

#[test]
fn execution_status_uses_snake_case_wire_names() {
    assert_eq!(
        serde_json::to_string(&ExecutionStatus::Succeeded).expect("serializes"),
        "\"succeeded\""
    );
    assert_eq!(
        serde_json::to_string(&ExecutionStatus::Interrupted).expect("serializes"),
        "\"interrupted\""
    );

    let restored: ExecutionStatus = serde_json::from_str("\"interrupted\"").expect("deserializes");
    assert_eq!(restored, ExecutionStatus::Interrupted);
}

#[test]
fn paper_size_parses_display_names_and_dimensions() {
    for (name, expected) in [
        ("a0", PaperSize::A0),
        ("a4", PaperSize::A4),
        ("b5", PaperSize::B5),
        ("letter", PaperSize::Letter),
        ("legal", PaperSize::Legal),
        ("tabloid", PaperSize::Tabloid),
    ] {
        let parsed = PaperSize::parse(name).expect("parses");
        assert_eq!(parsed, expected);
        assert_eq!(parsed.to_string(), name);
    }

    assert!(PaperSize::parse("A4").is_err());
    assert!(PaperSize::parse("unknown").is_err());
}

#[test]
fn paper_size_reports_iso_and_default_properties() {
    assert!(PaperSize::A4.is_iso216());
    assert!(!PaperSize::Letter.is_iso216());
    assert!(PaperSize::A4.is_default());
    assert!(!PaperSize::A3.is_default());

    let (width, height) = PaperSize::A4.dimensions();
    assert!((width - 595.28).abs() < f64::EPSILON);
    assert!((height - 841.89).abs() < f64::EPSILON);
    assert!((PaperSize::A4.ratio() - width / height).abs() < f64::EPSILON);
}

#[test]
fn paper_size_covers_all_variants() {
    let cases = [
        ("a0", PaperSize::A0, true),
        ("a1", PaperSize::A1, true),
        ("a2", PaperSize::A2, true),
        ("a3", PaperSize::A3, true),
        ("a4", PaperSize::A4, true),
        ("a5", PaperSize::A5, true),
        ("a6", PaperSize::A6, true),
        ("b4", PaperSize::B4, true),
        ("b5", PaperSize::B5, true),
        ("letter", PaperSize::Letter, false),
        ("legal", PaperSize::Legal, false),
        ("tabloid", PaperSize::Tabloid, false),
    ];

    for (name, expected, iso) in cases {
        let parsed = PaperSize::parse(name).expect("parses");
        assert_eq!(parsed, expected);
        assert_eq!(parsed.to_string(), name);
        assert_eq!(parsed.is_iso216(), iso);

        let (width, height) = parsed.dimensions();
        assert!(width > 0.0);
        assert!(height > 0.0);
        assert!(parsed.ratio() < 1.0);
    }
}

#[test]
fn output_uses_tagged_snake_case_wire_format() {
    let output = Output::Text {
        text: "hello".to_owned(),
    };
    let json = serde_json::to_value(&output).expect("serializes");
    assert_eq!(json["type"], "text");
    assert_eq!(json["text"], "hello");

    let restored: Output = serde_json::from_value(json).expect("deserializes");
    assert_eq!(restored, output);
}

#[test]
fn output_mime_bundle_preserves_arbitrary_mime_data() {
    let mut data = serde_json::Map::new();
    data.insert(
        "application/vnd.custom".to_owned(),
        serde_json::json!({"value": 42}),
    );
    let output = Output::MimeBundle {
        data,
        metadata: serde_json::Map::new(),
    };

    let json = serde_json::to_string(&output).expect("serializes");
    let restored: Output = serde_json::from_str(&json).expect("deserializes");
    assert_eq!(restored, output);
}
