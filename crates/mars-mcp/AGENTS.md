# mars-mcp — MCP Server & Client

Model Context Protocol integration. Depends on `mars-core`, `mars-document`,
`mars-kernel`, `mars-lab`, `mars-protocol`. No MCP SDK — the required subset is
hand-rolled JSON-RPC 2.0 over newline-delimited JSON (same framing as
`mars kernel serve`).

## Modules

- `jsonrpc` — request/response envelopes, error codes, line I/O helpers.
- `mcp` — protocol constants (`PROTOCOL_VERSION`), `Tool`,
  `CallToolResult`, `initialize_result`.
- `session` — `MarsSession`: open notebooks (`HashMap<PathBuf, NotebookEntry>`
  with dirty tracking) and per-`(path, language)` `Kernel`s created lazily
  with notebook dependencies/`cargo_lock`.
- `tools` — `definitions()`: the `tools/list` catalog (names + JSON schemas).
- `handlers` — document/cell tool handlers.
- `exec` — execution and kernel tool handlers; maps `KernelResponse` onto
  cell `outputs`/`execution_count`.
- `lab` — Mars Lab tool handlers (`mars_lab_*`): datasets, training jobs,
  artefacts. `LabBackend` owns a current-thread tokio runtime and a lazily
  opened `mars_lab::db::Database` shared with `mars lab` settings.
- `server` — `MarsMcpServer` dispatch (`initialize`, `ping`, `tools/list`,
  `tools/call`, notifications ignored) and `serve`/`serve_stdio` loops.
- `client` — `McpClient`: spawn a stdio MCP server, `initialize` handshake,
  `list_tools`, `call_tool`.

## Rules

- stdout carries only protocol JSON — diagnostics go to stderr.
- Notebook mutations auto-save via `mars_document::save_atomic`; notebooks
  auto-open from disk so one-shot clients (`mars tools call`, which spawns a
  fresh server per invocation) work correctly.
- Tool handler errors return `CallToolResult` with `isError: true`, not
  JSON-RPC errors; protocol-level failures use JSON-RPC error codes.
- Keep tool names `mars_*` and schemas in `tools.rs` — adding a tool means
  definition + handler + tests.

## Testing

```bash
cargo test -p mars-mcp
cargo clippy -p mars-mcp --all-targets -- -D warnings
```
