# mars-runtime — Rust Execution Engine

Executes Rust code interactively with shared state across cells. Depends on `mars-core`.

## Architecture

Two execution backends implement `RuntimeBackend` (`backend.rs`):

- **`RustSession`** (default, "replay"): accumulates cell sources, concatenates them into one `fn main()` program (`expression.rs::render_program`), compiles with `rustc --edition=2024 --error-format=json` when no dependencies are declared, or generates a Cargo project (`manifest.rs`, `toml.rs`) and uses `cargo build` otherwise. Runs the binary as a subprocess with process-group isolation (`execution.rs::configured_command`, `terminate_process_tree` — SIGKILL on the process group / `taskkill /T /F`).
- **`mars-runtime-evcxr`** (experimental, optional): real persistent REPL via the `evcxr` crate. Only compiled in via the `experimental-evcxr` feature on `mars-kernel`/`mars-cli`.

## Key modules

- `session.rs` — `RustSession` workdir under `temp_dir()/mars-runtime-<uuid>`; `Drop` removes it. Stores `cells`, `execution_count`, `dependencies`, `cargo_lock`, `offline`, `locked`, and `stdin`/`stdin_slot` handles for `mars_prompt`.
- `backend.rs` — `RuntimeBackend` trait plus `StdinWriter`, a cloneable `Fn(Option<&str>)` handle letting callers answer prompts while `execute_*` holds `&mut self` (used by the Tauri `input_writers` map and `mars kernel serve`'s `Input` fast path). `None` signals EOF.
- `session_execution.rs` — `execute_with_control`: builds candidate cell list, compiles, falls back to `ExpressionOutput::Display` if the trailing expression fails, then runs the binary with timeout/cancellation polling.
- `expression.rs` — wraps cells in `fn main()`, strips trailing `;`, detects final expressions (`split_final_expression`), emits `__MARS_RESULT__=` marker for display output. Only the _last_ cell auto-displays — replayed cells discard final expressions to avoid duplicate output.
- `compilation.rs` — `compile_command` picks rustc vs cargo; writes `Cargo.toml`/`Cargo.lock` for dependency builds.
- `diagnostics.rs` — parses rustc JSON diagnostics into readable messages with spans.
- `output.rs` — `ExecutionResult`, `ExpressionOutput` (Debug/Display/Discard), `BinaryOutput`, stdout marker normalization, Evcxr content-block parsing. `StreamEvent::Comm` carries `mars_core::CommMessage` widget traffic; `RuntimeBackend::handle_comm` delivers inbound comm messages (default: ignored).
- `cancellation.rs` — `CancellationToken` wrapping `Arc<AtomicBool>`.

## Rules

- Never execute user code in the host process — always subprocess isolation.
- Timeout and cancellation must kill the whole process tree, not just the direct child.
- Keep `RuntimeBackend` implementors honest: `reset()` clears all state; `execution_count` increments only on successful evaluation.
- No `unwrap()`/`expect()` in production paths; errors go through `RuntimeError`.

## Testing

```bash
cargo test -p mars-runtime
cargo clippy -p mars-runtime --all-targets -- -D warnings
cargo run -p mars-runtime --bin runtime_benchmark   # perf benchmark binary
```

Line and function coverage must stay at or above 80% — verify with `cargo llvm-cov -p mars-runtime --tests` (workspace gate: `make coverage`).
