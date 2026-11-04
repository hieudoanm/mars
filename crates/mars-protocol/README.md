# mars-protocol

Communication protocol between frontend and backend for the Mars notebook platform.

## Purpose

`mars-protocol` defines typed request/response messages and events for kernel communication. Suitable for WebSocket or JSON-RPC transport.

## Messages

- **`KernelRequest`** — Execute (with optional timeout), Reset, Status.
- **`KernelResponse`** — Execution (outputs, count, duration), Status, Reset, Error.
- **`KernelEvent`** — StatusChanged, Output.
- **`RequestEnvelope`** / **`ResponseEnvelope`** — ID-tagged message wrappers.

All types are Serde-serializable with `#[serde(tag = "type", rename_all = "snake_case")]` for JSON transport.

## Usage

```rust
use mars_protocol::{KernelRequest, RequestEnvelope, KernelResponse};

let envelope = RequestEnvelope::new(KernelRequest::Execute {
    source: "1 + 1".to_owned(),
    timeout_millis: None,
});
let json = serde_json::to_string(&envelope)?;
```
