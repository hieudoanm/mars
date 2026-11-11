# Mars Crates

This directory contains the Rust crates that form the core of the Mars notebook platform.

## Overview

Mars is organized as a modular Cargo workspace with crates for:

- **Core types and utilities** — Shared domain types, IDs, errors
- **Document management** — Notebook format, serialization, validation
- **Execution runtimes** — Multi-language execution backends
- **Kernel** — Session management and runtime dispatch
- **Protocol** — Communication between frontend and backend
- **Rendering** — Output rendering for notebooks
- **Integration** — Jupyter compatibility, LSP, MCP
- **CLI** — Command-line interface

## Crate List

### Core Crates

| Crate                               | Purpose                                                         |
| ----------------------------------- | --------------------------------------------------------------- |
| [`mars-core`](./mars-core/)         | Shared domain types, IDs, metadata, errors, execution status    |
| [`mars-document`](./mars-document/) | Notebook representation, serialization, validation, persistence |
| [`mars-protocol`](./mars-protocol/) | Communication between frontend and backend (kernel protocol)    |
| [`mars-renderer`](./mars-renderer/) | Markdown, HTML, LaTeX, tables, images, charts rendering         |

### Execution Crates

| Crate                                           | Purpose                                    |
| ----------------------------------------------- | ------------------------------------------ |
| [`mars-runtime`](./mars-runtime/)               | Rust execution (replay and Evcxr backends) |
| [`mars-runtime-evcxr`](./mars-runtime-evcxr/)   | Experimental Evcxr Rust backend            |
| [`mars-runtime-python`](./mars-runtime-python/) | Python execution backend                   |
| [`mars-runtime-r`](./mars-runtime-r/)           | R execution backend                        |
| [`mars-runtime-julia`](./mars-runtime-julia/)   | Julia execution backend                    |
| [`mars-runtime-cpp`](./mars-runtime-cpp/)       | C++ execution backend                      |

### Integration Crates

| Crate                             | Purpose                                                |
| --------------------------------- | ------------------------------------------------------ |
| [`mars-kernel`](./mars-kernel/)   | Kernel lifecycle, execution requests, runtime dispatch |
| [`mars-jupyter`](./mars-jupyter/) | Jupyter `.ipynb` import/export                         |
| [`mars-lsp`](./mars-lsp/)         | rust-analyzer LSP client for code editing              |
| [`mars-mcp`](./mars-mcp/)         | Model Context Protocol server for AI assistants        |

### Application Crates

| Crate                     | Purpose                         |
| ------------------------- | ------------------------------- |
| [`mars-cli`](./mars-cli/) | Command-line interface for Mars |

## Architecture

### Separation of Concerns

The crate architecture follows these principles:

- **Document management** — Separate from execution and rendering
- **Execution isolation** — Each runtime is an isolated subprocess
- **Protocol layer** — Typed messages for frontend-backend communication
- **Rendering layer** — Output rendering independent of execution
- **Integration points** — Jupyter, LSP, MCP as separate crates

### Dependency Graph

```
mars-core (shared types)
    ↓
mars-document (notebook format)
    ↓
mars-runtime (Rust execution)
    ↓
mars-kernel (runtime dispatch)
    ↓
mars-cli (user interface)

mars-runtime-python (Python)
mars-runtime-r (R)
mars-runtime-julia (Julia)
mars-runtime-cpp (C++)
mars-runtime-evcxr (Experimental Rust)

mars-protocol (communication)
mars-renderer (output rendering)

mars-jupyter (Jupyter compatibility)
mars-lsp (LSP integration)
mars-mcp (MCP integration)
```

## Common Patterns

### RuntimeBackend Trait

All language runtimes implement the `RuntimeBackend` trait from `mars-runtime`:

```rust
pub trait RuntimeBackend: Send + Sync {
    fn reset(&mut self) -> Result<(), RuntimeError>;
    fn execute(&mut self, source: &str) -> Result<CellOutput, RuntimeError>;
    fn inspect_variables(&mut self) -> Result<Vec<Variable>, RuntimeError>;
    fn diagnose(&mut self, source: &str) -> Result<Vec<Diagnostic>, RuntimeError>;
}
```

### Error Handling

All crates use `RuntimeError` from `mars-core` for consistent error reporting:

```rust
#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("Compilation failed: {0}")]
    Compilation(String),
    #[error("Execution failed: {0}")]
    Execution(String),
    // ...
}
```

### Subprocess Isolation

All runtime backends execute code in isolated subprocesses:
- Never execute user code in the host process
- Timeout and cancellation kill the whole process tree
- Stdout/stderr captured via protocol
- Stdin handled via kernel input channel

## Development

### Building All Crates

```bash
cargo build --workspace
```

### Testing All Crates

```bash
cargo test --workspace
```

### Coverage Gate

Crates under `crates/` must maintain at least **80% line and function
coverage**. From the repository root, run:

```bash
make coverage
```

This runs `cargo llvm-cov` with `--fail-under-lines 80` and
`--fail-under-functions 80`, and fails when coverage drops below either
threshold.

### Linting All Crates

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

### Formatting All Crates

```bash
cargo fmt --all
```

## AGENTS.md Files

Each crate has an `AGENTS.md` file documenting:

- Architecture and module breakdown
- Rules and constraints
- Testing instructions
- Integration points

See individual crate directories for detailed architecture documentation.

## Related Documentation

- [Architecture documentation](../docs/04-architecture.md)
- [Runtime documentation](../docs/06-runtime.md)
- [Development documentation](../docs/13-development.md)
- [Repository AGENTS.md](../AGENTS.md)

## Adding a New Crate

When adding a new crate to the workspace:

1. Create the crate directory with `cargo new --lib <name>`
2. Add it to the workspace members in the root `Cargo.toml`
3. Create a `README.md` documenting its purpose
4. Create an `AGENTS.md` documenting architecture and rules
5. Follow the naming convention: `mars-*`
6. Update this README with the new crate
7. Ensure it follows the common patterns (RuntimeBackend trait, error handling, etc.)

## Crate-Specific Documentation

For detailed information about each crate, see its individual README and AGENTS.md files:

- [mars-core](./mars-core/README.md)
- [mars-document](./mars-document/README.md)
- [mars-protocol](./mars-protocol/README.md)
- [mars-renderer](./mars-renderer/README.md)
- [mars-runtime](./mars-runtime/README.md)
- [mars-runtime-evcxr](./mars-runtime-evcxr/README.md)
- [mars-runtime-python](./mars-runtime-python/README.md)
- [mars-runtime-r](./mars-runtime-r/README.md)
- [mars-runtime-julia](./mars-runtime-julia/README.md)
- [mars-runtime-cpp](./mars-runtime-cpp/README.md)
- [mars-kernel](./mars-kernel/README.md)
- [mars-jupyter](./mars-jupyter/README.md)
- [mars-lsp](./mars-lsp/README.md)
- [mars-mcp](./mars-mcp/README.md)
- [mars-cli](./mars-cli/README.md)
