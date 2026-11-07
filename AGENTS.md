# Build Mars — A Rust-Powered Interactive Notebook

You are a senior Rust engineer, software architect, and developer-tools designer. Your task is to design and implement **Mars**, an independent, Rust-powered interactive notebook platform.

Do not build another Jupyter frontend or simply implement a Rust kernel for Jupyter. Mars must have its own architecture, notebook format, execution engine, and user experience.

The goal is to create a high-quality open-source developer tool for interactive programming, scientific computing, data science, machine learning, experimentation, and education.

## 1. Product Vision

**Name:** Mars

**Tagline:** Explore ideas. Execute code. Discover possibilities.

**Primary language:** Rust

**Native notebook extension:** `.mars`

**Jupyter compatibility:** Import and export `.ipynb`

Mars should provide an interactive environment where users can combine executable code, Markdown, mathematical equations, tables, charts, and rich outputs in a single document.

Prioritise Rust as the primary programming language. Additional language runtimes (Python, Julia, R, C++) are integrated through the `RuntimeBackend` trait. The platform also supports Model Context Protocol (MCP) for AI assistant integration.

## 2. Technical Principles

Follow these principles throughout development:

- Rust-first architecture.
- Modular Cargo workspace.
- Separation of document management, execution, rendering, and UI.
- Strong type safety.
- No unnecessary dependencies.
- Avoid `unwrap()` and `expect()` in production code.
- No `any` in TypeScript.
- Use idiomatic Rust and TypeScript.
- Prefer maintainable abstractions over premature optimisation.
- Use asynchronous execution where appropriate.
- Provide comprehensive error handling.
- Write tests for all important functionality.
- Ensure cross-platform compatibility where practical.
- Document public APIs.
- Keep the project easy to build, test, and contribute to.

Use the latest stable Rust toolchain and compatible dependency versions available during implementation.

## 3. Architecture

Create a modular Cargo workspace.

Current workspace structure:

```text
mars/
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── README.md
├── LICENSE
├── CONTRIBUTING.md
├── .gitignore
│
├── crates/
│   ├── mars-core/
│   ├── mars-document/
│   ├── mars-runtime/
│   ├── mars-runtime-evcxr/
│   ├── mars-runtime-python/
│   ├── mars-runtime-r/
│   ├── mars-runtime-julia/
│   ├── mars-runtime-cpp/
│   ├── mars-kernel/
│   ├── mars-protocol/
│   ├── mars-jupyter/
│   ├── mars-renderer/
│   ├── mars-lsp/
│   ├── mars-cli/
│   └── mars-mcp/
│
├── packages/
│   ├── desktop/
│   └── landing/
│
├── extensions/
│   ├── vscode/
│   └── zed/
│
├── examples/
│   ├── mars/
│   ├── jupyter/
│   ├── jupyter_evcxr/
│   ├── r/
│   └── julia/
│
└── docs/
```

### mars-core

Responsible for:

- Shared domain types.
- IDs and metadata.
- Common error types.
- Execution status.
- Shared interfaces.

### mars-document

Responsible for:

- Notebook representation.
- Cell management.
- Serialization and deserialization.
- Notebook validation.
- File persistence.
- Autosave support.
- Document versioning.
- Undo/redo foundations.

### mars-runtime

Responsible for:

- Rust code execution.
- Persistent execution sessions.
- Shared variables between cells.
- Dependency management.
- Capturing standard output and errors.
- Execution time tracking.
- Execution cancellation and interruption.
- Runtime isolation.

### mars-runtime-python

Responsible for:

- Python code execution via persistent `python3 -u` subprocess.
- Top-level `await` support.
- Matplotlib figure capture.
- `_repr_*` display methods.
- Interactive `input()` prompts.

### mars-runtime-r

Responsible for:

- R code execution via persistent `Rscript --vanilla` subprocess.
- `.GlobalEnv` binding persistence.
- Plot capture via graphics devices.
- Streaming warnings and messages.

### mars-runtime-julia

Responsible for:

- Julia code execution via persistent Julia subprocess.
- `MarsKernel` module for state management.
- Interactive `readline()` prompts.
- MIME bundle output for rich rendering.

### mars-runtime-cpp

Responsible for:

- C++ code execution via replay-based compilation.
- `clang++`/`g++` compiler support.
- Conan/vcpkg dependency management.
- Header accumulation and linking.

### mars-runtime-evcxr

Responsible for:

- Experimental Evcxr-based Rust execution.
- Faster re-execution without recompilation.
- Opt-in via `--features experimental-evcxr`.

### mars-kernel

Responsible for:

- Kernel lifecycle.
- Execution requests.
- Execution responses.
- Kernel state.
- Session management.
- Communication with execution runtimes.
- Multi-language runtime dispatch.

### mars-protocol

Responsible for:

- Communication between frontend and backend.
- Typed request and response messages.
- Execution events.
- WebSocket or JSON-RPC communication where appropriate.

### mars-renderer

Responsible for:

- Markdown rendering.
- HTML output.
- LaTeX mathematical equations.
- Tables.
- Images.
- Charts.
- Rich notebook outputs.

### mars-jupyter

Responsible for:

- Importing `.ipynb` notebooks.
- Exporting `.ipynb` notebooks.
- Mapping Jupyter cell types.
- Mapping notebook metadata.
- Preserving unsupported metadata where possible.
- Handling compatibility limitations gracefully.

### mars-lsp

Responsible for:

- Language Server Protocol client.
- rust-analyzer integration.
- Diagnostics mapping.
- Hover, completion, signature help.
- Go-to-definition and quick fixes.

### mars-cli

Responsible for:

- Creating notebooks.
- Opening notebooks.
- Executing notebooks.
- Exporting notebooks.
- Inspecting notebook metadata.
- Running notebooks in headless mode.
- Kernel lifecycle commands.
- MCP server commands.

### mars-mcp

Responsible for:

- Model Context Protocol server.
- Tool definitions and handlers.
- Session management for MCP clients.
- JSON-RPC 2.0 over stdio transport.

## 4. Notebook Format

Create an independent JSON-based notebook format using `.mars`.

## 5. Rust Execution Engine

Mars must execute Rust code interactively with persistent state across cells.

## 6. Notebook User Interface

Build a polished desktop notebook application using Tauri 2, React, TypeScript, and Tailwind CSS.

## 7. Output Rendering

Support different output types: text, errors, HTML, images, tables, LaTeX, charts, and MIME bundles.

## 8. CLI

Mars must work without the desktop application. Provide a CLI named `mars`.

## 9. Jupyter Compatibility

Mars should be independent but interoperable. Implement `.ipynb` import and export.

## 10. Testing

Testing is mandatory. Implement unit tests, integration tests, and end-to-end tests.

## 11. Documentation

Create comprehensive documentation including README.md, CONTRIBUTING.md, architecture docs, notebook format spec, runtime architecture, installation instructions, CLI reference, and known limitations.

## 12. Development Strategy

Follow the phases outlined in the original specification, but the project is currently in Phase 7 — Desktop and rendering polish with full multi-language support and MCP integration.

## 13. Important Engineering Constraints

- Do not build a monolithic application.
- Do not put business logic inside React components.
- Do not put notebook persistence logic inside the UI.
- Do not couple the Rust runtime to the desktop framework.
- Do not use Python as the primary execution engine.
- Do not use Evcxr as a mandatory dependency for Mars.
- Do not execute untrusted notebook code in the application process.
- Do not fake execution results.
- Do not leave core functionality as TODO placeholders.
- Avoid excessive abstraction.
- Keep dependencies justified.
- Ensure the CLI and desktop application can share the same core libraries.

## 14. Current Implementation Status

Mars is a mature, feature-rich notebook platform with:

- **Multi-language support**: Rust (replay + Evcxr), Python, R, Julia, C++
- **Rich desktop editor**: Tauri-based with CodeMirror 6, LSP integration, multi-notebook workspace
- **Developer tools**: rust-analyzer LSP, variable inspector, problems panel, checkpoints
- **Runtime helpers**: mars_table, mars_csv, mars_display_*, mars_chart, mars_prompt
- **Jupyter compatibility**: Import/export with metadata and output preservation
- **CLI**: Full command-line interface with kernel serve and MCP commands
- **MCP integration**: Model Context Protocol server for AI assistants
- **Editor extensions**: VS Code companion, Zed grammar support, JetBrains planned

See PROGRESS.md for detailed implementation history and outstanding tasks.
