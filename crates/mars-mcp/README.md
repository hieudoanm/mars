# mars-mcp

Model Context Protocol (MCP) server and client for Project Mars notebooks.

## Purpose

`mars-mcp` provides MCP integration for Project Mars, enabling AI assistants (Claude Desktop, Cursor, etc.) to:

- Open and inspect Project Mars notebooks
- Edit notebook cells
- Execute code cells
- Get variable inspection results
- Get kernel diagnostics
- Save notebooks
- List available tools

## Architecture

The crate implements a hand-rolled JSON-RPC 2.0 subset over newline-delimited JSON, without using an MCP SDK. It includes:

- **`jsonrpc`** — Request/response envelopes, error codes, line I/O helpers
- **`mcp`** — Protocol constants, Tool, CallToolResult, initialize_result
- **`session`** — MarsSession: open notebooks with dirty tracking and per-(path, language) kernels
- **`tools`** — Tool definitions and JSON schemas
- **`handlers`** — Document and cell tool handlers
- **`exec`** — Execution and kernel tool handlers, maps KernelResponse to cell outputs
- **`server`** — MarsMcpServer dispatch and serve loops
- **`client`** — McpClient: spawn stdio MCP server, initialize handshake, list_tools, call_tool

## Dependencies

- `mars-core` — Shared types and IDs
- `mars-document` — Notebook format and persistence
- `mars-kernel` — Kernel and runtime integration
- `mars-protocol` — Protocol messages
- `serde` / `serde_json` — JSON serialization

## CLI Usage

### MCP Server

```bash
mars mcp serve --notebook analysis.mars --language rust
```

Options:
- `--runtime replay|evcxr` — Select Rust runtime backend
- `--offline` — Offline mode for dependency resolution
- `--notebook <file>` — Open specific notebook
- `--language <lang>` — Set notebook language

### Tool Listing

```bash
mars tools list
```

### Tool Execution

```bash
mars tools call mars_execute_cell --language rust '{"source":"let x = 42;"}'
```

## Tool Catalog

- `mars_open_notebook` — Open a notebook
- `mars_close_notebook` — Close a notebook
- `mars_list_cells` — List notebook cells
- `mars_get_cell` — Get a specific cell
- `mars_update_cell` — Update a cell
- `mars_append_cell` — Append a new cell
- `mars_delete_cell` — Delete a cell
- `mars_execute_cell` — Execute a cell
- `mars_inspect_variables` — Inspect kernel variables
- `mars_diagnostics` — Get kernel diagnostics
- `mars_save_notebook` — Save notebook

## Rules

- stdout carries only protocol JSON — diagnostics go to stderr
- Notebook mutations auto-save via atomic save
- Tool handler errors return CallToolResult with `isError: true`, not JSON-RPC errors
- Keep tool names `mars_*` and schemas in `tools.rs`

## Testing

```bash
cargo test -p mars-mcp
cargo clippy -p mars-mcp --all-targets -- -D warnings
```

## Related Documentation

- [MCP documentation](../../docs/18-mcp.md)
- [CLI documentation](../../docs/12-cli.md)
- [AGENTS.md](./AGENTS.md) — Architecture rules and module details
