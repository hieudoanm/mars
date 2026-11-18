# mars-tauri — desktop Rust host

Scope: `packages/desktop/src-tauri/**`. Follow the repository-level `AGENTS.md`
and `packages/desktop/AGENTS.md` first; these instructions describe the Tauri
backend crate specifically.

## Responsibilities

- `lib.rs` owns `AppState` and the `generate_handler!` registry — every new
  `#[tauri::command]` must be added there and must convert errors to
  `Result<T, String>` explicitly.
- `kernel*.rs` own kernel lifecycle: kernels are keyed by `(session,
language)` and lazily created via `kernel_for`; stdin writers,
  `python_options`, `sql_sources`, and `cancellations` are parallel maps that
  must stay consistent with the kernel entry.
- `lsp*.rs` own per-notebook `mars-lsp` sessions; sync cells/dependencies
  before serving editor queries.
- `notebook_cmds.rs`/`notebook_files.rs`/`checkpoints.rs`/`open.rs` own all
  filesystem access — validate paths, stay inside checkpoint directories,
  and prefer `mars-document` (`load`, `save_atomic`) over raw file I/O.
- `lab/` is a thin command layer over `mars-lab`; the `Database` `OnceCell`
  in `lab/mod.rs` is the single shared connection — do not open additional
  connections.
- The frontend boundary is `packages/desktop/src/lib/kernelService.ts`:
  command names and argument casing must match on both sides.

## Rules

- No `unwrap()`/`expect()` in production code; return `Err(String)`.
- Do not execute notebook source in the Tauri process — route through
  `mars-kernel` runtimes only.
- Preserve cancellation, timeout, and reset semantics; kernel state is
  intentionally process-local.
- `tauri.conf.json` CSP stays as restrictive as possible; `capabilities/`
  permissions stay minimal (`core:default`, `dialog:default`, window close).
- Adding a Tauri plugin requires all of: the `tauri-plugin-*` crate in
  `Cargo.toml`, `.plugin(...)` registration in `lib.rs`, the matching
  `@tauri-apps/plugin-*` npm package, a permission entry in
  `capabilities/default.json`, and any plugin config under `plugins` in
  `tauri.conf.json`. `tauri-plugin-dialog` is the reference wiring.
- Keep serde payload shapes stable — they are the wire contract with the
  React frontend (`src/types/types.ts` mirrors them).

## Testing

```bash
cargo fmt --all --check
cargo clippy -p mars-tauri --all-targets -- -D warnings
cargo test -p mars-tauri
```

Line and function coverage must stay at or above 80% — `make coverage-tauri`
enforces both thresholds. When adding commands or helpers, keep the decision
logic in `pub` functions that take plain data (no `AppHandle`) so integration
tests can drive them; `AppHandle`-bound wrappers should stay thin. Tauri
helpers that need a handle should be generic over `tauri::Runtime` so tests
can pass a `tauri::test::mock_builder()` app (register
`tauri_plugin_notification::init()` when the notification path is exercised).

Tests are integration tests under `tests/` (linking `mars_tauri_lib`) — do not
add `#[cfg(test)]` modules inside `src/`. Anything a test exercises must be
`pub` and reachable from the crate root (e.g. `pub mod notebook_files`).

End-to-end coverage lives in `packages/desktop/e2e/` (`test:e2e:tauri` via
`tauri-driver`, Linux/Windows only).

## Layout

See `README.md` in this directory for the module map and config files.
Workspace-wide cargo commands compile this crate and need Tauri's Linux
system libraries (webkit2gtk stack) — CI installs them.

## License

Same as the repository — see root `LICENSE`.
