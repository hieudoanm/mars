# mars-lsp — LSP Client for Project Mars

Minimal rust-analyzer LSP client for Project Mars notebook sessions. Depends on
`mars-runtime`.

## Architecture

- `LspClient` (`client.rs`) spawns a rust-analyzer subprocess with stdio
  transport and manages its lifecycle.
- `protocol.rs` defines LSP message types (requests, responses, notifications)
  matching the Language Server Protocol specification.
- `transport.rs` handles newline-delimited JSON I/O for LSP communication.
- The client sends workspace initialization, document open/change notifications,
  and receives diagnostics, completion, hover, and other results.
- Responses are translated to Project Mars-specific UI commands (diagnostics, hover,
  completion) for the desktop application.

## Key Modules

- `client.rs` — LspClient spawning, lifecycle, and request handling
- `protocol.rs` — LSP message type definitions
- `transport.rs` — JSON-RPC transport layer
- `types.rs` — Shared type definitions for LSP messages

## Rules

- Never execute user code in the host process — rust-analyzer runs in a
  subprocess.
- Timeout and cancellation must terminate the rust-analyzer process.
- Keep protocol messages minimal and compatible with rust-analyzer's JSON-RPC
  format.
- Diagnostics must map source positions correctly to notebook cell
  coordinates.
- No `unwrap()`/`expect()` in production paths; errors go through `LspError`.

## Testing

```bash
cargo test -p mars-lsp
cargo clippy -p mars-lsp --all-targets -- -D warnings
```

Line and function coverage must stay at or above 80% — verify with `cargo llvm-cov -p mars-lsp --tests` (workspace gate: `make coverage`).

## Integration

The LSP client is used by the Project Mars desktop application to provide Rust code
editing features. It is not exposed as a standalone CLI tool.
