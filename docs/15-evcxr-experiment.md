# Evcxr persistent runtime experiment

**Status:** isolated experimental backend; replay remains the default.

## Architecture

`mars-runtime-evcxr` implements `mars-runtime::RuntimeBackend` and owns one
`evcxr::CommandContext` per notebook session. The context preserves values,
functions, types, and Cargo dependencies between cell requests. It runs through
Evcxr's child-process protocol: the Project Mars CLI or Tauri process hosts the Evcxr
controller, while Evcxr compiles and loads user code in a separate evaluator
process. Both application entry points call `mars_kernel::runtime_hook()` so
the same executable can service Evcxr runtime and rustc-wrapper subprocesses.
No Evcxr types enter `mars-core`, and `.mars` serialization is unchanged.

The kernel selects the existing replay runtime unless explicitly opted in.
Requests and results continue through the existing `RuntimeBackend`, kernel,
and `mars-protocol` output types. CLI `mars run` also uses the backend factory.
The experimental runtime does not implement Project Mars's embedded Cargo.lock
semantics; it rejects `--locked` and embedded locks rather than claiming
reproducibility it cannot provide. Dependency names and values are validated
by the existing runtime helper and passed to Evcxr as `:dep` directives.

## Build and setup

Requirements are the workspace Rust toolchain, Cargo, and the `rust-src`
component. Evcxr 0.22.0 and the compatible Salsa macro patch versions are
locked in `Cargo.lock`. The first use builds Evcxr and its Rust Analyzer
dependencies and can take substantially longer than subsequent builds.

```sh
rustup component add rust-src
cargo build -p mars --features experimental-evcxr
cargo run -p mars --features experimental-evcxr -- run examples/valid-notebook.mars --runtime evcxr
```

For the persistent JSONL kernel endpoint:

```sh
cargo run -p mars --features experimental-evcxr -- kernel serve --runtime evcxr
```

For the desktop, build Tauri with `--features experimental-evcxr` and launch
with `MARS_EXPERIMENTAL_RUNTIME=evcxr`. Without that feature the desktop and
CLI retain replay and report that the requested experimental backend is not
compiled in. The backend selector is opt-in; existing CLI and VS Code defaults
remain replay.

The benchmark runner emits CSV with each individual sample:

```sh
cargo run -p mars-runtime-evcxr --release --bin benchmark -- 20 replay > replay-benchmark.csv
cargo run -p mars-runtime-evcxr --release --bin benchmark -- 20 evcxr > evcxr-benchmark.csv
```

The runner samples session construction and first-cell, repeated-cell,
variable-reuse, function, struct, dependency, and 8/16/32-cell workloads for
both backends. Session-start and first-cell samples are cold per newly created
session; later cells within a session are warm. Multi-cell and dependency
workloads use five runs to keep the long replay baseline practical; the
single-cell and startup workloads use the requested 20 independent sessions.
The runner reports median and nearest-rank p95 from the raw samples. It does
not clear the OS page cache, Rust compiler cache, or Cargo registry, so “cold”
here means a new runtime session, not a cold machine or empty build cache.

## Benchmark results

The baseline investigation measured replay at 3.48 s median for 8 small cells
and 6.86 s for 16 cells, on the host and with the limitations recorded in
[`PERFORMANCE_INVESTIGATION.md`](../PERFORMANCE_INVESTIGATION.md). It did not
include Evcxr, so those numbers are context rather than a paired result.

The paired runner is included, but **no Evcxr latency medians or p95 values
are reported**. On the current macOS ARM64 host, the integration run reached a
second cell that reads a value declared in the previous cell and did not
complete within two minutes. A process sample showed the Project Mars controller
waiting for Evcxr evaluation while the evaluator child was idle. The runner
was stopped rather than publishing a timing for a stuck execution. This is a
compatibility failure for the tested environment, not evidence of a measured
speedup. The raw benchmark harness is ready to rerun after that failure is
resolved; no output CSV is checked in.

The harness uses 20 independent sessions for session-start and single-cell
workloads. Five independent samples are used for multi-cell and dependency
workloads to keep the repeated-compilation replay baseline practical. Its
nearest-rank p95 values are coarse. The benchmark does not clear the OS page
cache, Rust compiler cache, or Cargo registry.

## Compatibility findings

The opt-in integration executable encodes scenarios for sequential values,
functions, structs, borrowing and moves, useful compilation errors,
failed-cell state retention, reset, a `regex` dependency, timeout, panic, and
evaluator termination/recovery. The suite did not complete because a later
cell reading a previous value stalled, so these scenarios are not represented
as passing compatibility results. Stdout and stderr are wired through
Evcxr's output channels and mapped to the existing Project Mars `Text` and `Error`
variants, but output capture also lacks a completed runtime test. The adapter
preserves the existing kernel response shape at the type boundary.
Run the scenario executable explicitly with
`MARS_EVCXR_RUN_INTEGRATION=1 cargo test -p mars-runtime-evcxr --test evcxr_runtime`.

Evcxr implements REPL semantics rather than replay semantics. Successful
side-effects are not repeated on later cells. A compilation error is handled
inside Evcxr and should leave the preceding state usable. A panic or abrupt
process exit may discard the process state; the adapter reports the failure
and recreates a clean session. Timeout and cancellation kill the evaluator
child because arbitrary Rust code has no cooperative cancellation point. The
observed controller stall did not return promptly even after the child was
terminated, so timeout/cancellation are not yet reliable end-to-end guarantees
for this Evcxr version and platform.

Known compatibility limits:

- Evcxr documents that macros defined in one cell are not generally available
  to later cells. Procedural macro and compiler-plugin workflows are not
  equivalent to Cargo projects.
- Evcxr manages dependencies through its own generated Cargo project. Project Mars
  notebook lockfile pinning, `--locked`, and full offline/reproducible
  dependency guarantees are not implemented by this adapter.
- Evcxr's evaluator is a child process, but it is not a hardened sandbox.
  User code can access the host user account, filesystem, and network, and
  may start descendant processes. Direct evaluator termination does not
  guarantee cleanup of every user-created descendant on every platform.
- Restart after timeout, crash, or evaluator exit loses all live session
  values. Project Mars does not replay prior cells into the replacement Evcxr context.
- Evcxr's Rust Analyzer based completion and type handling add startup and
  compatibility surface. Rust toolchain and Evcxr upgrades require a fresh
  behavior and performance run.
- Rich display output is limited to MIME values Evcxr itself produces; this
  experiment does not add Project Mars-specific plotting or table APIs.

## Limitations and recommendation

Keep this backend as an opt-in experiment and do not adopt it. A basic
cross-cell value read stalled in the current environment, so the core
persistence claim has not been demonstrated even though Evcxr is designed for
that use. There are no paired Evcxr latency measurements to support adoption.
The dependency lockfile gap and incomplete process-tree isolation add further
production risks. Revisit only after the cross-cell stall is understood and
the full parity suite and benchmark complete on supported platforms.

An alternative next step is to retain replay while reducing redundant work
with stable incremental compilation and compiled-program caching. For a
persistent backend, evaluate Evcxr against a separate supervised worker with
explicit protocol, resource limits, process-group/job cleanup, and a
Project Mars-managed dependency lock contract.
