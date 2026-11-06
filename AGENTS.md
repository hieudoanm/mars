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

Prioritise Rust as the primary programming language. Design the architecture so that additional language runtimes can be integrated in the future.

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

Suggested structure:

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
│   ├── mars-kernel/
│   ├── mars-protocol/
│   ├── mars-renderer/
│   ├── mars-jupyter/
│   └── mars-cli/
│
├── apps/
│   └── desktop/
│
├── examples/
│   ├── getting-started.mars
│
├── tests/
│
└── docs/
    ├── architecture.md
    ├── notebook-format.md
    ├── runtime.md
    └── development.md
```

You may adjust this structure when there is a strong architectural reason.

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

### mars-kernel

Responsible for:

- Kernel lifecycle.
- Execution requests.
- Execution responses.
- Kernel state.
- Session management.
- Communication with execution runtimes.

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

### mars-cli

Responsible for:

- Creating notebooks.
- Opening notebooks.
- Executing notebooks.
- Exporting notebooks.
- Inspecting notebook metadata.
- Running notebooks in headless mode.

## 4. Notebook Format

Create an independent JSON-based notebook format using `.mars`.

Example:

```json
{
  "format_version": 1,
  "metadata": {
    "title": "My First Mars Notebook",
    "language": "rust"
  },
  "cells": [
    {
      "id": "cell-001",
      "cell_type": "markdown",
      "source": "# Welcome to Mars",
      "metadata": {}
    },
    {
      "id": "cell-002",
      "cell_type": "code",
      "source": "let numbers = vec![1, 2, 3, 4, 5];\nnumbers",
      "metadata": {},
      "execution_count": null,
      "outputs": []
    }
  ]
}
```

Requirements:

- Every cell must have a stable unique ID.
- Support Markdown cells.
- Support code cells.
- Support raw cells for future compatibility.
- Preserve execution outputs.
- Preserve cell metadata.
- Support notebook-level metadata.
- Validate documents before saving.
- Support format migrations.
- Use Serde for serialization.
- Implement atomic file saving to avoid corrupting notebooks.

Do not tightly couple the document format to Rust execution. The document model must eventually support multiple programming languages.

## 5. Rust Execution Engine

This is one of the most important components.

Mars must execute Rust code interactively.

Example notebook:

Cell 1:

```rust
let numbers = vec![1, 2, 3, 4, 5];
```

Cell 2:

```rust
let sum: i32 = numbers.iter().sum();
sum
```

Cell 3:

```rust
let mean = sum as f64 / numbers.len() as f64;
mean
```

The variables defined in earlier cells must remain accessible in subsequent cells.

Requirements:

- Persistent Rust execution sessions.
- Cell-by-cell execution.
- Shared execution state.
- Capture stdout.
- Capture stderr.
- Capture compilation errors.
- Capture runtime errors.
- Return expression results.
- Track execution counts.
- Measure execution duration.
- Support interruption of long-running code.
- Support resetting the execution session.
- Support dependency management.
- Support Cargo-compatible dependencies.

Investigate practical Rust execution approaches before implementing the runtime.

Consider:

1. Embedded Rust compiler integration.
2. Persistent compilation and execution.
3. Long-lived subprocess execution.
4. Existing incremental Rust execution technologies.

Evaluate trade-offs around compilation latency, variable persistence, memory safety, portability, and implementation complexity.

Do not pretend that executing each cell as an independent Rust executable provides persistent variable state.

Select a technically realistic approach and document its limitations.

Do not execute arbitrary user code inside the main desktop process.

Design runtime isolation from the beginning.

## 6. Notebook User Interface

Build a polished desktop notebook application.

Preferred stack:

- Tauri 2.
- React.
- TypeScript.
- Tailwind CSS v4.
- CodeMirror 6.
- Rust backend.

The UI should feel like a professional development environment rather than a basic code editor.

### Design direction

Create a modern, minimalist interface inspired by scientific computing environments and professional developer tools.

Visual characteristics:

- Dark theme as the default.
- Deep charcoal backgrounds.
- Subtle Mars-inspired red/orange accents.
- Clean typography.
- Carefully designed spacing.
- Minimal borders.
- Rounded panels.
- Smooth interactions.
- Clear visual hierarchy.
- No excessive gradients.
- No unnecessary visual decoration.

The interface must be responsive.

### Main layout

Implement:

1. Application title bar.
2. File navigation.
3. Notebook editor.
4. Cell toolbar.
5. Execution controls.
6. Output renderer.
7. Status bar.
8. Optional sidebar.

### Notebook editor

Users must be able to:

- Create Markdown cells.
- Create Rust code cells.
- Edit cells.
- Delete cells.
- Duplicate cells.
- Reorder cells.
- Execute individual cells.
- Execute all cells.
- Execute cells above.
- Execute cells below.
- Clear cell outputs.
- Restart the runtime.
- Save notebooks.
- Open existing notebooks.

### Keyboard shortcuts

Implement common shortcuts:

- Shift + Enter: Execute cell and select next cell.
- Ctrl/Cmd + Enter: Execute cell.
- Alt + Enter: Execute cell and insert a new cell below.
- A: Insert cell above when in command mode.
- B: Insert cell below when in command mode.
- D, D: Delete selected cell.
- M: Convert to Markdown.
- Y: Convert to code.
- S: Save notebook.
- Escape: Enter command mode.

Make shortcuts configurable in the future.

### Code editor

Use CodeMirror 6.

Requirements:

- Rust syntax highlighting.
- Line numbers.
- Code folding.
- Bracket matching.
- Autocomplete foundations.
- Error highlighting.
- Configurable editor font size.
- Dark theme.
- Basic editor preferences.

Keep editor state separate from notebook document state.

## 7. Output Rendering

Support different output types.

### Text output

```text
Hello from Mars!
```

### Error output

Show compilation and runtime errors with useful diagnostics.

### Markdown output

Render Markdown with headings, lists, tables, links, and code blocks.

### Mathematical equations

Support LaTeX using KaTeX.

Example:

```latex
E = mc^2
```

### HTML output

Support controlled HTML rendering.

Sanitise untrusted HTML and prevent unsafe script execution.

### Images

Support PNG, JPEG, and SVG where appropriate.

### Tables

Support structured tabular output.

### Charts

Establish a rendering architecture that can support:

- Line charts.
- Bar charts.
- Scatter plots.
- Histograms.
- Pie charts.

The output model should be extensible without requiring changes to the notebook document schema for every new output type.

## 8. CLI

Mars must work without the desktop application.

Provide a CLI named `mars`.

Suggested commands:

```bash
mars new analysis.mars

mars open analysis.mars

mars run analysis.mars

mars export analysis.mars html analysis.html

mars export analysis.mars ipynb analysis.ipynb

mars import notebook.ipynb

mars inspect analysis.mars

mars kernel list

mars kernel restart
```

The CLI should:

- Provide helpful errors.
- Support meaningful exit codes.
- Display execution progress.
- Support headless execution.
- Produce reproducible execution results where practical.

Do not implement commands that do not have functional backing.

## 9. Jupyter Compatibility

Mars should be independent but interoperable.

Implement `.ipynb` import and export.

Map:

- Code cells.
- Markdown cells.
- Raw cells.
- Outputs.
- Execution counts.
- Metadata.

Preserve information that Mars cannot currently interpret.

Document any unsupported features.

Do not claim full Jupyter compatibility until it has been tested against representative notebook files.

## 10. Testing

Testing is mandatory.

### Unit tests

Cover:

- Notebook serialization.
- Notebook deserialization.
- Cell operations.
- Cell IDs.
- Document validation.
- Metadata preservation.
- Execution state.
- Output conversion.
- Error handling.

### Integration tests

Cover:

- Creating a notebook.
- Saving a notebook.
- Opening a notebook.
- Executing sequential cells.
- Shared variable state.
- Runtime restart.
- Execution failure.
- Notebook import.
- Notebook export.
- CLI execution.

### End-to-end tests

Test the primary desktop workflows:

1. Launch Mars.
2. Create a notebook.
3. Add a Markdown cell.
4. Add a Rust cell.
5. Execute the cell.
6. Verify the output.
7. Execute another cell using an earlier variable.
8. Save and reopen the notebook.

## 11. Documentation

Create:

- README.md
- CONTRIBUTING.md
- LICENSE
- Architecture documentation.
- Notebook format specification.
- Runtime architecture documentation.
- Installation instructions.
- Development instructions.
- CLI reference.
- Known limitations.

The README should explain:

- What Mars is.
- Why it exists.
- How it differs from Jupyter.
- Features.
- Screenshots or placeholders for screenshots.
- Installation.
- Quick start.
- Roadmap.
- Contribution instructions.

Use MIT as the proposed licence unless the repository already specifies another licence. Do not overwrite an existing licence without checking.

## 12. Development Strategy

Do not attempt to implement every feature simultaneously.

Follow this order:

### Phase 1 — Foundation

- Initialise the Cargo workspace.
- Define core domain types.
- Implement the `.mars` format.
- Implement notebook serialization.
- Implement cell management.
- Add unit tests.

### Phase 2 — Execution

- Research and select a Rust execution strategy.
- Implement persistent execution sessions.
- Implement sequential cell execution.
- Capture output and errors.
- Implement runtime lifecycle.
- Add integration tests.

### Phase 3 — CLI

- Implement notebook creation.
- Implement notebook inspection.
- Implement notebook execution.
- Implement notebook export.

### Phase 4 — Desktop

- Initialise Tauri.
- Build the React application.
- Integrate CodeMirror.
- Implement notebook editing.
- Connect the execution engine.
- Implement save and open functionality.

### Phase 5 — Rendering

- Implement Markdown.
- Implement mathematical equations.
- Implement rich output.
- Implement tables.
- Add chart rendering.

### Phase 6 — Compatibility

- Implement Jupyter import.
- Implement Jupyter export.
- Add compatibility tests.

### Phase 7 — Polish

- Improve UX.
- Add keyboard shortcuts.
- Add autosave.
- Improve error diagnostics.
- Add performance optimisations.
- Complete documentation.

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

When a feature cannot be implemented reliably, explain the limitation and provide a realistic alternative.

## 14. Working Instructions for Codex

First inspect the existing repository.

If it is empty, initialise the project.

If it already contains code, preserve useful existing work and follow its conventions.

Before implementing:

1. Inspect the project structure.
2. Check the available Rust and Node.js toolchains.
3. Evaluate Rust execution technologies.
4. Establish the architecture.
5. Create a development plan.

Then begin implementation.

Do not stop after producing an architecture proposal.

Implement working functionality incrementally.

After each phase:

- Run formatting.
- Run Clippy.
- Run relevant tests.
- Fix errors.
- Update documentation.
- Record completed work.

Use:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Run frontend checks and desktop builds where applicable.

If the full desktop stack cannot be built in the current environment, continue implementing and testing the independent Rust components.

Maintain a `PROGRESS.md` file documenting:

- Completed features.
- Current implementation phase.
- Outstanding tasks.
- Known limitations.
- Commands used to validate the project.

## 15. Definition of Done for the Initial MVP

The first usable Mars MVP must allow a user to:

1. Launch Mars from the desktop application.
2. Create a new `.mars` notebook.
3. Add and edit Markdown cells.
4. Add and edit Rust code cells.
5. Execute Rust code.
6. Preserve variables across sequential cells.
7. Display execution results.
8. Display compilation errors.
9. Restart the execution runtime.
10. Save and reopen notebooks.
11. Execute notebooks through the CLI.
12. Import and export basic `.ipynb` notebooks.

A beautiful interface without a functional execution engine is not a completed MVP.

Likewise, a working Rust execution engine without a usable notebook experience is not a completed MVP.

**The ultimate objective is to create a genuinely independent, production-quality, Rust-first notebook platform that can grow into a complete alternative to Jupyter.**
