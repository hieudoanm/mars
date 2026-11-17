---
title: CLI
description: Project Mars cli documentation
order: 4
author: Project Mars Team
tags: [cli, commands, import, export, kernel]
---

# CLI reference

The `mars` CLI is the headless interface to the document, runtime, renderer,
and Jupyter conversion crates. Commands return a nonzero exit code when input
cannot be loaded, validation fails, execution fails, or conversion is not
supported.

## Command summary

```text
mars notebook new <file.mars> [language] [--template <name>] [--example <name>] [--language <lang>]
mars notebook templates
mars notebook doctor [file.mars]
mars notebook open <file.mars>
mars notebook inspect <file.mars> [--json]
mars notebook run <file.mars> [options]
mars notebook import <file.ipynb> [file.mars]
mars notebook export <file.mars> [ipynb|html|md] [output]
mars kernel [list|restart|serve]
mars mcp serve [--runtime replay|evcxr] [--offline] [--notebook <file.mars>] [--language <lang>]
mars tools [list|call <name> [json] [--server <cmd>]]
```

## Create and inspect notebooks

```bash
mars notebook new analysis.mars rust    # Rust (default)
mars notebook new analysis.mars python  # Python
mars notebook new analysis.mars r       # R
mars notebook new analysis.mars --template ml
mars notebook new analysis.mars --example rust/02-machine-learning/machine-learning
mars notebook inspect analysis.mars
mars notebook inspect analysis.mars --json
```

`mars notebook new` creates a `.mars` document — a starter Markdown cell by
default, or a full template with `--template rust|python|ml|dashboard`, or a
copy of a shipped example with `--example <name>` (`mars notebook templates`
lists them). `mars notebook inspect` prints a summary of the notebook's
metadata, cells, and stored outputs without executing it; `--json` emits a
machine-readable summary for scripts and CI.

`mars notebook doctor [file.mars]` checks which language runtimes are
installed (`cargo`, `python3`, `Rscript`, `julia`, `clang++`/`g++`) and
prints versions or install hints — pass a notebook path to check only the
languages it uses.

## Run a notebook

```bash
mars notebook run analysis.mars
mars notebook run analysis.mars --timeout-ms 5000
mars notebook run analysis.mars --locked --offline --deterministic --report
```

By default, a successful run writes execution counts and outputs back to the
input notebook. A failed run writes the partial state available before the
failing cell. `--no-save` disables this write and is appropriate for read-only
checks or CI probes.

`mars notebook run` saves partial outputs when execution fails or is interrupted and
returns a nonzero exit code.

### Multi-language execution

Notebooks specify their language in `metadata.language`. Cells can override this
with `metadata.language` at the cell level. Project Mars maintains one runtime session
per language:

```bash
mars notebook run analysis.mars  # Runs all cells with their respective language runtimes
```

### Execution options

|| Option | Meaning |
|| ------------------ | -------------------------------------------------------------- |
|| `--timeout-ms <n>` | Bound compilation and execution for each cell |
|| `--locked` | Require the embedded Cargo lockfile for dependency-backed runs |
|| `--offline` | Prevent Cargo registry and network access |
|| `--deterministic` | Require a lockfile and apply locked execution policy |
|| `--no-save` | Do not write outputs or counts back to the notebook |
|| `--report` | Print a final execution summary |
|| `--changed-only` | Skip cells before the first code cell changed vs the git baseline |
|| `--since <ref>` | Diff base for `--changed-only` (implies it); default `HEAD` |
|| `--watch` | Re-run the notebook whenever the file changes |
|| `--watch-ms <n>` | Watch poll interval in milliseconds (default `1000`) |

`--deterministic` implies `--locked` and rejects dependency-backed notebooks
without `metadata.cargo_lock`. `--report` adds a final human-readable summary
line; it is not a versioned machine-readable protocol.

`--changed-only` diffs the notebook against `git show <ref>:<path>` using
stable cell IDs, then resumes execution at the first new or modified code
cell — everything from that point runs so downstream state stays consistent.
The notebook must be tracked by git; use `--since <ref>` to pick a different
base (`HEAD~3`, a branch, a tag).

`--watch` runs the notebook once, then polls the file every `--watch-ms`
milliseconds and re-runs on change — handy for live demos. Errors are
reported without stopping the loop; Ctrl+C exits.

## Import and export

```bash
mars notebook import notebook.ipynb analysis.mars
mars notebook export analysis.mars ipynb notebook.ipynb
mars notebook export analysis.mars html analysis.html
mars notebook export analysis.mars md analysis.md
```

Markdown export preserves Markdown cells, writes code cells as fenced `rust`
blocks, and converts outputs into portable Markdown:

- text and errors become fenced blocks
- tables become Markdown tables
- LaTeX becomes `$$` blocks
- images and offline charts become data URI images

`markdown` is accepted as an alias for `md`. When the output path is omitted,
Project Mars derives one from the input file name.

Jupyter import maps language kernelspecs to Project Mars language metadata:

- `ir` → `r`
- `python3` → `python`
- `rust` → `rust`

## Open a notebook

```bash
mars notebook open analysis.mars
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
Project Mars notebooks through the Model Context Protocol:

- `mcp serve` — Starts an MCP server speaking JSON-RPC 2.0 over stdio
- `tools list` — Lists all available MCP tools
- `tools call` — Invokes a specific MCP tool with JSON arguments

See [MCP integration documentation](/docs/mcp/) for detailed information on
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
mars notebook run examples/mars/rust/01-getting-started/getting-started.mars --no-save --report
```

Run a dependency-backed notebook using only its stored resolution:

```bash
mars notebook run analysis.mars --deterministic --offline --report
```

Generate default output paths:

```bash
mars notebook export analysis.mars html
mars notebook export analysis.mars ipynb
mars notebook export analysis.mars md
```

Run all examples:

```bash
cd examples/mars
./run.sh
```

## Diff, reports, and publishing

```bash
mars notebook diff analysis.mars HEAD~3   # cell-aware diff vs a git ref or .mars file
mars notebook run analysis.mars --changed-only --report --json-report
mars notebook export analysis.mars slides # self-contained slideshow HTML
```

`mars notebook diff` compares cells by stable IDs with line-level diffs.
`--json-report` emits a machine-readable run summary for CI. `slides`
exports a keyboard-navigable, print-friendly presentation. `html` export is
fully self-contained (inline CSS, embedded images) — publish it anywhere.

### SQL cells

Any code cell can run SQL by setting `cell.metadata.language` to `"sql"`
(the desktop language picker offers it directly). SQL cells execute in a
persistent in-process SQLite session — no external database required — and
support `SELECT`, DDL, and DML. Results render as notebook tables.

Tables are pre-loaded from CSV files declared in notebook metadata:

```json
"metadata": {
  "sql": {
    "tables": {
      "sales": "data/sales.csv",
      "customers": "data/customers.csv"
    }
  }
}
```

Paths resolve relative to the `.mars` file. Each named CSV becomes a SQL
table, so `SELECT * FROM sales` works immediately. Sessions persist across
cells (create tables, insert rows, query later); `mars notebook run` picks
up `metadata.sql.tables` automatically, as do `mars kernel serve` and the
MCP server.

### Scheduled runs

```cron
0 3 * * * cd /srv/notebooks && mars notebook run analysis.mars --changed-only --locked --report --json-report >> run.log 2>&1
```

`--changed-only` exits successfully when nothing changed, so scheduled runs
stay quiet until there is work to do.
