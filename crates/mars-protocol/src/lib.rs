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
    },
    Reset,
    Status,
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
    Error {
        message: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum KernelEvent {
    StatusChanged { status: ExecutionStatus },
    Output { output: Output },
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
            }
        );
    }
}
