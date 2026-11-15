# mars-runtime-python

Python execution backend for Project Mars notebooks with persistent sessions.

## Purpose

`mars-runtime-python` implements the `RuntimeBackend` trait for Python code execution. It uses a persistent `python3 -u` subprocess with a shared module namespace for state management.

## Architecture

### Key Components

- **`session.rs`** — PythonSession that spawns a long-lived `python3 -u driver.py` subprocess
- **Driver protocol** — Newline-delimited JSON protocol for communication between Rust and Python
- **Shared module namespace** — Variables persist across cells in the same namespace
- **Builtin functions** — `display()` and `input()` are implemented as driver builtins

### Execution Model

1. Spawn `python3 -u driver.py` subprocess
2. Driver speaks newline-delimited JSON protocol
3. Execute cell sources in shared module namespace
4. Capture stdout/stderr through file-like Channel objects
5. Replace `sys.stdin` to prevent protocol line consumption
6. Support `input()` via kernel input channel
7. Display trailing expression with `_repr_*` support
8. Capture matplotlib figures as PNG (Agg backend)

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

- **Persistent session** — Variables persist across cells in shared module namespace
- **Interactive input** — `input()` support via kernel input channel
- **Rich display** — `_repr_html_`/`_repr_png_`/`_repr_jpeg_`/`_repr_svg_` support
- **Automatic display** — Trailing expression displays automatically
- **Matplotlib capture** — Figures captured as PNG (Agg backend)
- **Shell escapes** — `!command` for shell commands
- **Package installation** — `%pip`/`%pip3` magic for package installation
- **Top-level await** — Support for `await`/`async for`/`async with` at top level
- **Variable inspection** — `inspect_variables` support
- **Diagnostics** — Parse checking without evaluation

## Interpreter Resolution

The runtime discovers Python via:

1. The `MARS_PYTHON` environment variable (takes precedence)
2. The `python3` command on PATH
3. The `python` command on PATH (fallback)
4. The `py` command on PATH (fallback)

## Requirements

- Python 3.8 or later
- No Python packages required (driver is self-contained)
- For matplotlib: install matplotlib in your Python environment

## Rules

- Never execute user code in the host process — always subprocess
- Timeout/cancellation kills the whole process tree; next cell starts fresh session (state lost)
- stderr is a non-fatal text stream (warnings do not fail cells)
- `execution_count` increments only on successful evaluation
- Shell escapes `%` magics other than `%pip` are rejected

## Testing

```bash
cargo test -p mars-runtime-python   # requires python3 on PATH
cargo clippy -p mars-runtime-python --all-targets -- -D warnings
```

## Limitations

- Timeout/cancellation kills the interpreter; next cell starts fresh session (state lost)
- No automatic dependency management via notebook metadata
- `readline()`/`scan()` consume protocol lines and are unsupported

## Related Documentation

- [Python runtime documentation](../../docs/08-python-runtime.md)
- [Runtime architecture](../../docs/06-runtime.md)
- [AGENTS.md](./AGENTS.md) — Architecture rules and implementation details
