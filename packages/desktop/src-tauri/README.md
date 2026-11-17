# mars-tauri — Project Mars desktop host

Tauri 2 Rust backend for the Project Mars desktop app (`packages/desktop`).
It owns everything the React frontend must not: filesystem access, kernel
lifecycle, LSP sessions, and Mars Lab persistence. The frontend stays thin —
all notebook mutations go through `mars-document`, all execution through
`mars-kernel`, all rendering through `mars-renderer`.

## Responsibilities

- `lib.rs` — `AppState` (kernel/input-writer/python-option/sql-source/
  cancellation maps plus `LspSessions` and the `RuntimeBackendKind`), the
  `tauri::Builder` wiring, and the `generate_handler!` command registry.
  Every `#[tauri::command]` in the crate must appear in that list.
- `kernel.rs` — shared kernel helpers: `kernel_key`, `kernel_for` (lazy
  per-`(session, language)` `Kernel` creation), `kernels_lock`,
  `stream_sink` (forwards `StreamEvent`s to the webview), cancellation
  tokens, and stdin writer registration.
- `kernel_exec.rs` — execution commands: `execute_cell`,
  `execute_cell_with_timeout`, `cancel_execution`, `reset_kernel`,
  `kernel_status`, `configure_kernel`.
- `kernel_query.rs` — inspection commands: `inspect_variables`,
  `diagnose_cell`, `send_kernel_input`, `send_kernel_comm`.
- `lsp.rs` / `lsp_cmds.rs` — `LspSessions` (per-session `mars-lsp`
  `LspSession` map keyed by notebook path) and the editor commands:
  `lsp_sync`, hover, completion, signature help, code actions, definition,
  inlay hints, semantic tokens, diagnostics.
- `notebook_cmds.rs` — notebook commands: open/save (via `mars-document`
  `load`/`save_atomic`), file explorer operations (`notebook_files.rs`,
  `pub` so `tests/` can exercise it),
  external open/terminal (`open.rs`), checkpoints (`checkpoints.rs`),
  import/export (`.ipynb`, Markdown, HTML, PDF, slides).
- `lab/` — Mars Lab commands as a thin layer over `mars-lab`: dataset
  upload/preview/insights (`datasets.rs`), training jobs (`train.rs`),
  job listing/metrics (`jobs.rs`). The shared `mars_lab::db::Database` is
  initialised lazily via `tokio::sync::OnceCell` in `lab/mod.rs`.
- `eval_output_script` — runs trusted renderer-side output scripts.

## Configuration

- `tauri.conf.json` — window (1280×860, min 840×640), restrictive CSP
  (`default-src 'self'`), bundle metadata/icons, dev URL `:1420`.
- `capabilities/default.json` — `core:default`, `core:window:allow-close`,
  `dialog:default` for the `main` window. Keep permissions minimal.
- Features: `custom-protocol` (Tauri custom protocol),
  `experimental-evcxr` (opt-in Evcxr Rust backend via `mars-kernel`).

## Build & test

```bash
cargo fmt --all --check
cargo clippy -p mars-tauri --all-targets -- -D warnings
cargo test -p mars-tauri
pnpm --filter @mars/desktop build   # produces ../dist consumed by tauri
```

The crate is a workspace member, so workspace-wide cargo commands compile it
and require Tauri's Linux system libraries (webkit2gtk etc.) — CI installs
them in `cli-quality`.

## Conventions

- Commands return `Result<T, String>`; convert errors with `map_err` /
  `to_string`, never `unwrap`/`expect`.
- Kernel state is process-local and keyed by `(session, language)` — do not
  recreate kernels from unrelated UI state.
- Argument casing must match the frontend callers in
  `packages/desktop/src/lib/kernelService.ts` (camelCase ↔ snake_case via
  serde/tauri conventions).
- Validate paths and collisions in Rust; the renderer is untrusted input.
- See `packages/desktop/AGENTS.md` for the frontend-side contract.

## Layout

```text
src-tauri/
├── Cargo.toml          # mars-tauri; lib crate-type staticlib/cdylib/rlib
├── build.rs            # tauri-build
├── tauri.conf.json     # window, CSP, bundle config
├── capabilities/       # Tauri permission sets (default.json)
├── icons/              # bundle icons
├── src/                # Rust host (see Responsibilities); no inline tests
└── tests/              # integration tests against mars_tauri_lib
│                       #   (notebook_files, notebook_cmds, kernel, lsp,
│                       #    lab, checkpoints, open)
```

Package name is `mars-tauri`; lib name `mars_tauri_lib` (mobile builds consume
the staticlib/cdylib).

## License

Same as the repository — see root `LICENSE`.
