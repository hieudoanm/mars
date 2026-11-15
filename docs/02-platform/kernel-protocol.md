# Kernel protocol and future architecture

## Current boundary

Project Mars does not (yet) expose a standalone kernel protocol. Today the layers are:

```
React UI ──invoke()──> Tauri commands (src-tauri) ──> mars-kernel ──> mars-runtime
```

- `packages/desktop/src/lib/kernelService.ts` is the single frontend boundary for
  every Tauri command; components never call `invoke` directly.
- `packages/desktop/src-tauri/src/lib.rs` registers typed commands that delegate
  to `mars-kernel`, which owns session lifecycle, execution requests,
  streaming events, cancellation, and timeouts.
- `mars-runtime` executes cell code in an isolated subprocess (never in the
  host process) and returns typed `Output` values defined in `mars-core`.

This keeps the desktop app thin and the kernel reusable — `mars-cli` already
shares `mars-kernel`/`mars-runtime` for headless execution.

### `mars kernel serve` (JSON-lines over stdio or TCP)

`mars kernel serve [--runtime replay|evcxr] [--notebook <file.mars>] [--tcp <port>]`
runs a kernel in the current process and speaks `mars-protocol` as
newline-delimited JSON — on stdin/stdout by default, or over TCP when
`--tcp <port>` is given. The TCP listener binds `127.0.0.1` by default
because the protocol has no authentication; pass `--tcp-host <addr>`
(e.g. `0.0.0.0`) to expose it to remote clients deliberately. Each TCP
connection gets its own kernel session; connections are handled
concurrently.
one JSON message per line:

- Requests: `{"id": "...", "request": {"type": "execute"|"reset"|"status", ...}}`
- Responses: `{"id": "...", "response": {...}}`
- **Stream events**: while an `execute` request is in flight, the kernel also
  emits `{"id": "...", "event": {"type": "phase"|"chunk"|"prompt", ...}}`
  lines (compiling/running phases, stdout/stderr chunks, prompt requests)
  before the final response envelope. Covered by
  `crates/mars-cli/tests/kernel_serve.rs`.

Limitation: `mars_prompt` reads stdin, which is owned by the protocol channel,
so interactive prompts cannot be answered through `kernel serve` — prompt
events are surfaced for observability only.

## Future direction: a kernel transport

The intended next architectural step is to put a transport in front of
`mars-kernel` so the kernel can run independently of the Tauri host:

- **JSON-RPC over stdio / TCP** — implemented: `mars kernel serve` (stdio)
  and `mars kernel serve --tcp <port>` drive a kernel from any client.
  WebSocket remains the step needed for browser clients.
- **WebSocket or ZMQ** — required for remote kernels and browser clients;
  ZMQ would also allow a Jupyter-protocol gateway so `jupyter console` /
  `jupyter nbclient` could attach to a Project Mars kernel.
- **Session/multiplexing** — one transport should support multiple kernel
  sessions so tabs keep their per-notebook kernels.

`mars-protocol` already defines the typed message shapes this transport would
carry; the work is framing, session routing, and a reference client.

## Explicit scope decisions

The following are deliberately **out of scope** for now:

- **Remote/collaborative editing** — real-time multi-user documents need an
  OT/CRDT layer and conflict semantics that do not exist yet.
- ~~Widget protocol~~ — implemented for Python: the driver bridges the
  `comm` package, `KernelRequest::Comm` delivers front-end widget updates,
  and `KernelStreamEvent::Comm` streams widget traffic during `Execute`.
- **Extension ecosystem** — no plugin API is planned until the kernel
  transport stabilizes.
- **Second language runtime** — implemented: `rust`, `python`, `r`,
  `julia`, and `cpp` backends are registered in `mars-kernel`'s language
  registry; `kernel serve` multiplexes one kernel per language.
- **Debugger/breakpoints** — Rust debugging integration is a separate large
  project.

These decisions keep the protocol surface small enough to design well the
first time.
