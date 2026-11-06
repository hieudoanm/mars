# mars-desktop — GPUI Desktop Application

Pure-Rust desktop client for Mars built on [GPUI](https://gpui.rs) (`gpui = "=0.2.2"`). Binary: `mars-desktop`.

This crate replaces the earlier Tauri+React approach. The Tauri app still lives at `apps/desktop/` (package `mars-tauri`) — do not confuse the two.

## Architecture

- `src/main.rs` — `Application` bootstrap, key bindings (`cmd-n/o/s`, `cmd-shift-enter`, `cmd-r`, `cmd-q`), window creation, optional notebook path from `argv[1]`.
- `src/app.rs` — `NotebookApp` view: toolbar, scrollable cell list, status bar. Owns the `Notebook`, file `path`, and a shared `Arc<Mutex<Kernel>>`.
- `src/output_text.rs` — flattens `mars_core::Output` into plain-text lines for display.

## Rules

- All kernel access goes through `mars-kernel` (`handle_with_cancellation`, `Reset`, `Status`); never execute notebook code in the UI process beyond the kernel/runtime isolation.
- Kernel execution runs on `cx.background_executor()` inside `cx.spawn`; UI state is only mutated back on the main thread via `WeakEntity::update`.
- Notebook persistence uses `mars_document::{load, save_atomic}`; kernel (re)configuration uses `Kernel::with_backend` with `RuntimeBackendKind::from_environment()` (`MARS_EXPERIMENTAL_RUNTIME=evcxr` selects the experimental backend).
- File dialogs use GPUI's `prompt_for_paths` / `prompt_for_new_path` — no external dialog crates.
- Keep the dark Mars palette in the `const` colors at the top of `app.rs`.

## Current limitations (v1 scaffold)

- Cells are rendered read-only — no text editing yet (needs a GPUI input/editor element).
- Outputs render as plain text; rich outputs (HTML, images, LaTeX, charts) show placeholders.
- No cell add/delete/reorder, no markdown rendering, no LaTeX, no autosave, no per-cell keyboard shortcuts beyond the toolbar/global bindings.

## Testing

```bash
cargo check -p mars-desktop
cargo clippy -p mars-desktop --all-targets -- -D warnings
cargo run -p mars-desktop -- path/to/notebook.mars
```
