# Runtime Architecture

## Purpose and guarantees

Project Mars runtimes provide persistent notebook sessions without running arbitrary notebook code inside the desktop process. All runtimes guarantee:

- Successful cells can define bindings used by later cells.
- Compilation and execution happen in a child process.
- Failed or interrupted cells are not committed to the session.
- The session can be reset without restarting the desktop application.
- A timeout can cover both compilation and execution.
- Cancellation cleans up compiler/runtime descendants on supported platforms.

No runtime provides instruction-level interruption inside arbitrary code. Once user code is running, cancellation can terminate the process, but code cannot observe a cooperative cancellation hook unless the notebook author implements one.

## Language dispatch

`mars-kernel::runtime::create_runtime` dispatches on the notebook language. Each cell's `metadata.language` overrides the notebook-level `metadata.language`; runtimes are language-agnostic `RuntimeBackend` implementations, so variables never leak across languages.

Supported languages:

- **Rust** — See [Rust runtime](./07-rust-runtime.md)
- **Python** — See [Python runtime](./08-python-runtime.md)
- **R** — See [R runtime](./09-r-runtime.md)
- **Julia** — See [Julia runtime](./10-julia-runtime.md)
- **C++** — See [C++ runtime](./11-cpp-runtime.md)

## Display output

Runtimes translate display markers in captured stdout. Libraries such as `plotters` print:

```text
EVCXR_BEGIN_CONTENT text/html
<div><svg ...></svg></div>
EVCXR_END_CONTENT
```

Project Mars removes the markers and turns a standalone SVG payload into an `image/svg+xml` MIME bundle (with a `text/plain` fallback), so saved notebook outputs render as figures instead of raw marker text. HTML that merely contains an SVG alongside visible content stays an HTML output.

## Multi-language notebooks

`mars notebook run` keeps one runtime per language found in the notebook. For example, a notebook with Rust, Python, and R cells maintains three separate runtime sessions. Cargo dependency options apply only to Rust cells. `mars kernel serve` binds a single language per connection (`--language` overrides the notebook's `metadata.language`).

## Performance considerations

Each runtime has different performance characteristics:

- **Rust replay**: Rebuilds accumulated source on each execution (~400ms per cell for small workloads)
- **Python**: Persistent interpreter, low latency for pure Python, overhead for C extension calls
- **R**: Persistent R session, compiles per cell, moderate latency
- **Julia**: Persistent Julia session, JIT compilation improves over time
- **C++**: Replay compilation, similar to Rust but with generally slower compilation

See individual runtime documentation for specific performance characteristics and benchmarks.

## Language-specific documentation

For detailed information about each runtime, see:

- [Rust runtime](./07-rust-runtime.md) — Replay and Evcxr backends, Cargo dependencies
- [Python runtime](./08-python-runtime.md) — Persistent driver, matplotlib, top-level await
- [R runtime](./09-r-runtime.md) — `.GlobalEnv` persistence, plot capture, jsonlite
- [Julia runtime](./10-julia-runtime.md) — MarsKernel module, MIME bundles, interactive input
- [C++ runtime](./11-cpp-runtime.md) — Replay compilation, Conan/vcpkg, clang++/g++ support
