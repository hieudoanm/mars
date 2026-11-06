# mars-desktop — GPUI Desktop Application

Pure-Rust desktop client for Mars built on GPUI via the `gpui-pre` family of crates.
Binary: `mars-desktop`.

This crate replaces the earlier Tauri+React approach. The Tauri app still lives at `apps/desktop/` (package `mars-tauri`) — do not confuse the two.

## Dependencies

- `gpui` — alias for `gpui-pre = "=0.3.9"` (workspace dep in root `Cargo.toml`). Do not mix with the crates.io `gpui` crate; types are incompatible.
- `gpui-platform` — alias for `gpui-pre-platform = "=0.3.9"`; provides `application()` since `gpui-pre` has no built-in platform backend.
- `gpui-kit-phosphor = "=0.1.0"` — Phosphor icons. Registered via `PhosphorAssets` in `main.rs` (`application().with_assets(PhosphorAssets)`). `PhosphorIcon`/`Phosphor` render directly as elements, sized by the surrounding text size and colored by text color; default features include the `regular` weight only — enable `bold`/`fill`/etc. features if other weights are needed.

## Architecture

- `src/main.rs` — `gpui_platform::application()` bootstrap with `PhosphorAssets`, key bindings (`cmd-n/o/s`, `cmd-shift-enter`, `cmd-r`, `cmd-q`), window creation, optional notebook path from `argv[1]`.
- `src/app.rs` — `NotebookApp` view: toolbar, scrollable cell list, status bar. Owns the `Notebook`, file `path`, and an `Option<Arc<Mutex<Kernel>>>` (kernel creation/configuration failure is surfaced in the status bar, not `expect`ed).
- `src/output_text.rs` — flattens `mars_core::Output` into plain-text lines for display.

## Rules

- All kernel access goes through `mars-kernel` (`handle_with_cancellation`, `Reset`, `Status`); never execute notebook code in the UI process beyond the kernel/runtime isolation.
- Kernel execution runs on `cx.background_executor()` inside `cx.spawn`; UI state is only mutated back on the main thread via `WeakEntity::update`.
- Notebook persistence uses `mars_document::{load, save_atomic}`; kernel (re)configuration uses `Kernel::with_backend` with `RuntimeBackendKind::from_environment()` (`MARS_EXPERIMENTAL_RUNTIME=evcxr` selects the experimental backend).
- Cell operations go through `Notebook::{insert_cell, remove_cell, move_cell, cell_mut}` — never mutate `notebook.cells` directly.
- File dialogs use GPUI's `prompt_for_paths` / `prompt_for_new_path` — no external dialog crates.
- Keep the dark Mars palette in the `const` colors at the top of `app.rs`.
- Avoid `unwrap`/`expect` in production paths (kernel creation, window open, file I/O all degrade to status-bar messages).

## GPUI gotchas (learned the hard way)

- `cx.spawn` needs direct `async move |this: WeakEntity<Self>, cx: &mut AsyncApp|` closures.
- Action handlers must be element-level `.on_action(cx.listener(...))` on a focused stateful element — `Context::on_action` outside the dispatch tree panics in `key_dispatch.rs`.
- `Window::focus` takes `(&FocusHandle, &mut App)`; pass `cx` from a `Context`.
- `cx.open_window` returns `Result` in `gpui-pre`; handle it instead of unwrapping.

## Current limitations (v2 scaffold)

- Cells are rendered read-only — no text editing yet (needs a GPUI input/editor element).
- Outputs render as plain text; rich outputs (HTML, images, LaTeX, charts) show placeholders.
- Cell add (code/markdown), delete, reorder (up/down), and Code↔Markdown/Raw toggle are implemented; there is no insert-between or duplicate.
- No markdown rendering, no LaTeX, no autosave, no per-cell keyboard shortcuts beyond the toolbar/global bindings.

## Testing

```bash
cargo check -p mars-desktop
cargo clippy -p mars-desktop --all-targets -- -D warnings
cargo run -p mars-desktop -- path/to/notebook.mars
```
