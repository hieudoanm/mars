# mars-runtime-r — R Execution Backend

Executes R code interactively with shared state across cells. Depends on
`mars-core` and `mars-runtime`.

## Architecture

- `RSession` (`session.rs`) implements `RuntimeBackend` by spawning a
  long-lived `Rscript --vanilla driver.R` subprocess (driver source embedded
  via `include_str!`, written to a per-session temp dir removed on `Drop`).
- The driver speaks newline-delimited JSON — the same protocol as the Python
  and Julia drivers. A dedicated `stdin_reader` thread dispatches work
  (`execute`/`inspect`/`check`) onto the exec loop while `input_response`
  messages go straight to a pending-input queue, so a cell blocked in
  `readline()` could be answered mid-execution (currently unsupported).
- Requests: `execute`, `inspect`, `check`, `input_response`, `shutdown`.
  Responses: `stream`, `display`, `mime`, `input_request`, `result`,
  `inspection`, `check_result`.
- User stdout/stderr are captured through `sink()` and `outputHandler()`
  functions and forwarded as stream events; warnings and `message()` calls
  go to stderr.
- Cell sources are parsed and evaluated in `.GlobalEnv`, so bindings persist
  across cells in the global environment.
- The trailing visible expression's value is sent as display data via
  `print()`. Assignment-like heads (`<-`, `=`, `function`, `if`, `for`, etc.)
  are treated as silent so they produce no output, matching REPL behaviour.
- Plots drawn on the active graphics device are captured as PNG (or PDF
  fallback) images via `recordPlot()`/`dev.off()` sequences.
- `inspect_variables` and `diagnose` go through the driver's `inspect`/`check`
  requests; `check` only parses, it does not evaluate.

## Interpreter resolution

`MARS_RSCRIPT` env var wins; otherwise probes `Rscript --version`, then
`R --version` on PATH.

## Rules

- Never execute user code in the host process — always the subprocess.
- Timeout/cancellation kills the whole process tree and drops the driver;
  the next cell respawns a fresh R session (state is lost, same contract as
  the other interpreter backends).
- Requires the `jsonlite` R package for data interchange; the driver checks
  for `library(jsonlite)` and fails gracefully if not available.
- stderr is a non-fatal `Output::Text` stream (`[stderr]` prefix): R warnings
  must not mark a cell as failed. Genuine exceptions arrive via the driver's
  `error` result.
- `readline()`/`scan()` are unsupported and consume protocol lines; users
  should avoid them in notebook cells.
- `execution_count` increments only on successful evaluation, matching the
  Rust backend (Jupyter would count errored cells too — documented difference).

## Testing

```bash
cargo test -p mars-runtime-r   # requires Rscript + jsonlite on PATH; skips otherwise
cargo clippy -p mars-runtime-r --all-targets -- -D warnings
```
