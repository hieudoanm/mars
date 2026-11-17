---
title: MCP Integration
description: Project Mars MCP server and client for AI assistants and automation
order: 11
author: Project Mars Team
tags: [mcp, model-context-protocol, tools, ai, automation]
---

# Model Context Protocol (MCP)

Project Mars ships an MCP server and client so AI assistants, IDEs, and automation
tools can inspect, edit, execute, and save `.mars` notebooks — and manage Mars
Lab datasets, training jobs, and model artefacts — through the standardized
Model Context Protocol.

## What is MCP?

The Model Context Protocol is an open standard that lets AI assistants and
tools interact with external systems through a unified interface. Project Mars
implements it to expose notebook and Mars Lab operations as callable tools over a
newline-delimited JSON-RPC 2.0 transport on stdio — the same framing as
`mars kernel serve`.

## Server

```bash
mars mcp serve [--runtime replay|evcxr] [--offline] \
    [--notebook <file.mars>] [--language <lang>]
```

Configure it as a stdio MCP server:

```json
{
  "mcpServers": {
    "mars": {
      "command": "mars",
      "args": ["mcp", "serve", "--offline"]
    }
  }
}
```

`--notebook` preloads a notebook into the session and `--language` sets the
default runtime for kernel tools. Protocol traffic uses stdout only;
diagnostics go to stderr.

## Tools

The server implements `initialize`, `ping`, `tools/list`, and `tools/call`.
Handler errors are reported as tool results with `isError: true`; malformed
requests use standard JSON-RPC error codes.

### Notebook and cell tools

| Tool                    | Purpose                                                   |
| ----------------------- | --------------------------------------------------------- |
| `mars_notebook_new`     | Create a `.mars` file with a starter Markdown cell        |
| `mars_notebook_open`    | Load a notebook and return a summary                      |
| `mars_notebook_read`    | Read metadata and cells (`cell_index`, `include_outputs`) |
| `mars_notebook_save`    | Persist session changes atomically                        |
| `mars_notebook_inspect` | Format version, language, dependencies, cell counts       |
| `mars_cell_add`         | Insert a `code`, `markdown`, or `raw` cell                |
| `mars_cell_update`      | Update a cell's source and/or type                        |
| `mars_cell_delete`      | Delete a cell by index                                    |
| `mars_cell_move`        | Move a cell to a new index                                |
| `mars_cell_execute`     | Execute one cell (`index`) or ad-hoc `source`             |
| `mars_notebook_execute` | Run all code cells, record outputs, and save              |
| `mars_cell_diagnose`    | Compile-check source without committing it                |

### Kernel tools

| Tool                    | Purpose                   |
| ----------------------- | ------------------------- |
| `mars_kernel_status`    | Kernel execution status   |
| `mars_kernel_reset`     | Reset the runtime session |
| `mars_kernel_variables` | List live bindings        |

Kernel tools accept an optional `path`; omitting it uses a
notebook-independent kernel keyed by `language`. Kernels are created lazily
per `(notebook, language)` pair and pick up the notebook's declared
dependencies and `cargo_lock` snapshot, so Rust, Python, R, and Julia
sessions behave like `mars kernel serve`.

### Mars Lab tools

The server also exposes Mars Lab — datasets, training jobs, and model
artefacts backed by the same SQLite database as `mars lab`:

| Tool                      | Purpose                                                                                                                                   |
| ------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| `mars_lab_algorithms`     | List the algorithm catalogue (`task` filter)                                                                                              |
| `mars_lab_dataset_upload` | Upload a CSV file as a dataset (`path`)                                                                                                   |
| `mars_lab_datasets`       | List stored datasets                                                                                                                      |
| `mars_lab_dataset`        | Dataset record, stored insight, and computed profile                                                                                      |
| `mars_lab_train`          | Train a model and export ONNX (`dataset_id`, `task`, `algorithm`, `features`, optional `label`, `hyperparameters`, `split_ratio`, `seed`) |
| `mars_lab_jobs`           | List training jobs (`limit`)                                                                                                              |
| `mars_lab_job`            | Job record, metrics, and artefacts                                                                                                        |
| `mars_lab_export`         | Copy an artefact (`onnx`/`pkl`/`joblib`) to `output`                                                                                      |

Lab tools share the `mars-lab` configuration (`DATABASE_URL`,
`MARS_LAB_DATA_DIR`), so agents see the same data as the CLI and desktop
app. Training runs synchronously inside the tool call.

## Client

```bash
mars tools list [--server "<command>"]
mars tools call <name> '<json-args>' [--server "<command>"]
```

By default the client spawns this same `mars` binary as `mars mcp serve`.
`--server` accepts a whitespace-separated command line for pointing at
another MCP server.

```bash
mars tools call mars_notebook_new '{"path":"analysis.mars"}'
mars tools call mars_cell_execute '{"path":"analysis.mars","index":1}'
```

Each `tools call` is a fresh server process, so tools work against the files
on disk: cell mutations auto-save atomically and read/execute tools
auto-open notebooks they have not seen before.

## Example session

```text
$ mars tools call mars_notebook_new '{"path":"demo.mars"}'
$ mars tools call mars_cell_add '{"path":"demo.mars","cell_type":"code","source":"println!(\"hi\");"}'
$ mars tools call mars_cell_execute '{"path":"demo.mars","index":1}'
→ {"status":"succeeded","response":{"outputs":[{"type":"text","text":"hi\n"}]}}
```

## Architecture

The implementation lives in `crates/mars-mcp`:

- `jsonrpc` — request/response envelopes and line I/O
- `mcp` — protocol constants and `Tool`/`CallToolResult` types
- `session` — open notebooks with dirty tracking and per-language kernels
- `tools` — the `tools/list` catalog with JSON schemas
- `handlers` / `exec` — document and execution tool handlers
- `lab` — Mars Lab tool handlers with a lazily opened database
- `server` / `client` — stdio serve loop and client primitives

No MCP SDK is used; the required subset is implemented directly on the
existing `serde_json` infrastructure, keeping the dependency surface small.

## Limitations

- The server loop is sequential: an execution request blocks until the cell
  finishes (bounded by `timeout_ms`).
- Interactive stdin prompts (`input()`) are not supported over MCP yet.
- Mid-execution cancellation is not exposed; use per-call timeouts.
- Stdio transport only — no WebSocket or TCP MCP transport.
