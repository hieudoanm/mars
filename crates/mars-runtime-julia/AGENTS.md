# mars-runtime-julia — Julia Execution Backend

Executes Julia code interactively with shared state across cells. Depends on
`mars-core` and `mars-runtime`.

## Architecture

- `JuliaSession` (`session.rs`) implements `RuntimeBackend` by spawning a
  long-lived `julia --startup-file=no --color=no driver.jl` subprocess
  (driver source embedded via `include_str!`, written to a per-session temp
  dir removed on `Drop`).
- The driver speaks newline-delimited JSON — the same protocol as the Python
  and R drivers. A `Threads.@spawn` stdin reader dispatches work
  (`execute`/`inspect`/`check`) onto a requests channel while
  `input_response` messages go straight to a pending-input channel, so a
  cell blocked in `readline()` can be answered mid-execution.
- Requests: `execute`, `inspect`, `check`, `input_response`, `shutdown`.
  Responses: `stream`, `display`, `mime`, `input_request`, `result`,
  `inspection`, `check_result`.
- The driver contains a minimal JSON parser/writer (`jparse_*`, `json_write`)
  because no JSON stdlib is guaranteed; keep it in sync with the message
  shapes used by `session.rs`.
- Cell sources are parsed with `Meta.parseall` and each top-level expression
  is evaluated via `Core.eval` in a dedicated module `MarsKernel`, so
  bindings persist across cells in an isolated namespace (Main stays clean).
- `display` and `readline` are shadowed inside `MarsKernel` by
  `mars_display`/`mars_readline` (installed via `setglobal!` with a
  `Core.eval` fallback). `display()` publishes `mime` bundles built from
  `repr(MIME"...", x)`; `readline(prompt)` emits `input_request` and blocks
  on the pending-input channel.
- The trailing non-silent expression's value is sent as `display` parts
  (MIME reprs plus `text/plain`); `ans`/`ans_prev` are maintained.
  Assignment-like heads (`:=`, `function`, `using`, `struct`, ...) are
  treated as silent so they produce no output, matching REPL behaviour.
- stdout/stderr are captured through `ProtocolIO` (an `IO` subtype that
  emits `stream` messages on newline boundaries and on flush) wrapped in
  `redirect_stdout`/`redirect_stderr` for the duration of each cell.
  `stderr` is a non-fatal `Output::Text` stream (`[stderr]` prefix).
- `inspect_variables`/`diagnose` go through the driver's `inspect`/`check`
  requests; `check` only parses (`Meta.parseall`), it does not evaluate.

## Interpreter resolution

`MARS_JULIA` env var wins; otherwise probes `julia --version` on PATH.

## Rules

- Never execute user code in the host process — always the subprocess.
- Timeout/cancellation kills the whole process tree and drops the driver;
  the next cell respawns a fresh Julia session (state is lost, same contract
  as the other interpreter backends).
- Protocol writes must go to `REAL_STDOUT` (the pipe captured at startup),
  never the redirected `stdout`, or stream capture would recurse.
- `repr(MIME"...", x)` falls back to the `text/plain` repr for unimplemented
  MIME types — `mime_payload` drops payloads identical to the plain repr so
  plain values are not mislabeled as html/images.
- Requires a reasonably recent Julia (`Meta.parseall`, `findlast`,
  `redirect_stderr`); 1.9+ is the safe baseline.
- `execution_count` increments only on successful evaluation, matching the
  other backends.

## Testing

```bash
cargo test -p mars-runtime-julia   # requires a julia on PATH; skips otherwise
cargo clippy -p mars-runtime-julia --all-targets -- -D warnings
```
