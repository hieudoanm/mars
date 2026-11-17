# Project Mars protocol

`mars-protocol` defines the typed boundary between a desktop or external
frontend and the Rust kernel. Requests and responses are JSON-serializable and
carry a caller-provided correlation ID.

The current request types are:

- `execute` — compile and run one cell in the persistent session. It
  accepts an optional `timeout_millis` (omitted values preserve unlimited
  runtime execution) and an optional `language` selecting the runtime for
  multi-language `mars kernel serve` sessions; single-runtime kernels
  ignore it.
- `reset` — clear the runtime state.
- `status` — query execution status and count.
- `cancel` — request cancellation of the in-flight execution. The
  `mars kernel serve` loop reads requests concurrently with a running
  `execute`, so cancellation is honoured while a cell is still compiling or
  running; the server acknowledges immediately with a `cancelled` response.
- `input` — write a line to the running cell's stdin (`input()` /
  `mars_prompt` prompts). `mars kernel serve` answers these through each
  backend's stdin-writer fast path so they are not blocked behind the
  in-flight `execute`.
- `variables` — inspect live session bindings (name, type, value preview).
  Replay runtimes re-execute committed cells to inspect them, so cell side
  effects run again; this is a documented runtime limitation.
- `diagnose` — compile-check a candidate cell source without committing it and
  return typed diagnostics (`cell_index`, line/column spans, severity, code,
  message).

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
mars kernel serve [--runtime replay|evcxr] [--notebook <file.mars>]
```

It reads one `RequestEnvelope` per line and writes one `ResponseEnvelope` per
line, allowing integrations to keep a kernel session alive without coupling
the runtime to the desktop process. Request reading runs on a separate thread
from execution so `cancel` (and queued `reset`/`status`) requests are processed
while an `execute` is blocked; `EventEnvelope` stream lines may interleave
before the matching response. `--notebook` supplies the notebook metadata
(Cargo dependency declarations and an optional lockfile) used to configure the
session; the VS Code extension passes the active notebook path for this reason.

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
