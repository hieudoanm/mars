# 13 — MCP (Model Context Protocol)

Project Mars exposes its notebook and kernel capabilities over MCP so external agents
(Claude Desktop, IDE assistants, custom tooling) can inspect, edit, execute,
and save `.mars` notebooks.

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
