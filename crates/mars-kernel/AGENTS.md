# mars-kernel — Kernel Lifecycle & Request Handling

Bridges the protocol layer and runtime backends. Depends on `mars-core`, `mars-protocol`, `mars-runtime` (and optionally `mars-runtime-evcxr`).

## Responsibilities

- `Kernel` — owns a `Box<dyn RuntimeBackend>` and an `ExecutionStatus`. Constructed via `new` / `with_dependencies` / `with_backend` / `with_configuration`.
- `RuntimeBackendKind` — `Replay` (default) | `Evcxr`. `from_environment()` reads `MARS_EXPERIMENTAL_RUNTIME=evcxr`.
- `handle(RequestEnvelope) -> ResponseEnvelope` — dispatches `KernelRequest::{Execute, Reset, Status}` to `KernelResponse`. Updates `status` to `Running` before execution and maps results: `interrupted` → `Interrupted`, any `Output::Error` → `Failed`, else `Succeeded`.
- `runtime.rs::create_runtime` — factory selecting the backend. Evcxr is gated behind the `experimental-evcxr` feature and rejects `cargo_lock`/`locked`.
- `runtime_hook()` — forwards to `mars_runtime_evcxr::runtime_hook` when the feature is enabled; must run before any Evcxr use.

## Rules

- The kernel is in-process and synchronous — it does not spawn background kernels (`mars kernel list` reflects this).
- Execution requests accept an optional `timeout_millis` and an external cancellation flag via `handle_with_cancellation` / `execute_source`.
- Keep `KernelResponse` mapping in one place (`execute`); callers should not re-derive status.

## Testing

```bash
cargo test -p mars-kernel
cargo clippy -p mars-kernel --all-targets -- -D warnings
cargo test -p mars-kernel --features experimental-evcxr   # also test the evcxr path
```
