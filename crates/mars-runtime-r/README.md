# mars-runtime-r

R execution backend for Project Mars notebooks with persistent sessions.

## Purpose

`mars-runtime-r` implements the `RuntimeBackend` trait for R code execution. It uses a persistent `Rscript --vanilla` subprocess with `.GlobalEnv` for state management.

## Architecture

### Key Components

- **`session.rs`** — RSession that spawns a long-lived `Rscript --vanilla` subprocess
- **Driver protocol** — Newline-delimited JSON protocol for communication between Rust and R
- **`.GlobalEnv` persistence** — Variables persist across cells in R's global environment
- **Plot capture** — Graphics device capture for PNG (or PDF fallback) images

### Execution Model

1. Spawn `Rscript --vanilla` subprocess
2. Driver speaks newline-delimited JSON protocol
3. Parse and evaluate cell sources in `.GlobalEnv`
4. Capture stdout per cell
5. Stream warnings and `message()` calls as stderr
6. Display trailing visible expression via `print()`
7. Capture plots drawn on graphics device as PNG (or PDF fallback)

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

- **Persistent session** — Variables persist across cells in `.GlobalEnv`
- **Plot capture** — Graphics device plots captured as PNG (or PDF fallback)
- **Warnings streaming** — Warnings and `message()` calls stream as stderr
- **Automatic display** — Trailing visible expression displays via `print()`
- **Variable inspection** — `inspect_variables` support
- **Diagnostics** — Parse checking without evaluation

## Interpreter Resolution

The runtime discovers R via:

1. The `MARS_RSCRIPT` environment variable (takes precedence)
2. The `Rscript` command on PATH
3. The `R` command on PATH (fallback)

## Requirements

- R 4.0 or later
- `jsonlite` R package for data interchange

Install jsonlite:

```r
install.packages("jsonlite")
```

Or via system package manager:

```bash
# Ubuntu/Debian
sudo apt install r-cran-jsonlite

# Fedora
sudo dnf install R-jsonlite
```

## Rules

- Never execute user code in the host process — always subprocess
- Timeout/cancellation kills the R process; next cell starts fresh session (state lost)
- `readline()`/`scan()` are unsupported and consume protocol lines
- `cat(..., file = stderr())` bypasses stream capture
- Requires `jsonlite` package to be installed

## Testing

```bash
cargo test -p mars-runtime-r   # requires Rscript + jsonlite on PATH
cargo clippy -p mars-runtime-r --all-targets -- -D warnings
```

## Limitations

- Timeout/cancellation kills the R process; next cell starts fresh session (state lost)
- `readline()`/`scan()` are unsupported
- Requires `jsonlite` package to be installed
- No automatic dependency management via notebook metadata

## Related Documentation

- [R runtime documentation](../../docs/09-r-runtime.md)
- [Runtime architecture](../../docs/06-runtime.md)
