# mars-cli

Command-line interface for the Project Mars notebook platform.

## Purpose

`mars-cli` provides the `mars` executable for headless notebook creation, execution, export, and inspection without the desktop application.

## Commands

- `mars new <file>` — Create a new notebook.
- `mars open <file>` — Open/inspect a notebook.
- `mars run <file>` — Execute a notebook headless.
- `mars export <file> <format> <out>` — Export to HTML, Markdown, PDF, or ipynb.
- `mars import <file>` — Import a Jupyter notebook.
- `mars inspect <file>` — Show notebook metadata.
- `mars kernel list` — List available kernels.
- `mars kernel restart` — Restart the runtime.

## Usage

```bash
mars new analysis.mars
mars run analysis.mars
mars export analysis.mars html analysis.html
mars import notebook.ipynb
```

The binary is named `mars` and exits with meaningful codes (0 on success, non-zero on error).
