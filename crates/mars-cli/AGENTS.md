# mars-cli — `mars` Command-Line Interface

The `mars` binary. Depends on all other crates; thin orchestration layer — no business logic beyond argument parsing and command dispatch.

## Commands (`commands.rs`)

- `mars notebook new <file.mars> [language] [--template rust|python|ml|dashboard] [--example <name>] [--language <lang>]` — creates a notebook saved atomically. Default is a starter Markdown cell; `--template` builds a starter notebook (`templates.rs`), `--example` copies a shipped `examples/mars/**.mars` notebook discovered from cwd ancestors or `MARS_EXAMPLES_DIR`. Positional language and `--language` are equivalent; the flag overrides template/example defaults.
- `mars notebook templates` — lists built-in templates and discovered examples.
- `mars notebook doctor [file.mars]` — probes runtime tools (`cargo`, `python3`, `Rscript`, `julia`, `clang++`/`g++`) and reports versions/install hints (`doctor.rs`); with a notebook path it checks only used languages and fails when one is missing.
- `mars notebook inspect <file.mars> [--json]` — prints format version, language, cell count; `--json` emits a machine-readable summary.
- `mars notebook open <file.mars>` — loads the notebook then opens it with the OS handler (`util::open_with_system`).
- `mars notebook run <file.mars> [opts]` — executes code cells sequentially through `create_runtime`, forwarding process stdin to the active language's `StdinWriter` so `input()`/`mars_prompt` work interactively (stdin EOF raises `EOFError` rather than hanging), records `execution_count`/`outputs` on cells, captures the first `Cargo.lock` into `metadata.cargo_lock` when dependencies exist, saves results by default. Options (`options.rs`): `--runtime replay|evcxr`, `--timeout-ms`, `--offline`, `--locked`, `--deterministic` (implies `--locked`, requires `metadata.cargo_lock`), `--no-save`, `--report`, `--changed-only`, `--since <git-ref>` (implies `--changed-only`; diffs cells by stable `CellId` via `git show` — `changes.rs` — and resumes at the first new/modified code cell), `--watch`/`--watch-ms` (re-run on file change, polling mtime+size).
- `mars notebook diff <file.mars> [ref|file]` — cell-aware diff matched by stable cell IDs with line-level hunks (`changes.rs`).
- `mars notebook import <file.ipynb> [file.mars]` / `mars notebook export <file.mars> [ipynb|html|slides|md|pdf] [--page-size=...] [output]` — Jupyter and renderer-backed export; `slides` is a self-contained keyboard-navigable deck (`mars-renderer::slides`).
- `mars kernel list|restart|serve` — kernels are per-process; `serve` runs a JSON-lines protocol loop on stdin/stdout (`kernel.rs::serve_kernel`) so external tools can drive `Kernel` via `mars-protocol`. The server keeps one `Kernel` per language (`Execute.language` selects/creates it), answers `Input` requests through backend `StdinWriter`s so prompts work mid-execution, resets all kernels on `reset`, and aggregates `status` across languages.
- `mars mcp serve` — MCP (Model Context Protocol) server on stdio (`mcp.rs::serve_mcp` → `mars-mcp`). Exposes notebook/cell/kernel operations as `tools/call` tools; see `docs/02-platform/mcp.md`.
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
cargo run -p mars -- notebook run examples/getting-started.mars
```

Line and function coverage must stay at or above 80% — verify with `cargo llvm-cov -p mars --tests` (workspace gate: `make coverage`).
