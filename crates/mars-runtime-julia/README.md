# mars-runtime-julia

Julia execution backend for Project Mars notebooks with persistent sessions.

## Purpose

`mars-runtime-julia` implements the `RuntimeBackend` trait for Julia code execution. It uses a persistent Julia subprocess with a dedicated `MarsKernel` module for state management.

## Architecture

### Key Components

- **`session.rs`** — JuliaSession that spawns a long-lived `julia` subprocess with an embedded driver
- **Driver protocol** — Newline-delimited JSON protocol for communication between Rust and Julia
- **`MarsKernel` module** — Dedicated Julia module for isolated namespace and persistent bindings
- **Shadowed functions** — `display()` and `readline()` are shadowed for Project Mars integration

### Execution Model

1. Spawn `julia --startup-file=no --color=no driver.jl` subprocess
2. Driver speaks newline-delimited JSON protocol
3. Parse cell sources with `Meta.parseall`
4. Evaluate expressions in `MarsKernel` module
5. Capture stdout/stderr via `ProtocolIO`
6. Send `display` parts for trailing expressions
7. Maintain `ans`/`ans_prev` for last results

### Driver Protocol

**Requests:**
- `execute` — Execute a cell
- `inspect` — Inspect variables
- `check` — Parse without evaluation
- `input_response` — Answer pending input request
- `shutdown` — Terminate driver

**Responses:**
- `stream` — stdout/stderr output
- `display` — Rich display output
- `mime` — MIME bundle output
- `input_request` — Request for user input
- `result` — Execution result
- `inspection` — Variable inspection result
- `check_result` — Parse check result

## Features

- **Persistent session** — Variables persist across cells in `MarsKernel` module
- **Interactive input** — `readline()` support via kernel input channel
- **Rich display** — MIME bundle support via `repr(MIME"...", x)`
- **Automatic display** — Trailing non-silent expressions display automatically
- **Variable inspection** — `inspect_variables` support
- **Diagnostics** — Parse checking without evaluation

## Interpreter Resolution

The runtime discovers Julia via:

1. The `MARS_JULIA` environment variable (takes precedence)
2. The `julia` command on PATH

## Requirements

- Julia 1.9 or later (for `Meta.parseall`, `findlast`, `redirect_stderr`)
- No Julia packages required (driver embeds minimal JSON parser/writer)

## Rules

- Never execute user code in the host process — always subprocess
- Timeout/cancellation kills the whole process tree; next cell starts fresh session
- Protocol writes must go to `REAL_STDOUT`, not redirected stdout
- `repr(MIME"...", x)` falls back to `text/plain` for unimplemented MIME types
- `execution_count` increments only on successful evaluation

## Testing

```bash
cargo test -p mars-runtime-julia   # requires julia on PATH; skips otherwise
cargo clippy -p mars-runtime-julia --all-targets -- -D warnings
```

## Limitations

- Timeout/cancellation kills the Julia process; next cell starts fresh session (state lost)
- Requires Julia 1.9+ for certain features
- Driver protocol is custom JSON-RPC subset

## Related Documentation

- [Julia runtime documentation](../../docs/10-runtime-julia.md)
- [Runtime architecture](../../docs/06-runtime.md)
- [AGENTS.md](./AGENTS.md) — Architecture rules and implementation details
