# mars-runtime

Rust code execution engine for the Mars notebook platform.

## Purpose

`mars-runtime` provides persistent, interactive Rust execution sessions. It compiles cell-by-cell Rust programs, shares variables across cells, captures stdout/stderr, and supports cancellation and timeouts.

## Modules

- **`session`** — `RustSession`: create, execute, reset, query execution count and Cargo.lock.
- **`session_execution`** — Execution entry points (`execute`, `execute_with_timeout`, `execute_with_control`) and `RuntimeBackend` impl.
- **`compilation`** — `compile_program`, `compile_command`, `CompileOptions`.
- **`diagnostics`** — Compiler diagnostic formatting (`format_compiler_diagnostics`).
- **`manifest`** — Cargo manifest generation and dependency validation (`cargo_manifest`, `validate_dependencies`).
- **`execution`** — Child process execution with timeout/cancellation.
- **`expression`** — Program rendering for expression output.
- **`output`** — `ExecutionResult` and output parsing helpers.
- **`error`** — `RuntimeError`.
- **`backend`** — `RuntimeBackend` trait.
- **`cancellation`** — `CancellationToken`.
- **`toml`** — TOML dependency value parsing.

## Usage

```rust
use mars_runtime::RustSession;

let mut session = RustSession::new()?;
let result = session.execute("let x = 41; x + 1")?;
```
