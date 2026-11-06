# mars-desktop

Pure-Rust desktop client for Mars built on GPUI (`gpui-pre`). See `AGENTS.md` for architecture and rules.

## Run

```bash
cargo run -p mars-desktop                 # new notebook
cargo run -p mars-desktop -- path/to/file.mars
```

Requires the macOS Metal Toolchain (`xcodebuild -downloadComponent MetalToolchain`) on macOS.

## Features

- Dark Mars-themed notebook UI with Phosphor icons (`gpui-kit-phosphor`)
- Toolbar: New, Open, Save, add Code/Markdown cells, Run all, Restart kernel
- Per-cell controls: run, move up/down, toggle cell type, delete
- Scrollable cell list with plain-text outputs and kernel status bar
- Native file dialogs; kernel execution off the UI thread via `mars-kernel`
- `MARS_EXPERIMENTAL_RUNTIME=evcxr` selects the experimental backend
