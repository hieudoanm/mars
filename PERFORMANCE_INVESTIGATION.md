# Mars Performance Investigation

**Date:** 2026-11-01  
**Scope:** Read-only investigation of the Rust workspace and performance of the existing runtime. The only repository file created for this investigation is this report; the pre-existing working-tree edit to `examples/getting-started.mars` was preserved.

## 1. Executive summary

Mars is slow primarily because a successful code-cell run is implemented as a fresh whole-program compile and execution. `RustSession` appends the new source to all earlier successful cells, generates one `main`, starts `rustc` (or Cargo when dependencies are declared), then starts the compiled program. The runtime process itself persists in the desktop/kernel, but the compiler and user program do not.

The local release benchmark measured 8 small stateful cells at a **3.48 s median total** and 16 cells at **6.86 s**. The benchmark's inputs grow with each cell; the total is nearly proportional to cell count, consistent with repeatedly rebuilding a growing source file. The getting-started declaration cell took **414 ms** in a temporary valid copy; a separate `1 + 1` expression measured **422 ms median**. These measurements include compilation, process startup, execution, and the runtime's polling; they do not isolate those phases.

The first priority is to reduce repeat compilation while preserving the process boundary. A low-risk experiment is a stable per-session incremental directory for the existing `rustc` path, plus caching the compiled artifact for an identical generated program. These can reduce repeat work but cannot make each new cell independent of prior source. In parallel, run a time-boxed Evcxr integration spike: Evcxr already implements persistent Rust evaluation and compilation caching, but its state-preservation technique, dependency/toolchain requirements, and operational behavior must be tested against Mars's semantics before selection.

The focused example has an immediate data problem: the current user-edited output object is not valid Mars output JSON. `mars run examples/getting-started.mars` fails during deserialization before compilation starts. Clearing outputs in a temporary copy allowed its code cell to run in **414 ms**. The stored timeout text refers to the CLI kernel server and is stale for the current desktop path, which constructs the kernel inside the Tauri backend.

## 2. Existing execution architecture

The Cargo workspace has nine members: `mars-core`, `mars-document`, `mars-runtime`, `mars-protocol`, `mars-kernel`, `mars-jupyter`, `mars-renderer`, `mars-cli`, and the Tauri backend at `apps/desktop/src-tauri`. Responsibilities are separated by crate: shared types; `.mars` persistence; compilation/execution; typed request/response models; kernel lifecycle; Jupyter conversion; HTML rendering; CLI; and desktop adapters.

Notebook cells are represented in `mars-document` as `Cell` values with stable ID, type, source, metadata, optional execution count, and typed outputs. The format deliberately does not prescribe a runtime. `examples/getting-started.mars` currently has one Markdown cell and one code cell:

```rust
let numbers = vec![1, 2, 3, 4, 5];
```

With no external dependencies, Mars generates a Rust source file whose `main` contains the successful prior cells and the current cell, then invokes `rustc --edition=2024 --error-format=json ... -o session-bin`. If the notebook declares dependencies, it writes a temporary Cargo manifest and source file and invokes `cargo build`; the per-session temporary directory also contains Cargo's target directory. After successful compilation, Mars starts the binary, captures stdout/stderr, and commits the source to session history only after a successful process exit. A timeout/cancellation kills the compiler or execution process tree.

Variables persist because all prior cell source is replayed into the same generated `main` scope. This preserves ordinary Rust lexical typing and ownership within the reconstructed program. It does **not** preserve a live Rust stack, heap, thread, or process between cells. Side effects from earlier cells run again. Each result is therefore the output of a whole replay, not an execution of only the latest cell.

### `getting-started.mars` inspection

The example's current code source is minimal and does not itself imply a long compile. The saved output at lines 20–25 is:

```json
{"type":"error","text":"Error: Execution timeout after 30000ms..."}
```

Mars's `Output` enum uses internally tagged JSON. Its error variant requires `name`, `message`, and `traceback`; it has no `text` field. Consequently, `Notebook::load` rejects the current example before the runtime sees it. The saved object matches the VS Code extension's error-output conversion: `extensions/vscode/src/extension.ts` converts `application/vnd.code.notebook.error` items to `{ type: "error", text }`, then serializes that shape directly. This identifies a concrete serializer/schema mismatch. The message itself is generated by the VS Code execution timeout catch path and mentions `mars kernel serve`; desktop Tauri commands call the in-process Rust kernel directly. The CLI server is a separate JSONL adapter used by the VS Code integration.

To verify the cell without changing the working-tree file, I made a temporary copy with code-cell outputs cleared and ran `target/debug/mars run /tmp/mars-getting-started-investigation.mars --no-save --report`. The cell succeeded in **414 ms**, with no output because it is a declaration-only cell. The original file remains untouched.

## 3. Execution lifecycle

```text
React cell action / CLI run
  -> desktop: Tauri execute_cell command (or CLI directly)
  -> mars-kernel: serializes access to a Kernel and tracks status
  -> mars-runtime: append prior successful sources + current source
  -> generate session.rs and compile with rustc (or write Cargo project + cargo build)
  -> launch compiled program in a child process group
  -> capture stdout/stderr and compiler diagnostics
  -> return typed response; desktop stores output and renders it
```

In the desktop path, the React hook invokes a synchronous Tauri command. A mutex protects the `Kernel`; a shared cancellation flag can be set while execution waits. Tauri constructs one kernel at app startup, and `configure_kernel` replaces it when notebook dependencies are configured. The kernel and runtime remain alive across cells, but a compiler subprocess and program subprocess are spawned for each cell. The CLI `mars run` creates a fresh `RustSession` for the notebook run. `mars kernel serve` holds one kernel for its JSONL process lifetime.

Notebook serialization is not in the hot per-cell execution path. The desktop sends source strings to Tauri and receives typed outputs; notebook persistence is a separate command. JSON/protocol and frontend rendering are unlikely to explain the measured hundreds of milliseconds per tiny cell, though no UI profiling was performed.

## 4. Performance measurements

### Environment and methodology

- macOS 15.6, Apple Silicon (`aarch64-apple-darwin`); exact hardware SKU and memory were unavailable from the restricted shell.
- `rustc 1.99.1 (58cf8f9c3 2026-10-13)`, LLVM 23.1.1; `cargo 1.99.1`.
- Workspace toolchain is pinned by `rust-toolchain.toml` to 1.99.1.
- Runtime benchmark was built in release mode using `cargo run -p mars-runtime --release --bin runtime_benchmark -- 8`; the measured benchmark loops ran the resulting binary directly, five times at 8 cells and three times at 16 cells.
- The 8/16-cell benchmark is a small arithmetic/state workload. Each benchmark cell adds a tiny statement/expression and the runtime replays all earlier cells. The harness measures session creation plus each compile and execution; it excludes building the benchmark binary itself.
- Reported p95 is nearest-rank p95 over only 3 or 5 whole-run samples and therefore equals the sample maximum. Treat it as exploratory, not a service-level estimate. The harness does not report individual cell samples or compiler-versus-execution phase durations.

| Operation                                                |                                                                                                         Measurement | Interpretation                                                                                                                                                                                                                                                |
| -------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------: | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Runtime benchmark, 8 cells, release                      |                                                Median **3,479 ms**; sample range 3,391–3,668 ms; p95 proxy 3,668 ms | **435 ms/cell** average across the run.                                                                                                                                                                                                                       |
| Runtime benchmark, 16 cells, release                     |                                                Median **6,861 ms**; sample range 6,858–6,949 ms; p95 proxy 6,949 ms | **429 ms/cell** average. Doubling cells nearly doubled total time.                                                                                                                                                                                            |
| Simple expression `1 + 1`, first cell                    | Seven fresh JSONL kernel sessions; runtime-reported samples 533, 422, 433, 422, 413, 430, 401 ms; median **422 ms** | Includes compile and program run, not server launch.                                                                                                                                                                                                          |
| Same simple expression, second execution in same session |                              Seven paired sessions; samples 425, 424, 427, 420, 418, 422, 413 ms; median **422 ms** | No measurable warm benefit; exact unchanged source still takes the `compile_program` path every request, with no artifact/result cache.                                                                                                                       |
| Getting-started declaration cell                         |                                                                 **414 ms** in a temporary copy with outputs cleared | Confirms the tiny code compiles and runs; saved example itself currently fails document loading.                                                                                                                                                              |
| Kernel/server process startup                            |                `mars kernel serve` with immediate EOF: first sample **41.7 ms**; warm median **4.17 ms** (six runs) | CLI process plus `Kernel`/temporary session initialization, excluding execution. Not equivalent to native app startup.                                                                                                                                        |
| Desktop application cold/warm startup                    |                                                                                                        Not measured | Requires a native launch/readiness harness and controllable cold-start conditions; no stable app launch measurement was available in this shell.                                                                                                              |
| External dependency addition / dependency-backed compile |                                                                                                        Not measured | No controlled representative dependency workload was defined; network/cache state would confound a single sample. Dependency path currently invokes Cargo.                                                                                                    |
| Clean full-workspace build                               |                                                                                                        Not measured | Avoided `cargo clean` because it would delete existing build artifacts. Building the release runtime benchmark compiled its package graph in 6.44 s; the CLI's initial debug build in this session took 12.50 s. Neither is a clean whole-workspace baseline. |

The current benchmark tool can be rerun with:

```bash
make benchmark CELLS=8
make benchmark CELLS=16
make benchmark CELLS=32
```

Existing `docs/06-runtime.md` records prior approximate totals of 3.28 s, 6.56 s, and 13.36 s for 8, 16, and 32 cells. The new 8/16-cell results are directionally consistent. No performance changes were made, so these are baseline observations, not claims of improvement.

## 5. Identified bottlenecks

| Severity                            | Finding and evidence                                                                                                                                                                                          | Root cause                                                                                                                                                         | Recommended solution                                                                                                                                                                                                                 | Expected benefit / complexity / regression risk                                                                                                                                                                     |
| ----------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Critical**                        | Every cell performs a compiler invocation and a compiled-program invocation (`mars-runtime/src/lib.rs`, `execute_with_control`). Tiny cells take about 0.4 s; 8→16 cells nearly doubles total benchmark time. | Reconstruct full program to emulate persistent variables, then compile and link it from scratch.                                                                   | Preserve the isolation boundary while testing incremental compilation and a per-session identical-artifact cache. In parallel, benchmark Evcxr behind a runtime adapter.                                                             | Largest likely latency reduction. Incremental/cache experiment: low-to-medium complexity. Evcxr adapter: medium/high. Risk includes stale artifacts, compiler-cache invalidation, and runtime semantic differences. |
| **High**                            | Prior cells are recompiled and rerun for every new cell; benchmark's source grows with history.                                                                                                               | Current runtime uses replay as its state model.                                                                                                                    | Replace replay only after deciding explicit semantics for side effects, failed cells, and state rollback. A persistent evaluation context is the long-term user experience.                                                          | Avoids history-length growth. High complexity; changing semantics may expose state that replay previously reconstructed and needs careful recovery after panic/cancel.                                              |
| **High**                            | Replayed code repeats prior filesystem, network, random, logging, and other side effects; current runtime docs acknowledge this.                                                                              | Source replay substitutes for live state.                                                                                                                          | Use persistent isolated worker with restart/recovery and explicit reset; do not cache execution results by default.                                                                                                                  | Better correctness and latency; more complex interruption, crash, memory, and cleanup lifecycle.                                                                                                                    |
| **High**                            | A final expression first compiles with `Debug`; if that fails, Mars may compile a second time with `Display` (lines 142–160).                                                                                 | Output protocol uses a compile failure as type-detection fallback.                                                                                                 | Prefer compiler-assisted display selection or an interpreter/evaluation engine that already handles expression display. At minimum instrument this path and avoid a second full compile where its applicability can be known safely. | Some expressions avoid duplicate compiler startup. Medium complexity; incorrect heuristics could hide real diagnostics.                                                                                             |
| **High for dependency notebooks**   | Each dependency session uses a UUID temporary Cargo project and a `target` directory beneath that project (lines 73–76, 414–446).                                                                             | Session isolation prevents artifact reuse across sessions; Cargo resolves/builds in each unique project.                                                           | Use a versioned shared Cargo target cache keyed by toolchain, target, profile, lockfile, features, and relevant flags while keeping per-session source/workspace isolation.                                                          | Avoid recompiling unchanged third-party dependencies across sessions. Medium complexity; requires robust cache keying, locking, and disk management.                                                                |
| **Medium**                          | Runtime polls `try_wait` every 10 ms (line 255 onward); every compile and execution incurs child process management.                                                                                          | Portable cancellation/timeout implementation uses polling.                                                                                                         | Profile before changing; use event-driven waiting where available, retaining cross-platform fallback and process-tree termination.                                                                                                   | Small latency reduction relative to ~400 ms compile. Medium complexity; risk in cancellation races/platform behavior.                                                                                               |
| **Medium (example/data integrity)** | Current `getting-started.mars` output fails `Notebook::load` because its error object has `text` instead of required error fields; the VS Code serializer writes this shape directly.                         | VS Code `convertNotebookOutput` models a Mars error as plain text instead of preserving `name`, `message`, and `traceback` required by `mars-core::Output::Error`. | Map VS Code error MIME items to the Mars error schema (or a valid text output where structured errors are unavailable); validate the serialized notebook with `mars inspect`/`mars run --no-save`.                                   | Restores loadability and prevents future malformed error outputs. Low/medium complexity; no runtime speed change.                                                                                                   |
| **Low until profiled**              | Desktop call serialization and UI rendering add overhead but no evidence suggests they dominate.                                                                                                              | Tauri command and frontend response/output rendering.                                                                                                              | Add stage timestamps from click to Tauri entry, runtime phases, response, and render.                                                                                                                                                | Prevents optimizing the wrong layer; low complexity.                                                                                                                                                                |

## 6. Root causes and cache analysis

### What is and is not reused

- The runtime session reuses its temporary directory and source/binary paths. This is file-location reuse only: each direct `rustc` command has no `-C incremental=...` option, and each build overwrites the generated source/binary.
- Rust source downloads and Cargo registry/index state can be warm in Cargo's global cache. Compiled dependency artifacts for the runtime-generated Cargo project are placed in a unique per-session `target` directory, so they are not reused by a later session.
- Cargo's normal development profile enables incremental compilation for workspace/path members by default outside CI, but the runtime-generated project is a fresh unique project directory per `RustSession`; dependency builds still cannot benefit across sessions unless the target directory is shared. The direct `rustc` route bypasses Cargo profiles and does not enable incremental compilation.
- The UI/kernel process is persistent, but the compile process and execution binary are not. The JSONL server is persistent as a transport/kernel process, not as a compiler.

### Safe cache candidates

1. **Compiled dependencies:** share a Cargo target directory keyed by compiler version, host/target triple, profile, dependency manifest/lockfile, enabled features, and rustflags. Keep notebook code in isolated workspaces and verify concurrent-build behavior.
2. **Generated source and cell artifacts:** hash the complete generated source, dependency graph/lock, compiler identity, target, profile, environment flags, and output-wrapper mode. Reuse a binary only on a full key match. It still must execute to reproduce current side effects.
3. **Compiler incremental state:** test `rustc -C incremental=<session-cache>` for direct builds, or a stable per-session Cargo target directory for dependency builds. Incremental compilation is designed to reuse compiler work, but generated `main` changes after each cell; measure before adopting.
4. **Execution results:** unsafe as a default. Output reuse is valid only for cells with explicit purity/determinism guarantees and a complete dependency/state fingerprint. Notebook cell outputs depend on earlier state and can include I/O, random values, time, environment, network, or mutable globals. Do not cache arbitrary results.

Invalidate compile artifacts when source, generated wrappers, dependencies/lockfile, compiler version, target, build profile, compiler flags, relevant environment, or ABI-affecting options change. Runtime results additionally depend on all prior state and external side effects, so source hashes alone are insufficient.

## 7. Cargo and compiler configuration review

- Workspace uses resolver 2, edition 2024, workspace package metadata, and exact pins for common Rust dependencies. No workspace `[profile.*]` overrides, `.cargo/config.toml`, explicit linker, target directory, or runtime compiler flags were found.
- Workspace sets `rust-version = 1.85` while `rust-toolchain.toml` pins 1.99.1. That is reasonable for consumers with an MSRV declaration and local reproducibility, but the toolchain pin means development follows that exact toolchain.
- Mars's app builds are not directly relevant to every cell's latency: runtime invokes a bare `rustc` with default codegen settings for dependency-free code. Cargo's workspace profile settings do not automatically alter that direct `rustc` invocation.
- For development crates, `cargo check -p mars-runtime` is a useful faster feedback loop when code generation/linking is not needed. It cannot measure notebook compile latency.
- Quick experiments worth measuring: `-C incremental` for the direct runtime; a stable Cargo target cache for dependencies; linker timing/choice on the supported macOS versions; avoiding a second display fallback compile. Do not default to aggressive optimization flags: they often add compile/link time and make interactive code slower to reach its first result.

## 8. Execution strategy evaluation

| Approach                                             | Fit for Mars                                                                                                                                                                                                                                                                                                               | Main trade-off                                                                                                                                                                                                                                                                                                                                                                                                  |
| ---------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Current direct `rustc` replay                        | Simple, no runtime crate dependency, real Rust compiler semantics, process isolation and easy reset.                                                                                                                                                                                                                       | Recompiles/relinks all cells each run; replays side effects; no cross-session compile cache.                                                                                                                                                                                                                                                                                                                    |
| Current Cargo replay                                 | Supports notebook dependencies and lock/offline controls; Cargo can reuse artifacts within a session.                                                                                                                                                                                                                      | Cargo process/manifest resolution each run; per-session target directory prevents compiled dependency reuse across sessions.                                                                                                                                                                                                                                                                                    |
| Persistent subprocess around current replay compiler | Avoids restarting the kernel transport but not rustc or the generated binary per cell.                                                                                                                                                                                                                                     | Does not fix the primary cost unless compiler artifacts/state are also reused.                                                                                                                                                                                                                                                                                                                                  |
| Stable rustc incremental compilation                 | Low architecture disruption and may reuse compiler query work within a session.                                                                                                                                                                                                                                            | Still compiles and links each changed complete program; correctness/cache key details matter; expected gain must be benchmarked.                                                                                                                                                                                                                                                                                |
| Dynamic library per-cell evaluation                  | Can retain a host process and load new code, as demonstrated by Evcxr.                                                                                                                                                                                                                                                     | Rust has no stable general ABI for arbitrary types across separately compiled cell libraries. Needs code generation, type/state transfer, symbol and library lifecycle, toolchain matching, and crash recovery. Not a drop-in backend.                                                                                                                                                                          |
| Evcxr                                                | Strongest existing Rust notebook/REPL starting point. Evcxr supports persistent variables/items, dependencies, output, and compilation caching. Its documented design uses syntax analysis, generated code, Cargo-built shared libraries, and a subprocess; this directly targets the missing interactive execution model. | It is a separate execution model with compiler-assisted variable storage/restoration and known language/tooling limitations. Requires `rust-src`; large dependencies may take time. Need test on Mars format, cancellation, cross-platform support, dependency declarations, failure recovery, macros, and output. Better first integrated as an isolated adapter/process than embedded into the Tauri process. |
| Cranelift codegen backend                            | Potential debug compile-time improvement, particularly for dependency-heavy Cargo builds. Apple Silicon macOS is listed as supported.                                                                                                                                                                                      | Distributed as a nightly rustc component; unsupported features include some SIMD and unwind scenarios. It does not solve full-source replay, repeated process launches, or state persistence. Treat as a controlled experiment, not architecture.                                                                                                                                                               |
| WASM compilation/execution                           | Useful for sandboxing language/runtime extensions and portable artifacts.                                                                                                                                                                                                                                                  | Rust variables/types cannot simply survive recompiling unrelated cells as a general persistent native Rust context. Adds WASM runtime and host ABI work, and performance/feature constraints. More relevant for explicit sandbox targets than first fix.                                                                                                                                                        |

Evcxr references: [project](https://github.com/evcxr/evcxr), [how it works](https://github.com/evcxr/evcxr/blob/main/HOW_IT_WORKS.md), [common usage and cache](https://github.com/evcxr/evcxr/blob/main/COMMON.md), [REPL requirements](https://github.com/evcxr/evcxr/blob/main/evcxr_repl/README.md). Evcxr documents persistent bindings and a configurable compilation cache; its internal design uses generated code and dynamically loaded shared objects while user code runs in a subprocess. It also documents that a crash/exit restarts the subprocess and loses variables, and that external macros cannot currently be imported.

Rust compiler references: [`rustc -C incremental`](https://doc.rust-lang.org/rustc/codegen-options/), [Cargo profiles and incremental builds](https://doc.rust-lang.org/cargo/reference/profiles.html), and [rustc Cranelift backend](https://github.com/rust-lang/rust/tree/main/compiler/rustc_codegen_cranelift). The Cranelift backend is a nightly component; its documentation lists support limitations, so it should not be treated as a transparent stable compiler replacement.

## 9. Recommended architecture

Keep the current crate boundaries and `.mars` schema. Treat `mars-runtime` as a backend interface with selectable implementations: `ReplayRustRuntime` retained for a compatibility/debug fallback and an isolated persistent evaluator backend for interactive sessions. The desktop/CLI should communicate through the existing `mars-kernel` and protocol boundary, so an evaluator can run in a managed child process and be restarted without putting arbitrary notebook code in the Tauri process.

Prototype Evcxr as the first persistent backend candidate before building Mars-specific compiler state machinery. The spike should execute the getting-started example plus variable/function/struct declarations, ownership moves, compilation failures, panics, cancellation, reset, dependency addition, and stdout/stderr capture. Compare cold and warm cell latencies, memory growth, startup cost, and crash semantics against replay on macOS ARM64 and at least one CI platform. Use the existing runtime protocol adapter where possible; do not couple the `.mars` document format to Evcxr.

If Evcxr cannot meet product requirements, build or sponsor a Rust evaluation engine only after specifying the limits of cross-cell type/lifetime/macro semantics and version compatibility. No general-purpose dynamic library interface can provide an unrestricted stable Rust ABI for arbitrary notebook values.

## 10. Prioritised optimization roadmap

### Milestone 1 — Make the baseline trustworthy

- Repair/clear the malformed output in `getting-started.mars` in a separate implementation change; keep this report's preserved working-tree version as the user's current data.
- Add runtime phase timing for source generation, dependency preparation, compile, link if separately observable, child start, execution, and output conversion.
- Extend the benchmark to emit every cell's latency and JSON/CSV output, and include unchanged-cell, first-cell, dependency, function/struct, and numerical workloads.
- Gather at least 20 independent runs for tail latency and capture CPU model, memory, Rust/Cargo version, profile, cache state, and commit.

### Milestone 2 — Low-risk compiler/cache experiment

- Prototype `rustc -C incremental` with a stable per-session directory.
- Prototype a content-keyed compiled-program cache for unchanged complete source and a shared Cargo target cache for matching dependency/toolchain/profile fingerprints.
- Measure cold/warm, unchanged-cell, new-cell, dependency-change, and multi-notebook cases; reject optimizations that only improve synthetic microbenchmarks.
- Keep result caching disabled.

### Milestone 3 — Persistent evaluator comparison

- Build an Evcxr child-process adapter behind `mars-runtime`'s public interface.
- Map requests, outputs, error diagnostics, dependencies, reset, timeout, and cancellation. Test crashes and session recovery.
- Run parity and latency comparisons. Decide whether Evcxr can be a supported backend or only an optional experimental backend.

### Milestone 4 — Select production semantics

- If Evcxr passes parity and platform gates, adopt the isolated process backend and keep replay for explicit deterministic/headless fallback as needed.
- If it does not, define and implement a Mars-specific persistent evaluator with documented Rust feature/ABI boundaries.
- Update runtime docs and benchmark gates; preserve process isolation and reject arbitrary user code execution in the desktop process.

## 11. Risks and trade-offs

- Shared compilation caches need complete invalidation keys, cross-process coordination, cleanup/quotas, and must not mix compiler versions or dependency configurations.
- Incremental compilation may reduce compile work but can grow disk use and still relink/relaunch each cell; it may not reach interactive latency targets.
- Evcxr may reduce replay costs but adds an upstream dependency and its own semantics/toolchain constraints. Its documented macro limitation and `rust-src` requirement matter for a standalone product.
- A long-lived evaluator preserves state more directly, but panic, abort, segfault, cancellation, and dependency changes need defined behavior. Restart may necessarily lose state.
- Replay currently offers a clean isolation/reset boundary and only commits successful cells; replacing it changes side-effect and partial-state behavior.
- Cranelift is nightly and has codegen/runtime gaps; WebAssembly is a sandbox/backend choice, not a direct solution to persistent Rust lexical state.
- The current sample output issue can make users misdiagnose execution performance as a 30-second runtime timeout when the document is rejected before execution.

## 12. Final recommendations

1. **Why is Mars currently slow?** It recompiles and relinks the complete accumulated cell history, launches the resulting executable, and replays earlier side effects after every new cell. The measured small-cell cost is around 0.4 s per cell on this machine.
2. **Which bottleneck first?** Whole-program replay compilation. First add phase instrumentation, then test incremental compilation and identical-artifact reuse without weakening isolation.
3. **Can the current architecture be optimized?** Partly: cache dependencies, enable/test compiler incremental state, and reuse identical artifacts. Those measures cannot remove the growing replay cost or repeated effects, so the interactive architecture ultimately needs a persistent evaluator.
4. **Build an engine or integrate Evcxr?** Prototype Evcxr first as a child-process backend. Do not make it mandatory until parity, latency, support, and licensing/toolchain integration are established. Building a new general Rust REPL should be a fallback decision, not the first response to this bottleneck.
5. **Long-term persistent execution strategy?** A long-lived isolated Rust evaluator process with typed request/output protocol, compiler-backed state handling, explicit reset/recovery, and a shared dependency/artifact cache. The UI and CLI remain clients of `mars-kernel`; notebook format stays runtime-neutral.
6. **Next iteration:** fix/validate the getting-started output; add stage-level timings and a statistically useful benchmark; run the `-C incremental` and shared Cargo target experiments; then deliver an Evcxr adapter spike and compare behavior/performance before selecting the production backend.

## 13. Benchmarking methodology and exact rerun commands

Build and run the current release harness:

```bash
cargo build -p mars-runtime --release --bin runtime_benchmark
for n in 8 16 32; do
  target/release/runtime_benchmark "$n"
done
```

The harness reports aggregate total/average and last-cell latency. For a proper p50/p95, run at least 20 independent trials per workload after one warm-up, retain every per-cell sample, report median and nearest-rank p95, and separate cold and warm caches. Record whether `target` and Cargo registry caches were pre-populated. Add explicit workloads for simple expression, repeated identical expression, cross-cell variable, function/struct, dependency, and numerical computation. Keep compile and execution phase timings separately.

To measure CLI kernel startup outside the benchmark:

```bash
target/debug/mars kernel serve </dev/null
```

Repeat with an external timer and use a separate native-app readiness harness for desktop launch; process exit time is not a valid startup metric for an app that stays open. For a dependency workload, use a pinned lockfile and run once online (resolution/build) and repeatedly offline/warm; do not compare an unpinned network fetch to a warm dependency cache.
