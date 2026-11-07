use mars_core::{ExecutionStatus, Output};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RequestEnvelope {
    pub id: String,
    pub request: KernelRequest,
}

impl RequestEnvelope {
    pub fn new(request: KernelRequest) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            request,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum KernelRequest {
    Execute {
        source: String,
        #[serde(default)]
        timeout_millis: Option<u64>,
        /// Optional language override (e.g. `"python"`). When served by
        /// `mars kernel serve`, selects which language runtime executes the
        /// cell; single-runtime kernels ignore it.
        #[serde(default)]
        language: Option<String>,
    },
    Reset,
    Status,
    /// Request cancellation of the in-flight execution. Processed by the
    /// server loop concurrently with a running `Execute`; acknowledged with
    /// `KernelResponse::Cancelled`.
    Cancel,
    /// Inspect live bindings in the session. Replay runtimes re-execute cell
    /// side effects; see `mars-runtime` documentation for limitations.
    Variables,
    /// Compile-check `source` as a candidate next cell without committing it.
    Diagnose {
        source: String,
    },
    /// Write a line to the stdin of the currently running cell program.
    Input {
        text: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ResponseEnvelope {
    pub id: String,
    pub response: KernelResponse,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum KernelResponse {
    Execution {
        outputs: Vec<Output>,
        execution_count: u64,
        duration_millis: u128,
    },
    Status {
        status: ExecutionStatus,
        execution_count: u64,
    },
    Reset {
        status: ExecutionStatus,
    },
    Cancelled,
    Variables {
        variables: Vec<VariableInfo>,
    },
    Diagnostics {
        diagnostics: Vec<CellDiagnostic>,
    },
    Input,
    Error {
        message: String,
    },
}

/// Wire form of a session variable reported by `KernelResponse::Variables`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VariableInfo {
    pub name: String,
    pub type_name: Option<String>,
    pub value: Option<String>,
}

/// Wire form of a compile diagnostic reported by `KernelResponse::Diagnostics`.
/// `cell_index` refers to the candidate cell list inside the runtime session.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CellDiagnostic {
    pub cell_index: Option<usize>,
    pub line: u32,
    pub column: u32,
    pub end_line: u32,
    pub end_column: u32,
    pub severity: String,
    pub code: Option<String>,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum KernelEvent {
    StatusChanged { status: ExecutionStatus },
    Output { output: Output },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KernelStreamPhase {
    Compiling,
    Running,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KernelChunkStream {
    Stdout,
    Stderr,
}

/// Wire representation of a runtime `StreamEvent`, emitted by
/// `mars kernel serve` as `EventEnvelope` lines while an `Execute` request
/// is in flight, before the final `ResponseEnvelope`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum KernelStreamEvent {
    Phase {
        phase: KernelStreamPhase,
    },
    Chunk {
        stream: KernelChunkStream,
        data: String,
    },
    Prompt {
        message: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub id: String,
    pub event: KernelStreamEvent,
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let json =
            r#"{"id":"request-1","request":{"type":"execute","source":"println!(\"hello\");"}}"#;
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
}
