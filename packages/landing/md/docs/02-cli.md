---
title: CLI
description: Mars cli documentation
order: 2
author: Mars Team
tags: [cli, commands, import, export, kernel]
---

# CLI reference

The `mars` CLI is the headless interface to the document, runtime, renderer,
and Jupyter conversion crates. Commands return a nonzero exit code when input
cannot be loaded, validation fails, execution fails, or conversion is not
supported.

## Command summary

```text
mars new <file.mars> [language]
mars open <file.mars>
mars inspect <file.mars>
mars run <file.mars> [options]
mars import <file.ipynb> [file.mars]
mars export <file.mars> [ipynb|html|md] [output]
mars kernel [list|restart|serve]
mars mcp serve [--runtime replay|evcxr] [--offline] [--notebook <file.mars>] [--language <lang>]
mars tools [list|call <name> [json] [--server <cmd>]]
```

## Create and inspect notebooks

```bash
mars new analysis.mars rust    # Rust (default)
mars new analysis.mars python  # Python
mars new analysis.mars r       # R
mars inspect analysis.mars
```

`mars new` creates a `.mars` document with an initial Markdown cell.
`mars inspect` prints a summary of the notebook's metadata, cells, and stored
outputs without executing it.

## Run a notebook

```bash
mars run analysis.mars
mars run analysis.mars --timeout-ms 5000
mars run analysis.mars --locked --offline --deterministic --report
```

By default, a successful run writes execution counts and outputs back to the
input notebook. A failed run writes the partial state available before the
failing cell. `--no-save` disables this write and is appropriate for read-only
checks or CI probes.

`mars run` saves partial outputs when execution fails or is interrupted and
returns a nonzero exit code.

### Multi-language execution

Notebooks specify their language in `metadata.language`. Cells can override this
with `metadata.language` at the cell level. Mars maintains one runtime session
per language:

```bash
mars run analysis.mars  # Runs all cells with their respective language runtimes
```

### Execution options

|| Option             | Meaning                                                        |
|| ------------------ | -------------------------------------------------------------- |
|| `--timeout-ms <n>` | Bound compilation and execution for each cell                  |
|| `--locked`         | Require the embedded Cargo lockfile for dependency-backed runs |
|| `--offline`        | Prevent Cargo registry and network access                      |
|| `--deterministic`  | Require a lockfile and apply locked execution policy           |
|| `--no-save`        | Do not write outputs or counts back to the notebook            |
|| `--report`         | Print a final execution summary                                |

`--deterministic` implies `--locked` and rejects dependency-backed notebooks
without `metadata.cargo_lock`. `--report` adds a final human-readable summary
line; it is not a versioned machine-readable protocol.

## Import and export

```bash
mars import notebook.ipynb analysis.mars
mars export analysis.mars ipynb notebook.ipynb
mars export analysis.mars html analysis.html
mars export analysis.mars md analysis.md
```

Markdown export preserves Markdown cells, writes code cells as fenced `rust`
blocks, and converts outputs into portable Markdown:

- text and errors become fenced blocks
- tables become Markdown tables
- LaTeX becomes `$$` blocks
- images and offline charts become data URI images

`markdown` is accepted as an alias for `md`. When the output path is omitted,
Mars derives one from the input file name.

Jupyter import maps language kernelspecs to Mars language metadata:
- `ir` → `r`
- `python3` → `python`
- `rust` → `rust`

## Open a notebook

```bash
mars open analysis.mars
```

This opens the notebook with the platform default application.

## Kernel lifecycle

```bash
mars kernel list
mars kernel restart
mars kernel serve [--runtime replay|evcxr] [--language rust|python|r|julia] [--notebook <file.mars>]
mars kernel serve --tcp <port> [--tcp-host <addr>]
```

`mars kernel serve` runs a persistent newline-delimited JSON transport. Each
input line is a serialized `RequestEnvelope`; the corresponding output line is
a serialized `ResponseEnvelope`.

Pass `--notebook` to load the notebook's declared Cargo dependencies and
lockfile into the kernel. Without it, the kernel starts with an empty dependency
set.

The `--tcp` option starts a TCP server (binds to `127.0.0.1` by default for
security). Each TCP connection gets its own kernel session.

## MCP (Model Context Protocol)

```bash
mars mcp serve [--runtime replay|evcxr] [--offline] [--notebook <file.mars>] [--language <lang>]
mars tools list [--server <cmd>]
mars tools call <name> [json-args] [--server <cmd>]
```

The MCP commands allow AI assistants and automation tools to interact with
Mars notebooks through the Model Context Protocol:

- `mcp serve` — Starts an MCP server speaking JSON-RPC 2.0 over stdio
- `tools list` — Lists all available MCP tools
- `tools call` — Invokes a specific MCP tool with JSON arguments

See [MCP integration documentation](/docs/07-mcp/) for detailed information on
the MCP protocol, available tools, and AI assistant integration.

## Runtime selection

The `--runtime` flag selects the execution backend:

- `replay` (default): Re-executes all previous cells for each new cell, providing
  isolation and reproducibility
- `evcxr`: Experimental Evcxr backend with faster re-execution but limited
  dependency support

## Examples

Run a notebook without changing it:

```bash
mars run examples/mars/rust/01-getting-started/getting-started.mars --no-save --report
```

Run a dependency-backed notebook using only its stored resolution:

```bash
mars run analysis.mars --deterministic --offline --report
```

Generate default output paths:

```bash
mars export analysis.mars html
mars export analysis.mars ipynb
mars export analysis.mars md
```

Run all examples:

```bash
cd examples/mars
./run.sh
```
