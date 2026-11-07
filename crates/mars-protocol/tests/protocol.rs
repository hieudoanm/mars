use mars_core::{ExecutionStatus, Output};
use mars_protocol::{
    CellDiagnostic, EventEnvelope, KernelChunkStream, KernelEvent, KernelRequest, KernelResponse,
    KernelStreamEvent, KernelStreamPhase, RequestEnvelope, ResponseEnvelope, VariableInfo,
};

#[test]
fn request_round_trips_as_json() {
    let envelope = RequestEnvelope {
        id: "request-1".to_owned(),
        request: KernelRequest::Execute {
            source: "println!(\"hello\");".to_owned(),
            timeout_millis: None,
            language: None,
        },
    };
    let json = serde_json::to_string(&envelope).expect("serializes");
    let restored: RequestEnvelope = serde_json::from_str(&json).expect("deserializes");
    assert_eq!(envelope, restored);
}

#[test]
fn older_execute_requests_default_to_no_timeout() {
    let json = r#"{"id":"request-1","request":{"type":"execute","source":"println!(\"hello\");"}}"#;
    let restored: RequestEnvelope = serde_json::from_str(json).expect("deserializes");

    assert_eq!(
        restored.request,
        KernelRequest::Execute {
            source: "println!(\"hello\");".to_owned(),
            timeout_millis: None,
            language: None,
        }
    );
}

#[test]
fn request_envelope_new_generates_unique_ids() {
    let first = RequestEnvelope::new(KernelRequest::Status);
    let second = RequestEnvelope::new(KernelRequest::Status);

    assert_ne!(first.id, second.id);
    assert_eq!(first.request, KernelRequest::Status);
}

#[test]
fn kernel_requests_round_trip_all_variants() {
    let requests = vec![
        KernelRequest::Execute {
            source: "let x = 1;".to_owned(),
            timeout_millis: Some(1_000),
            language: Some("python".to_owned()),
        },
        KernelRequest::Reset,
        KernelRequest::Status,
        KernelRequest::Cancel,
        KernelRequest::Variables,
        KernelRequest::Diagnose {
            source: "let y = 2;".to_owned(),
        },
        KernelRequest::Input {
            text: "answer".to_owned(),
        },
    ];

    for request in requests {
        let envelope = RequestEnvelope {
            id: "request-1".to_owned(),
            request,
        };
        let json = serde_json::to_string(&envelope).expect("serializes");
        let restored: RequestEnvelope = serde_json::from_str(&json).expect("deserializes");
        assert_eq!(envelope, restored);
    }
}

#[test]
fn kernel_responses_round_trip_all_variants() {
    let responses = vec![
        KernelResponse::Execution {
            outputs: vec![Output::Text {
                text: "done".to_owned(),
            }],
            execution_count: 3,
            duration_millis: 12,
        },
        KernelResponse::Status {
            status: ExecutionStatus::Running,
            execution_count: 3,
        },
        KernelResponse::Reset {
            status: ExecutionStatus::Idle,
        },
        KernelResponse::Cancelled,
        KernelResponse::Variables {
            variables: vec![VariableInfo {
                name: "x".to_owned(),
                type_name: Some("i32".to_owned()),
                value: Some("1".to_owned()),
            }],
        },
        KernelResponse::Diagnostics {
            diagnostics: vec![CellDiagnostic {
                cell_index: Some(0),
                line: 1,
                column: 2,
                end_line: 1,
                end_column: 5,
                severity: "error".to_owned(),
                code: Some("E0308".to_owned()),
                message: "mismatched types".to_owned(),
            }],
        },
        KernelResponse::Input,
        KernelResponse::Error {
            message: "kernel failed".to_owned(),
        },
    ];

    for response in responses {
        let envelope = ResponseEnvelope {
            id: "response-1".to_owned(),
            response,
        };
        let json = serde_json::to_string(&envelope).expect("serializes");
        let restored: ResponseEnvelope = serde_json::from_str(&json).expect("deserializes");
        assert_eq!(envelope, restored);
    }
}

#[test]
fn kernel_events_round_trip_all_variants() {
    let events = vec![
        KernelEvent::StatusChanged {
            status: ExecutionStatus::Succeeded,
        },
        KernelEvent::Output {
            output: Output::Text {
                text: "streamed".to_owned(),
            },
        },
    ];

    for event in events {
        let json = serde_json::to_string(&event).expect("serializes");
        let restored: KernelEvent = serde_json::from_str(&json).expect("deserializes");
        assert_eq!(restored, event);
    }
}

#[test]
fn stream_events_round_trip_all_variants() {
    let events = vec![
        KernelStreamEvent::Phase {
            phase: KernelStreamPhase::Compiling,
        },
        KernelStreamEvent::Chunk {
            stream: KernelChunkStream::Stdout,
            data: "hello".to_owned(),
        },
        KernelStreamEvent::Prompt {
            message: "continue?".to_owned(),
        },
    ];

    for event in events {
        let envelope = EventEnvelope {
            id: "event-1".to_owned(),
            event,
        };
        let json = serde_json::to_string(&envelope).expect("serializes");
        let restored: EventEnvelope = serde_json::from_str(&json).expect("deserializes");
        assert_eq!(envelope, restored);
    }
}
