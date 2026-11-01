# mars-runtime-python — Python Execution Backend

Executes Python code interactively with shared state across cells. Depends on
`mars-core` and `mars-runtime`.

## Architecture

- `PythonSession` (`session.rs`) implements `RuntimeBackend` by spawning a
  long-lived `python3 -u driver.py` subprocess (driver source embedded via
  `include_str!`, written to a per-session temp dir removed on `Drop`).
- The driver speaks newline-delimited JSON. A dedicated `stdin_reader` thread
  dispatches work (`execute`/`inspect`/`check`) onto the exec loop while
  `input_response` messages go straight to a pending-input queue, so a cell
  blocked in `input()` can be answered mid-execution.
- Requests: `execute`, `inspect`, `check`, `input_response`, `shutdown`.
  Responses: `stream`, `display`, `mime`, `input_request`, `result`,
  `inspection`, `check_result`.
- User stdout/stderr are captured through file-like `Channel` objects and
  forwarded as stream events; `sys.stdin` is replaced with an empty
  `StringIO` during execution so user code cannot consume protocol lines.
- `input()` is implemented by a driver builtin that emits `input_request`
  (surfaced as `StreamEvent::Prompt`) and blocks on the pending-input queue;
  `write_stdin`/`stdin_writer` send `input_response` lines. A `null` text
  value raises `EOFError` (headless runs).
- `display()` is an IPython-style builtin publishing `mime` messages
  (`Output::MimeBundle`), honouring `raw=True` dicts, `include`/`exclude`,
  `metadata`, and `_repr_mimebundle_`.
- Cell sources starting with `!` run as shell commands; `%pip`/`%pip3` run
  `sys.executable -m pip`. Other `%` magics are rejected with a clear error.
- The last expression of a cell is `eval`'d and its value sent as display
  data; `_repr_html_`/`_repr_png_`/`_repr_jpeg_`/`_repr_svg_` are honoured,
  with `text/plain` `repr` as fallback. Matplotlib figures are flushed as PNG
  mime outputs (the driver is spawned with `MPLBACKEND=Agg` unless the user
  set `MPLBACKEND` already); each figure is shown once per session.
- Top-level `await`/`async for`/`async with` are supported via
  `ast.PyCF_ALLOW_TOP_LEVEL_AWAIT` and a persistent event loop.

## Interpreter resolution

`MARS_PYTHON` env var wins; otherwise probes `python3`, `python`, `py` with
`--version`.

## Rules

- Never execute user code in the host process — always the subprocess.
- Timeout/cancellation kills the whole process tree and drops the driver;
  the next cell respawns a fresh interpreter (state is lost, same contract
  as the Evcxr backend).
- `inspect_variables` and `diagnose` go through the driver's `inspect`/`check`
  requests; interactive input goes through `input_response`.
- stderr is a non-fatal `Output::Text` stream (`[stderr]` prefix): Python
  warnings must not mark a cell as failed. Genuine exceptions arrive via the
  driver's `error` result.
- `execution_count` increments only on successful evaluation, matching the
  Rust backend (Jupyter would count errored cells too — documented difference).

## Testing

```bash
cargo test -p mars-runtime-python   # requires a python3 on PATH
cargo clippy -p mars-runtime-python --all-targets -- -D warnings
```
