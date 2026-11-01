# Mars protocol

`mars-protocol` defines the typed boundary between a desktop or external
frontend and the Rust kernel. Requests and responses are JSON-serializable and
carry a caller-provided correlation ID.

The current request types are:

- `execute` — compile and run one Rust cell in the persistent session. It
  accepts an optional `timeout_millis`; omitted values preserve unlimited
  runtime execution.
- `reset` — clear the runtime state.
- `status` — query execution status and count.

An execute response carries the execution status, execution count, duration,
and zero or more typed outputs. Errors are represented as outputs rather than
unstructured transport failures when the runtime successfully handled the
request. Transport or protocol failures remain request-level errors.

`mars-kernel` owns a `mars-runtime::RustSession` and translates requests into
responses. The frontend should depend on this protocol rather than importing
runtime or notebook persistence internals.

The protocol is intentionally transport-neutral. A later Tauri command layer,
JSON-RPC endpoint, or WebSocket transport can carry these messages without
changing the domain types.

The protocol is not the `.mars` file format. Notebook persistence owns cell
metadata, source, outputs, and migrations; the protocol owns live session
operations and execution events. Keeping those responsibilities separate
allows a notebook to be edited without an active kernel and allows a kernel to
serve clients other than the desktop UI.

The CLI provides a persistent newline-delimited JSON adapter today:

```bash
mars kernel serve
```

It reads one `RequestEnvelope` per line and writes one `ResponseEnvelope` per
line, allowing integrations to keep a kernel session alive without coupling
the runtime to the desktop process.

Timed-out execution returns an execution response containing an `Interrupted`
error output, leaves the execution count unchanged, and reports
`interrupted` through the next status response. The Tauri layer exposes this as
`execute_cell_with_timeout`; the existing `execute_cell` command remains
unbounded for compatibility. Desktop cancellation is exposed separately as
`cancel_execution`; it signals the active process and is safe to call while an
execution command is waiting on the kernel mutex.

## JSONL usage

`mars kernel serve` accepts one JSON request per line and emits one JSON
response per line. A client should:

1. Generate a unique correlation ID for each request.
2. Write a complete JSON object followed by a newline.
3. Read exactly one response line for that request.
4. Treat an interrupted execution as a completed response with interrupted
   status, not as permission to reuse partial bindings.
5. Send `reset` when it needs a clean session.

The JSONL adapter is intentionally simple and does not yet provide a WebSocket
or JSON-RPC framing layer. Those transports are future adapters over the same
typed protocol.
