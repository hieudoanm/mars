# mars-runtime-cpp

C++ execution backend for Project Mars notebooks using replay-based compilation.

## Purpose

`mars-runtime-cpp` implements the `RuntimeBackend` trait for C++ code execution. It uses a replay-based approach similar to the Rust runtime:

- Accumulates cell sources into a single `main()` program
- Compiles with `clang++` (falls back to `g++`)
- Executes the compiled binary in an isolated subprocess
- Manages dependencies via Conan (preferred) or vcpkg
- Supports lockfile for reproducible builds

## Architecture

### Key Modules

- **`session.rs`** — CppSession workdir management, cell accumulation, execution_count tracking, dependencies, and stdin handles
- **`compilation.rs`** — C++ compilation command construction, header accumulation, and linking with Conan/vcpkg libraries
- **`diagnostics.rs`** — Parses compiler error/warning messages and maps them to notebook cell positions
- **`manifest.rs`** — Generates Conan/vcpkg manifest files from notebook dependencies

### Execution Model

1. Store the source of each successful cell
2. Generate a C++ program containing the accumulated source
3. Wrap the code in a `main()` function with necessary includes
4. Compile with `clang++` or `g++`
5. Execute the compiled binary in an isolated process group
6. Capture stdout, stderr, and diagnostics
7. Commit the cell only after successful completion

## Compiler Resolution

The runtime discovers the compiler via:

1. The `MARS_CPP_COMPILER` environment variable (takes precedence)
2. The `clang++` command on PATH (preferred for better diagnostics)
3. The `g++` command on PATH (fallback)

## Dependency Management

### Conan (Preferred)

Specify Conan dependencies in notebook metadata:

```json
{
  "metadata": {
    "cpp_package_manager": "conan",
    "dependencies": {
      "fmt": "10.0.0",
      "nlohmann_json": "3.11.0"
    }
  }
}
```

### vcpkg

Specify vcpkg dependencies in notebook metadata:

```json
{
  "metadata": {
    "cpp_package_manager": "vcpkg",
    "dependencies": {
      "fmt": "10.0.0",
      "nlohmann-json": "3.11.0"
    }
  }
}
```

### Lockfile Support

Conan lockfiles (`conan.lock`) can be embedded in notebook metadata for reproducible builds.

## Rules

- Never execute user code in the host process — always subprocess isolation
- Timeout and cancellation must kill the whole process tree, not just the direct child
- Keep `RuntimeBackend` implementation honest: `reset()` clears all state; `execution_count` increments only on successful evaluation
- C++17 is the minimum supported standard; C++20/23 support depends on compiler
- Header-only libraries are handled specially (no linking required)
- No `unwrap()`/`expect()` in production paths; errors go through `RuntimeError`

## Testing

```bash
cargo test -p mars-runtime-cpp   # requires clang++ or g++ on PATH
cargo clippy -p mars-runtime-cpp --all-targets -- -D warnings
```

## Limitations

- Replay-based approach recompiles all cells on each execution (no true REPL)
- Compilation is slower than Rust due to C++ build model
- Complex C++ template errors may be difficult to map to specific cells
- Dependency management is more complex than Cargo (Conan/vcpkg have different workflows)

## Related Documentation

- [C++ runtime documentation](../../docs/11-cpp-runtime.md)
- [Runtime architecture](../../docs/06-runtime.md)
- [AGENTS.md](./AGENTS.md) — Architecture rules and implementation details
