---
title: Getting Started
description: Mars getting-started documentation
order: 1
author: Mars Team
tags: [getting-started, install, cli, desktop, vscode]
---

# Getting started

Mars has three surfaces: a desktop application, a command-line interface, and
a VS Code extension. All of them use the same notebook format and execution
core.

## Install the CLI

From the repository root:

```bash
./scripts/install.sh --cli
```

The script builds a fresh release binary and installs `mars` to
`~/.local/bin`. Make sure that directory is on your `PATH`:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

You can also run the CLI directly from the workspace:

```bash
cargo run -p mars -- --help
```

## Create a notebook

```bash
mars new analysis.mars
```

This creates a `.mars` file with one Markdown cell. Open it in the desktop app
or VS Code, or edit the JSON directly.

## Run a notebook

```bash
mars run analysis.mars
```

By default, results are written back to the notebook. Use `--no-save` for a
read-only check:

```bash
mars run analysis.mars --no-save --report
```

Code cells execute in order and share bindings. A later cell can use a value
defined earlier:

```rust
let numbers = vec![1, 2, 3, 4, 5];
let total: i32 = numbers.iter().sum();
total
```

## Export a notebook

```bash
mars export analysis.mars html analysis.html
mars export analysis.mars md analysis.md
mars export analysis.mars ipynb analysis.ipynb
```

Markdown export preserves Markdown cells, fences Rust code, and converts
tables, LaTeX, images, and offline charts into portable Markdown.

## Install the desktop app and VS Code extension

```bash
./scripts/install.sh --desktop
./scripts/install.sh --cli --vscode
```

The VS Code extension executes cells through the `mars` CLI, so keep the CLI
installed unless it is already available on `PATH`.
