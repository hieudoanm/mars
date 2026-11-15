# mars-lsp

Minimal rust-analyzer LSP client for Project Mars notebook sessions.

## Purpose

`mars-lsp` provides Language Server Protocol (LSP) integration with rust-analyzer for the Project Mars desktop application. It enables rich code editing features including:

- Diagnostics
- Hover information
- Code completion
- Signature help
- Go-to-definition
- Quick fixes
- Inlay hints
- Semantic tokens

## Architecture

The crate implements a minimal LSP client that:

- Communicates with rust-analyzer via stdio
- Sends workspace and document notifications
- Receives diagnostics and completion results
- Translates LSP responses to Project Mars UI commands

## Dependencies

- `mars-runtime` — Runtime session integration
- `serde` / `serde_json` — LSP message serialization
- `uuid` — Document and session identification
- `thiserror` — Error handling

## Usage

The LSP client is used by the Project Mars desktop application to provide Rust code editing features. It is not exposed as a standalone CLI tool.

## Testing

```bash
cargo test -p mars-lsp
cargo clippy -p mars-lsp --all-targets -- -D warnings
```

## Related Documentation

- [Architecture documentation](../../docs/04-architecture.md)
- [Development documentation](../../docs/13-development.md)
- [Runtime documentation](../../docs/07-rust-runtime.md)
