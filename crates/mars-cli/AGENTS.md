# mars-cli — `mars` Command-Line Interface

The `mars` binary. Depends on all other crates; thin orchestration layer — no business logic beyond argument parsing and command dispatch.

## Commands (`commands.rs`)

- `mars new <file.mars>` — creates a notebook with a starter Markdown cell, saved atomically.
- `mars inspect <file.mars>` — prints format version, language, cell count.
- `mars open <file.mars>` — loads the notebook then opens it with the OS handler (`util::open_with_system`).
- `mars run <file.mars> [opts]` — executes all code cells sequentially through `create_runtime`, forwarding process stdin to the active language's `StdinWriter` so `input()`/`mars_prompt` work interactively (stdin EOF raises `EOFError` rather than hanging), records `execution_count`/`outputs` on cells, captures the first `Cargo.lock` into `metadata.cargo_lock` when dependencies exist, saves results by default. Options (`options.rs`): `--runtime replay|evcxr`, `--timeout-ms`, `--offline`, `--locked`, `--deterministic` (implies `--locked`, requires `metadata.cargo_lock`), `--no-save`, `--report`.
- `mars import <file.ipynb> [file.mars]` / `mars export <file.mars> [ipynb|html|md|pdf] [--page-size=...] [output]` — Jupyter and renderer-backed export.
- `mars kernel list|restart|serve` — kernels are per-process; `serve` runs a JSON-lines protocol loop on stdin/stdout (`kernel.rs::serve_kernel`) so external tools can drive `Kernel` via `mars-protocol`. The server keeps one `Kernel` per language (`Execute.language` selects/creates it), answers `Input` requests through backend `StdinWriter`s so prompts work mid-execution, resets all kernels on `reset`, and aggregates `status` across languages.
- `mars mcp serve` — MCP (Model Context Protocol) server on stdio (`mcp.rs::serve_mcp` → `mars-mcp`). Exposes notebook/cell/kernel operations as `tools/call` tools; see `docs/13-mcp.md`.
- `mars tools list|call <name> [json] [--server <cmd>]` — MCP client commands (`mcp.rs::run_tools` → `mars_mcp::McpClient`). Defaults to spawning this binary as `mars mcp serve`.

## Rules

- `main` calls `mars_kernel::runtime_hook()` first — required for the Evcxr backend's re-executed process.
- Exit codes: `0` on success, `1` on any error (message printed as `mars: <error>`); execution failure mid-notebook is an error.
- Keep commands honest — do not advertise flags or subcommands without functional backing.
- All file writes go through `mars_document::save_atomic` where applicable.

## Testing

```bash
cargo test -p mars
cargo clippy -p mars --all-targets -- -D warnings
cargo run -p mars -- run examples/getting-started.mars
```
