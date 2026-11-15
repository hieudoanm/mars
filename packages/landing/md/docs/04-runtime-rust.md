---
title: Rust Runtime
description: Project Mars Rust runtime documentation
order: 4
author: Project Mars Team
tags: [runtime, rust, execution, replay, evcxr]
---

# Rust Runtime

Project Mars provides two Rust execution backends: the replay runtime (default) and the
experimental Evcxr runtime. Both maintain persistent sessions without running
arbitrary notebook code inside the desktop process.

## What the runtime guarantees

- Successful cells can define bindings used by later cells.
- Compilation and execution happen in a child process.
- Failed or interrupted cells are not committed to the session.
- The session can be reset without restarting the application.
- A timeout can cover both compilation and execution.
- Cancellation cleans up compiler and runtime descendants on supported platforms.

It does **not** provide instruction-level interruption inside arbitrary code.
Once user code is running, cancellation can terminate the process, but code
cannot observe a cooperative cancellation hook unless the notebook author
implements one.

## Replay runtime (default)

The replay runtime re-executes all previous cells for each new cell by:

1. Storing the source of each successful cell.
2. Generating a Rust program containing the accumulated source.
3. Adding a conservative wrapper for the current cell's final expression.
4. Compiling the generated program with `rustc` or Cargo.
5. Starting the compiled child process in an isolated process group.
6. Capturing stdout, stderr, diagnostics, and duration.
7. Committing the current cell only after successful completion.

This provides isolation and reproducibility at the cost of recompilation and
replayed side effects.

### Side effects

Because previous cells are regenerated, side effects such as file writes,
logging, random-number generation, and external requests may happen again. Make
side-effecting cells idempotent, or isolate those effects behind explicit
functions.

## Evcxr runtime (experimental)

The Evcxr backend uses the Evcxr Jupyter kernel for faster re-execution:

- Maintains a long-lived Evcxr session.
- Faster cell execution without recompilation.
- Limited dependency support compared to replay.
- Opt-in via `--runtime evcxr` or configuration.

See `docs/10-evcxr-experiment.md` for detailed analysis and current limitations.

## Automatic result display

The runtime automatically prints the current cell's final expression. Replayed
cells keep their stored outputs and are not printed again. The runtime first
tries `Debug` and retries with `Display` when the value does not implement
`Debug`.

Automatic output extraction accepts multiline final expressions and removes
trailing line comments before analysis. Declarations and statement-only cells
are left untouched, so they still need explicit output.

## Display helpers

Project Mars injects helper functions into Rust cells for rich output:

- `mars_table(&[columns], &[vec![rows]])` — Render a table from column names and row data
- `mars_csv(r#"csv"#)` — Parse RFC 4180 CSV and render as a table
- `mars_display_html(&str)` — Render HTML (sanitized at render time)
- `mars_display_latex(&str)` — Render LaTeX equations via KaTeX
- `mars_display_image(mime, &[u8])` — Render an image from bytes
- `mars_chart(spec_json)` — Render a Vega-Lite chart specification
- `mars_display_markdown(&str)` — Render Markdown as a MIME bundle
- `mars_display_mime(mime, data)` — Render arbitrary MIME bundles
- `mars_prompt(&str)` — Interactive stdin prompt

These helpers emit special stdout markers that the runtime parses into typed
outputs. They appear in code-cell autocomplete with snippet placeholders.

### Cell directives

Cells can include metadata directives in comments:

```rust
// mars: dep serde = "1.0"
// mars: timeout 10000
```

These directives are parsed before execution and override notebook-level settings.

## Timeouts and cancellation

Compilation and execution are covered by an optional total timeout. A shared
cancellation flag can terminate either child process.

Cancellation has two layers:

- A shareable cancellation token lets callers cancel before compilation or
  between runtime phases.
- A child-process watcher enforces cancellation while `rustc`, Cargo, or the
  compiled program is running.

Each child starts in its own process group on Unix and is terminated as a
group. Windows uses `taskkill /T /F` before falling back to direct termination.
This prevents compiler and runtime descendants from outliving a cancelled cell.

## Dependencies

Dependency-backed notebooks use a temporary Cargo project and validate
dependency names before generation. They may include inline Cargo table
specifications, an embedded `Cargo.lock`, and `--locked` or `--offline`
execution controls. After a successful dependency run, the CLI persists a
generated lockfile into notebook metadata.

### Specifying dependencies

Add a Cargo dependencies table to notebook metadata:

```json
{
  "metadata": {
    "cargo": {
      "dependencies": {
        "polars": { "version": "0.36" },
        "ndarray": { "version": "0.15" },
        "plotters": { "version": "0.17" }
      }
    }
  }
}
```

Or use version requirements in a single line:

```json
{
  "metadata": {
    "cargo": {
      "dependencies": {
        "polars": "0.36",
        "ndarray": "0.15",
        "plotters": "0.17"
      }
    }
  }
}
```

### Lockfile persistence

After a successful run with dependencies, Project Mars generates a `Cargo.lock` and stores it in the notebook's `metadata.cargo_lock` field. This enables deterministic, offline, and locked execution modes:

```bash
# Run using only the stored lockfile
mars notebook run analysis.mars --locked --offline --deterministic --report
```

## Diagnostics

Compiler diagnostics use rustc's JSON message format when compiling direct
Rust sessions. Project Mars retains diagnostic levels, error codes, primary and
secondary labeled spans, source ranges, and rendered notes. The desktop
diagnostic adapter converts Rust source columns into JavaScript editor offsets
without splitting Unicode characters incorrectly. Cargo-backed compilation
falls back to Cargo's emitted diagnostic text when Cargo wraps compiler
messages.

The runtime also provides variable inspection via `inspect_variables` and
cell diagnostics via `diagnose_cell`, which map compiler errors back to
specific notebook cells and positions.

## Performance

Replay compilation is the validated default, not an assumption that it is
asymptotically optimal. The runtime benchmark is available from the repository
root:

```bash
make benchmark CELLS=8
make benchmark CELLS=16
make benchmark CELLS=32
```

Local measurements were approximately 3.28 s for 8 cells, 6.56 s for 16 cells,
and 13.36 s for 32 cells. This is roughly linear for the tested range because
each execution rebuilds the accumulated source.

A longer-lived compiler or runtime could reduce latency by retaining compiler
state, but it would need to solve process ownership, reset semantics,
cross-platform compiler availability, memory growth, isolation, dependency
changes, and recovery after a crashed session. Any replacement must preserve
the observable session semantics and include equivalent cancellation, reset,
isolation, and benchmark coverage.
