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
- Keep serde payload shapes stable — they are the wire contract with the
  React frontend (`src/types/types.ts` mirrors them).

## Testing

```bash
cargo fmt --all --check
cargo clippy -p mars-tauri --all-targets -- -D warnings
cargo test -p mars-tauri
```

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
