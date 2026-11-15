# Rust Runtime

## Purpose and guarantees

`mars-runtime` provides a persistent Rust notebook session without running arbitrary notebook code inside the desktop process. It guarantees:

- Successful cells can define bindings used by later cells.
- Compilation and execution happen in a child process.
- Failed or interrupted cells are not committed to the session.
- The session can be reset without restarting the desktop application.
- A timeout can cover both compilation and execution.
- Cancellation cleans up compiler/runtime descendants on supported platforms.

It does **not** provide instruction-level interruption inside arbitrary Rust code. Once user code is running, cancellation can terminate the process, but Rust code cannot observe a cooperative cancellation hook unless the notebook author implements one.

## Execution model

`mars-runtime` preserves bindings by replaying all previously successful cells into a generated Rust program and compiling that program with `rustc`. The compiled program runs in a temporary directory outside the desktop process.

Compilation and execution are covered by an optional total timeout. A shared cancellation flag can terminate either child process. Failed or interrupted cells are not appended to the session, so later cells cannot observe partial state.

The runtime automatically prints a final expression. It first tries `Debug` and retries with `Display` when the value does not implement `Debug`. This transformation remains intentionally conservative: item declarations and statement-only cells still need explicit output, while multiline expressions and trailing line comments are supported.

Replay means earlier side effects may occur again. Dependency-backed notebooks use a temporary Cargo project and validate dependency names before generation. They may provide inline Cargo table specifications, an embedded `Cargo.lock`, and `--locked`/`--offline` execution controls; the CLI persists a generated lockfile into notebook metadata after a successful dependency run.

The current replay strategy was chosen over embedding `rustc` or maintaining a long-lived compiler process because it keeps the runtime portable, isolated, and straightforward to reset. The tradeoff is repeated compilation and replayed side effects. The session reuses its temporary workspace and Cargo cache, while each cell remains an isolated child-process execution boundary. Tests explicitly cover the replay behavior so it is not mistaken for independent cell execution.

Conceptually, a session progresses as follows:

1. Store the source of each successful cell.
2. Generate a Rust program containing the accumulated source.
3. Add a conservative wrapper for the current cell's final expression when automatic result display is possible.
4. Compile the generated program directly or through a temporary Cargo project.
5. Start the compiled child process in an isolated process group/tree.
6. Capture stdout, stderr, diagnostics, and duration.
7. Commit the current cell only after successful completion.

Because previous cells are regenerated, side effects such as file writes, logging, random-number generation, and external requests may happen again. Users should make side-effecting cells idempotent or isolate those effects behind explicit functions.

## Performance

The repository includes a release benchmark:

```bash
make benchmark CELLS=8
make benchmark CELLS=16
```

On the development machine used for this implementation, the replay benchmark took approximately 3.28 seconds for 8 cells and 6.56 seconds for 16 cells (about 410 ms per cell). This is a small workload benchmark, not a universal performance claim. A 32-cell run measured approximately 13.36 seconds (about 418 ms per cell), further confirming the expected linear cost of rebuilding the accumulated program.

A long-lived compiler or incremental execution service could reduce that cost, but it would add compiler-process lifecycle, state isolation, portability, and reset-complexity risks. Project Mars therefore retains replay as the validated default until an alternative can demonstrate equivalent isolation and semantics with representative benchmarks.

## Automatic output extraction

Automatic output extraction accepts multiline final expressions and removes trailing line comments before analysis. Values are first formatted with `Debug`; when that compilation fails, Project Mars retries the generated wrapper with `Display`, allowing display-only values to render without changing explicit `println!` output. Declarations and statement-only cells are still left untouched.

## Cancellation

Cancellation has two layers: a shareable `CancellationToken` lets callers cancel before compilation or between runtime phases, and the child-process watcher enforces cancellation while rustc/Cargo or the compiled program is running. Each child is started in its own process group on Unix and is terminated as a group; Windows uses `taskkill /T /F` before falling back to direct termination. This prevents compiler/runtime descendants from outliving a cancelled cell. Fine-grained interruption inside arbitrary user Rust remains impossible without changing the execution model.

## Compiler diagnostics

Compiler diagnostics use rustc's JSON message format when compiling direct Rust sessions. Project Mars retains diagnostic levels, error codes, primary and secondary labeled spans, source ranges, and rendered notes. The desktop diagnostic adapter converts Rust source columns into JavaScript editor offsets without splitting Unicode characters incorrectly. Cargo-backed compilation falls back to Cargo's emitted diagnostic text when Cargo wraps compiler messages.

## Dependency management

Rust notebooks may declare registry dependencies in `metadata.dependencies`:

```json
{
  "metadata": {
    "dependencies": {
      "serde": "1",
      "polars": "0.36"
    }
  }
}
```

Project Mars creates a temporary Cargo project for those notebooks. Dependency resolution uses Cargo's configured cache and registry and may require network access. For reproducible builds, use `--locked` and `--offline`:

```bash
mars run analysis.mars --locked --offline
```

This requires `metadata.cargo_lock` to be present (set after first successful run).

## Evcxr backend (experimental)

Project Mars includes an experimental Evcxr backend for faster re-execution without recompilation. Enable via:

```bash
mars run analysis.mars --runtime evcxr
# or
export MARS_EXPERIMENTAL_RUNTIME=evcxr
mars run analysis.mars
```

**Limitations:**
- Does not support `--locked` or embedded Cargo.lock
- Experimental and not recommended for production use
- May have state persistence issues in certain scenarios

See [Evcxr experiment documentation](./15-evcxr-experiment.md) for details.

## Runtime helpers

Rust runtime provides helper functions for display and input:

- `mars_table()` — Display tabular data
- `mars_csv()` — Parse CSV files
- `mars_display_html()` — Render HTML output
- `mars_display_latex()` — Render LaTeX equations
- `mars_display_image()` — Display images
- `mars_display_markdown()` — Render Markdown
- `mars_display_mime()` — Display MIME bundles
- `mars_chart()` — Create chart specifications
- `mars_prompt()` — Interactive input prompts

See [Runtime helpers documentation](./16-runtime-helpers.md) for details.

## Architecture rules

See `crates/mars-runtime/AGENTS.md` for architecture rules and implementation details.
