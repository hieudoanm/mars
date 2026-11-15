# C++ Runtime

## Overview

`mars-runtime-cpp` implements a replay-based C++ execution similar to the Rust runtime: it accumulates cell sources into a single `main()` program, compiles with `clang++` (falling back to `g++`), and executes the compiled binary in an isolated subprocess. Variables persist across cells through accumulated source. The last expression is displayed via `std::cout`. Dependencies are managed via Conan (preferred) or vcpkg, with lockfile support for reproducible builds.

## Discovery

C++ runtime discovers the compiler via:

1. The `MARS_CPP_COMPILER` environment variable (takes precedence)
2. The `clang++` command on PATH (preferred for better diagnostics)
3. The `g++` command on PATH (fallback)

Verify installation:

```bash
clang++ --version
# or
g++ --version
```

## Execution model

The C++ runtime uses a replay-based approach:

- Accumulates cell sources into a single `main()` program
- Compiles with `clang++` (falling back to `g++`)
- Executes the compiled binary in an isolated subprocess
- Variables persist across cells through accumulated source
- Displays the last expression via `std::cout`
- Captures stdout/stderr and compiler diagnostics
- Manages dependencies via Conan (preferred) or vcpkg
- Supports lockfile for reproducible builds

Timeout or cancellation kills the compiled process; the next cell starts a fresh session.

## Compilation

### Standard version

The minimum supported standard is C++17. C++20/23 support depends on compiler availability:

```cpp
// mars: cpp_version 20
// or
// mars: cpp_std 20
```

### Header accumulation

The runtime automatically accumulates `#include` directives from previous cells:

```cpp
// Cell 1
#include <iostream>
#include <vector>

// Cell 2
#include <algorithm>
// Both headers are available
```

## Display support

### Trailing expression

The last expression of a cell is automatically displayed using `std::cout`:

```cpp
// Cell 1
std::vector<int> numbers = {1, 2, 3, 4, 5};

// Cell 2
int sum = 0;
for (int n : numbers) sum += n;
sum  // Displays: 15
```

### Explicit output

Use `std::cout` for explicit output:

```cpp
std::cout << "Hello from Project Mars!" << std::endl;
```

## Dependency management

### Conan (preferred)

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

Project Mars generates a temporary `conanfile.txt` for the build.

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

Project Mars generates a temporary `vcpkg.json` for the build.

### Lockfile support

Conan lockfiles (`conan.lock`) can be embedded in notebook metadata for reproducible builds:

```json
{
  "metadata": {
    "cpp_lockfile": "..."
  }
}
```

### Reproducible execution

```bash
mars notebook run analysis.mars --locked --offline
```

This requires `metadata.cpp_lockfile` to be present (set after first successful run).

## Configuration

### Compiler override

```bash
export MARS_CPP_COMPILER=clang++
```

### Conan installation

```bash
pip install conan
```

### vcpkg installation

```bash
git clone https://github.com/Microsoft/vcpkg.git
cd vcpkg
./bootstrap-vcpkg.sh
```

## Limitations

- No true REPL: Replay-based approach recompiles all cells on each execution
- Compilation latency: C++ compilation is slower than Rust
- Complex template errors: May be difficult to map to specific cells
- Dependency complexity: Conan/vcpkg workflows are more complex than Cargo
- Header-only libraries: Special handling required (no linking)
- Process isolation: Timeout/cancellation kills the entire process tree

## Example notebook

```cpp
// Cell 1: Basic types
#include <iostream>
#include <vector>
#include <algorithm>

std::vector<int> numbers = {5, 2, 8, 1, 9};

// Cell 2: Algorithms
std::sort(numbers.begin(), numbers.end());
for (int n : numbers) {
    std::cout << n << " ";
}
// Output: 1 2 5 8 9

// Cell 3: Custom function
auto sum = [](const std::vector<int>& v) {
    int total = 0;
    for (int n : v) total += n;
    return total;
};
sum(numbers)  // Output: 25
```

## Architecture rules

See `crates/mars-runtime-cpp/AGENTS.md` for architecture rules and implementation details.
