# MCP (Model Context Protocol)

Project Mars exposes its notebook, kernel, and Mars Lab capabilities over MCP
so external agents (Claude Desktop, IDE assistants, custom tooling) can
inspect, edit, execute, and save `.mars` notebooks and manage datasets and
training jobs.

## Server

```bash
mars mcp serve [--runtime replay|evcxr] [--offline] [--notebook <file.mars>] [--language <lang>]
```

Speaks MCP over stdio: newline-delimited JSON-RPC 2.0 messages on
stdin/stdout (same framing as `mars kernel serve`). All diagnostics go to
stderr. Configure it in an MCP client as a stdio server, e.g.:

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

`--notebook` preloads a notebook into the session; `--language` sets the
default runtime language for kernel tools that do not name a notebook.

## Tools

Notebook/cell tools (auto-persist mutations atomically):

| Tool                                                                         | Purpose                                               |
| ---------------------------------------------------------------------------- | ----------------------------------------------------- |
| `mars_notebook_new`                                                          | Create a `.mars` file with a starter Markdown cell    |
| `mars_notebook_open`                                                         | Load a notebook and return a summary                  |
| `mars_notebook_read`                                                         | Read metadata/cells (`cell_index`, `include_outputs`) |
| `mars_notebook_save`                                                         | Persist session changes                               |
| `mars_notebook_inspect`                                                      | Format version, language, deps, cell counts           |
| `mars_cell_add` / `mars_cell_update` / `mars_cell_delete` / `mars_cell_move` | Cell editing                                          |
| `mars_cell_execute`                                                          | Execute one cell (`index`) or ad-hoc `source`         |
| `mars_notebook_execute`                                                      | Run all code cells, record outputs, save              |
| `mars_cell_diagnose`                                                         | Compile-check without committing                      |

Kernel tools (per notebook+language session, kernels persist across calls):

| Tool                    | Purpose               |
| ----------------------- | --------------------- |
| `mars_kernel_status`    | Execution status      |
| `mars_kernel_reset`     | Fresh runtime session |
| `mars_kernel_variables` | Live bindings         |

`path` is optional on kernel tools; omitting it uses a notebook-independent
kernel session keyed by `language`.

Mars Lab tools (datasets, training jobs, and artefacts backed by the Lab
SQLite database — same data as `mars lab`):

| Tool                      | Purpose                                                                                                                                   |
| ------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| `mars_lab_algorithms`     | List the algorithm catalogue, optional `task` filter                                                                                      |
| `mars_lab_dataset_upload` | Upload a CSV file as a dataset (`path`)                                                                                                   |
| `mars_lab_datasets`       | List stored datasets                                                                                                                      |
| `mars_lab_dataset`        | Dataset record + stored insight + computed profile                                                                                        |
| `mars_lab_train`          | Train a model (`dataset_id`, `task`, `algorithm`, `features`, optional `label`, `hyperparameters`, `split_ratio`, `seed`) and export ONNX |
| `mars_lab_jobs`           | List training jobs (`limit`)                                                                                                              |
| `mars_lab_job`            | Job record + metrics + artefacts                                                                                                          |
| `mars_lab_export`         | Copy a job artefact (`onnx`/`pkl`/`joblib`) to `output`                                                                                   |

Lab tools share the `mars-lab` settings (`DATABASE_URL`, `MARS_LAB_DATA_DIR`),
so MCP clients see the same datasets and jobs as the CLI and desktop app.
Training runs synchronously inside the tool call.

## Client

```bash
mars tools list [--server "<command>"]
mars tools call <name> '<json-args>' [--server "<command>"]
```

By default the client spawns the same `mars` binary as `mars mcp serve`.
`--server` takes a whitespace-separated command line (no quoting support).
Each `tools call` is a fresh server process, so tools must work against the
files on disk — cell mutations auto-save, and read/execute tools auto-open
notebooks they have not seen before.

## Limitations

- The server loop is sequential: an `Execute` blocks until the cell finishes
  (bounded by `timeout_ms`). Interactive stdin prompts (`input()`) are not
  supported over MCP yet.
- Cancellation mid-execution is not exposed as a tool; use `timeout_ms`.

### Streamable HTTP

`mars mcp serve --http 127.0.0.1:8080` also accepts `--http <addr>` (and `--token <token>`, or the
`MCP_AUTH_TOKEN` environment variable) to serve the same MCP surface over
streamable HTTP at `http://<addr>/mcp` — the transport used by ChatGPT
connectors and remote Claude clients. When a token is configured, every
request must carry `Authorization: Bearer <token>`.
