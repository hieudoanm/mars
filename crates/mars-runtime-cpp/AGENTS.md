# mars-runtime-cpp — C++ Execution Backend

Executes C++ code interactively with shared state across cells. Depends on
`mars-core` and `mars-runtime`.

## Architecture

- `CppSession` (`session.rs`) implements `RuntimeBackend` using a replay-based
  approach similar to the Rust runtime:
  - Accumulates cell sources into a single `main()` program
  - Compiles with `clang++` (falls back to `g++` if clang unavailable)
  - Executes the compiled binary as a subprocess with process-group isolation
  - Captures stdout/stderr and compiler diagnostics
- The session maintains state in a temporary workdir under
  `temp_dir()/mars-runtime-cpp-<uuid>`, removed on `Drop`.
- Cell sources are wrapped in a `main()` function with necessary includes
  accumulated from previous cells.
- The last expression of a cell is automatically displayed using `std::cout`
  (similar to Rust's Debug display).
- Dependency management via Conan (preferred) or vcpkg:
  - `metadata.dependencies` in the notebook maps to package names and versions
  - Generates temporary `conanfile.txt` or `vcpkg.json` for the build
  - Lockfile support via `conan.lock` for reproducible builds

## Compiler Resolution

`MARS_CPP_COMPILER` env var wins; otherwise probes `clang++`, `g++` with
`--version`. Prefers `clang++` for better error messages and diagnostics.

## Key modules

- `session.rs` — `CppSession` workdir management, cell accumulation,
  execution_count tracking, dependencies, and stdin handles.
- `compilation.rs` — C++ compilation command construction, header
  accumulation, and linking with Conan/vcpkg libraries.
- `diagnostics.rs` — Parses compiler error/warning messages and maps them
  to notebook cell positions.
- `manifest.rs` — Generates Conan/vcpkg manifest files from notebook
  dependencies.

## Rules

- Never execute user code in the host process — always subprocess isolation.
- Timeout and cancellation must kill the whole process tree, not just the direct child.
- Keep `RuntimeBackend` implementation honest: `reset()` clears all state;
  `execution_count` increments only on successful evaluation.
- C++17 is the minimum supported standard; C++20/23 support depends on compiler.
- Header-only libraries are handled specially (no linking required).
- No `unwrap()`/`expect()` in production paths; errors go through `RuntimeError`.

## Testing

```bash
cargo test -p mars-runtime-cpp   # requires clang++ or g++ on PATH
cargo clippy -p mars-runtime-cpp --all-targets -- -D warnings
```

Line and function coverage must stay at or above 80% — verify with `cargo llvm-cov -p mars-runtime-cpp --tests` (workspace gate: `make coverage`).

## Limitations

- Replay-based approach recompiles all cells on each execution (no true REPL).
- Compilation is slower than Rust due to C++ build model.
- Complex C++ template errors may be difficult to map to specific cells.
- Dependency management is more complex than Cargo (Conan/vcpkg have different workflows).
