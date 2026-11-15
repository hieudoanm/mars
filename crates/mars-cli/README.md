# mars-cli

Command-line interface for the Project Mars notebook platform.

## Purpose

`mars-cli` provides the `mars` executable for headless notebook creation, execution, export, and inspection without the desktop application.

## Commands

- `mars notebook new <file>` — Create a new notebook.
- `mars notebook open <file>` — Open/inspect a notebook.
- `mars notebook run <file>` — Execute a notebook headless.
- `mars notebook export <file> <format> <out>` — Export to HTML, Markdown, PDF, or ipynb.
- `mars notebook import <file>` — Import a Jupyter notebook.
- `mars notebook inspect <file>` — Show notebook metadata.
- `mars kernel list` — List available kernels.
- `mars kernel restart` — Restart the runtime.

## Usage

```bash
mars notebook new analysis.mars
mars notebook run analysis.mars
mars notebook export analysis.mars html analysis.html
mars notebook import notebook.ipynb
```

The binary is named `mars` and exits with meaningful codes (0 on success, non-zero on error).
