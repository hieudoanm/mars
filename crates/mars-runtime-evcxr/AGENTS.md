# mars-runtime-evcxr — Experimental Evcxr Backend

Alternative `RuntimeBackend` implementation backed by the `evcxr` crate (pinned `=0.22.0`). Depends on `mars-core` and `mars-runtime`.

## Responsibilities

- `EvcxrSession` — wraps `evcxr::CommandContext` behind `Arc<Mutex<..>>` plus `EvalContextOutputs` channels for stdout/stderr.
- `context.rs` — creates the context, enables `:cache 500`, `:offline 1` when requested, and injects `:dep` commands for declared dependencies.
- `backend.rs` — `RuntimeBackend` impl: spawns a controller thread per execution, polls with `recv_timeout`, kills the evaluator subprocess and _recreates the whole context_ on timeout/cancellation (Evcxr cannot interrupt a running eval in place).
- `control.rs` — process termination and interrupted-result construction.
- `output.rs` — drains stdout/stderr channels; `append_mime_output` collapses single text/html outputs, promotes standalone-SVG HTML (e.g. plotters) into `image/svg+xml` mime bundles.
- `lib.rs::runtime_hook` — must be called at process start (`mars` CLI calls it in `main`) so Evcxr's re-executed runtime process enters the eval loop.

## Rules

- This crate is experimental: no `Cargo.lock`/`--locked` support (rejected in `mars-kernel::runtime::create_runtime`); `cargo_lock()` always returns `None`.
- Any evaluator-process death or timeout requires full context recreation — do not try to resume a tainted session.
- MIME promotion must not touch bundles that already contain `image/*` keys.

## Testing

```bash
cargo test -p mars-runtime-evcxr
cargo clippy -p mars-runtime-evcxr --all-targets -- -D warnings
cargo run -p mars-runtime-evcxr --bin benchmark   # benchmark binary
```

Line and function coverage must stay at or above 80% — verify with `cargo llvm-cov -p mars-runtime-evcxr --tests` (workspace gate: `make coverage`).
Enable via `--features experimental-evcxr` on `mars-kernel`/`mars-cli`, or `MARS_EXPERIMENTAL_RUNTIME=evcxr`.
