---
title: C++ Runtime
description: Mars C++ runtime documentation
order: 8
author: Mars Team
tags: [runtime, cpp, execution, conan, vcpkg]
---

# C++ Runtime

Mars provides a C++ execution runtime that uses a replay-based approach
similar to the Rust runtime. It accumulates cell sources into a single `main()`
program, compiles with `clang++` (or `g++`), and executes the compiled binary
in an isolated subprocess.

## Overview

The C++ runtime (`mars-runtime-cpp`) implements the `RuntimeBackend` trait
with a replay-based execution model:

- Accumulates cell sources into a single `main()` program
- Compiles with `clang++` (falls back to `g++` if unavailable)
- Executes the compiled binary as a subprocess with process-group isolation
- Captures stdout/stderr and compiler diagnostics
- Maintains state in a temporary workdir (removed on `Drop`)
- Automatic display of the last expression via `std::cout`
- Dependency management via Conan (preferred) or vcpkg

## Installation

A C++ compiler must be installed and available on your system:

```bash
# Install clang++ (preferred) or g++
# macOS: xcode-select --install
# Linux: sudo apt install clang++ or sudo apt install g++
# Windows: Install Visual Studio or MinGW
```

Mars discovers the C++ compiler via:

1. The `MARS_CPP_COMPILER` environment variable (takes precedence)
2. The `clang++` command on PATH (preferred)
3. The `g++` command on PATH (fallback)

Verify installation:

```bash
clang++ --version
# or
g++ --version
```

## Creating a C++ Notebook

```bash
mars new analysis.mars cpp
```

Or set the language in notebook metadata:

```json
{
  "metadata": {
    "language": "cpp"
  }
}
```

## Execution Model

### Replay-based Compilation

The C++ runtime uses a replay approach similar to the Rust runtime:

1. Stores the source of each successful cell
2. Generates a C++ program containing the accumulated source
3. Wraps the code in a `main()` function with necessary includes
4. Compiles the generated program with `clang++` or `g++`
5. Starts the compiled child process in an isolated process group
6. Captures stdout, stderr, diagnostics, and duration
7. Commits the current cell only after successful completion

This provides isolation and reproducibility at the cost of recompilation and
replayed side effects.

### Persistent Session

```cpp
// Cell 1
int x = 42;

// Cell 2
x + 10  // Returns 52, x is still available
```

Variables defined in earlier cells remain available in later cells through the
accumulated source.

### Automatic Display

The last expression of a cell is automatically displayed using `std::cout`:

```cpp
std::vector<int> numbers = {1, 2, 3, 4, 5};
int sum = 0;
for (int n : numbers) sum += n;
sum  // Displays: 15
```

## Dependencies

C++ dependency management is more complex than Rust's Cargo. Mars supports two
package managers:

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

Mars generates a temporary `conanfile.txt` for the build.

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

Mars generates a temporary `vcpkg.json` for the build.

### Lockfile Support

Conan lockfiles (`conan.lock`) can be embedded in notebook metadata for
reproducible builds:

```json
{
  "metadata": {
    "cpp_lockfile": "..."
  }
}
```

## C++ Standard

- **Minimum supported**: C++17
- **C++20/23**: Support depends on compiler availability
- Use `// mars: cpp_version 20` cell directive to specify a newer standard

## Compiler Selection

Prefer `clang++` for better error messages and diagnostics:

```bash
export MARS_CPP_COMPILER=clang++
```

Or let Mars auto-detect: it probes `clang++` first, then falls back to `g++`.

## Known Limitations

- **Replay compilation**: Recompiles all cells on each execution (no true REPL)
- **Compilation speed**: Slower than Rust due to C++ build model
- **Template errors**: Complex C++ template errors may be difficult to map to specific cells
- **Dependency complexity**: Conan/vcpkg workflows are more complex than Cargo
- **Header-only libraries**: Require special handling (no linking required)
- **Timeout/cancel**: A timed-out or cancelled cell kills the process; the next
  cell starts fresh and loses all state

## C++ Data Science Ecosystem

C++ has a growing data science ecosystem that works well in Mars notebooks:

| Domain | C++ Libraries |
| --- | --- |
| Data manipulation | pandas-cpp, xtensor |
| Machine learning | mlpack, LightGBM, XGBoost |
| Plotting | matplotlib-cpp, Plot-CPP |
| Numerical computing | Eigen, Armadillo, Blaze |
| Web scraping | cpp-httplib, cpr |

## Comparison with Other Runtimes

| Feature | C++ | Rust | Python |
| --- | --- | --- | --- |
| Execution model | Replay compilation | Replay compilation | Persistent driver |
| Package manager | Conan/vcpkg | Cargo | pip |
| Compilation speed | Slower | Moderate | None (interpreted) |
| Memory safety | Manual | Guaranteed | Runtime |
| Dependency pinning | conan.lock | Cargo.lock | requirements.txt |

## Troubleshooting

### Compiler not found

```
no C++ compiler found; install clang++ or g++ or set MARS_CPP_COMPILER
```

Solution: Install a C++ compiler or set the environment variable:

```bash
export MARS_CPP_COMPILER=/path/to/clang++
```

### Conan not found

```
conan command not found
```

Solution: Install Conan:

```bash
pip install conan
```

### Template errors hard to read

Complex C++ template errors can be verbose. Use:

- `clang++` for better error messages (preferred over `g++`)
- `-ferror-limit` compiler flag to limit output
- Cell directives to isolate problematic code

### Session lost after timeout

If a cell times out or is cancelled, the compiled process is killed and the
next cell starts with a fresh session, losing all previous state. This is a
known limitation of the replay-based approach.

## Desktop Support

The desktop application supports C++ cells with:

- C++ syntax highlighting via CodeMirror
- Full kernel integration (execution, variables, diagnostics)
- Rich output rendering (text, errors)
- Compiler diagnostic mapping to notebook cells

## VS Code Support

The VS Code extension currently supports Rust execution only. C++ notebooks
should use the desktop application for full functionality.

## Example Notebook

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

## Advanced Features

### Cell Directives

```cpp
// mars: cpp_version 20
// mars: cpp_std 20
// mars: timeout 10000
```

### Header Accumulation

The runtime automatically accumulates `#include` directives from previous cells:

```cpp
// Cell 1
#include <vector>

// Cell 2
#include <algorithm>
// Both headers are available
```

### Standard Library

Full C++ standard library is available (C++17 minimum):

```cpp
#include <iostream>
#include <vector>
#include <string>
#include <map>
#include <set>
#include <algorithm>
#include <numeric>
#include <functional>
```
