---
title: Runtime
description: Mars runtime documentation
order: 5
author: Mars Team
---

# Execution runtime

`mars-runtime` provides a persistent Rust notebook session without running
arbitrary notebook code inside the desktop process. Successful cells can define
bindings used by later cells, and failed or interrupted cells are never
committed to the session.

## What the runtime guarantees

- Successful cells can define bindings used by later cells.
- Compilation and execution happen in a child process.
- Failed or interrupted cells are not committed to the session.
- The session can be reset without restarting the application.
- A timeout can cover both compilation and execution.
- Cancellation cleans up compiler and runtime descendants on supported
  platforms.

It does **not** provide instruction-level interruption inside arbitrary Rust
code. Once user code is running, cancellation can terminate the process, but
Rust code cannot observe a cooperative cancellation hook unless the notebook
author implements one.

## The replay model

Mars preserves bindings by replaying every previously successful cell into a
generated Rust program and compiling that program with `rustc`. The compiled
program runs in a temporary directory outside the desktop process.

Conceptually, a session progresses as follows:

1. Store the source of each successful cell.
2. Generate a Rust program containing the accumulated source.
3. Add a conservative wrapper for the current cell's final expression when
   automatic result display is possible.
4. Compile the generated program, directly or through a temporary Cargo
   project.
5. Start the compiled child process in an isolated process group or tree.
6. Capture stdout, stderr, diagnostics, and duration.
7. Commit the current cell only after successful completion.

Because previous cells are regenerated, side effects such as file writes,
logging, random-number generation, and external requests may happen again. Make
side-effecting cells idempotent, or isolate those effects behind explicit
functions.

Replay keeps the runtime portable, isolated, and straightforward to reset. The
tradeoff is repeated compilation and replayed side effects. The session reuses
its temporary workspace and Cargo cache, while each cell remains an isolated
child-process execution boundary.

## Automatic result display

The runtime automatically prints a final expression. It first tries `Debug`
and retries with `Display` when the value does not implement `Debug`.

Automatic output extraction accepts multiline final expressions and removes
trailing line comments before analysis. Declarations and statement-only cells
are left untouched, so they still need explicit output.

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

## Display output

The replay runtime translates Evcxr-style display markers in captured stdout.
Libraries such as `plotters` print:

```text
EVCXR_BEGIN_CONTENT text/html
<div><svg ...></svg></div>
EVCXR_END_CONTENT
```

Mars removes the markers and turns a standalone SVG payload into an
`image/svg+xml` MIME bundle with a `text/plain` fallback. HTML that merely
contains an SVG alongside visible content stays an HTML output.

## Diagnostics

Compiler diagnostics use rustc's JSON message format when compiling direct
Rust sessions. Mars retains diagnostic levels, error codes, primary and
secondary labeled spans, source ranges, and rendered notes. The desktop
diagnostic adapter converts Rust source columns into JavaScript editor offsets
without splitting Unicode characters incorrectly. Cargo-backed compilation
falls back to Cargo's emitted diagnostic text when Cargo wraps compiler
messages.

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
