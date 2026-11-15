
# Getting started

Project Mars has multiple surfaces: a desktop application, a command-line interface,
and a VS Code extension. All of them use the same notebook format and execution
core. Project Mars supports Rust, Python, and R runtimes.

## Install the CLI

From the repository root:

```bash
./scripts/install.sh --cli
```

The script builds a fresh release binary and installs `mars` to
`~/.local/bin`. Make sure that directory is on your `PATH`:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

You can also run the CLI directly from the workspace:

```bash
cargo run -p mars -- --help
```

## Create a notebook

```bash
mars notebook new analysis.mars
```

This creates a `.mars` file with one Markdown cell. Specify a language:

```bash
mars notebook new analysis.mars rust    # Rust (default)
mars notebook new analysis.mars python  # Python
mars notebook new analysis.mars r       # R
```

Open it in the desktop app, VS Code, or edit the JSON directly.

You can also start from an existing example:

```bash
# Copy the getting-started example
cp examples/mars/rust/01-getting-started/getting-started.mars my-notebook.mars
```

## Run a notebook

```bash
mars notebook run analysis.mars
```

By default, results are written back to the notebook. Use `--no-save` for a
read-only check:

```bash
mars notebook run analysis.mars --no-save --report
```

Code cells execute in order and share bindings. A later cell can use a value
defined earlier:

```rust
let numbers = vec![1, 2, 3, 4, 5];
let total: i32 = numbers.iter().sum();
total
```

## Multi-language execution

Notebooks can mix languages by setting `metadata.language` at the notebook or
cell level:

```bash
# Create a Python notebook
mars notebook new analysis.mars python
# Create an R notebook
mars notebook new analysis.mars r
# Create a Julia notebook
mars notebook new analysis.mars julia
```

Each language runs in its own persistent session:
- **Python**: Supports `await`, matplotlib figure capture, and `_repr_*` display methods
- **R**: Supports plot capture and `.GlobalEnv` binding persistence
- **Julia**: Supports MIME bundle output, interactive `readline()`, and Julia plotting libraries

## Export a notebook

```bash
mars notebook export analysis.mars html analysis.html
mars notebook export analysis.mars md analysis.md
mars notebook export analysis.mars ipynb analysis.ipynb
```

Markdown export preserves Markdown cells, fences Rust code, and converts
tables, LaTeX, images, and offline charts into portable Markdown.

When the output path is omitted, Project Mars derives one from the input file name:

```bash
mars notebook export analysis.mars html   # Creates analysis.html
mars notebook export analysis.mars md     # Creates analysis.md
mars notebook export analysis.mars ipynb   # Creates analysis.ipynb
```

## Install the desktop app and VS Code extension

```bash
./scripts/install.sh --desktop
./scripts/install.sh --cli --vscode
```

The VS Code extension executes cells through the `mars` CLI, so keep the CLI
installed unless it is already available on `PATH`.

## Desktop application

The desktop app provides the full notebook experience:

- Multi-notebook workspace with tabs
- File explorer with Rust project file support
- CodeMirror 6 editor with rust-analyzer LSP integration
- Variable inspector, problems panel, and table of contents
- Checkpoints with diff view
- Rich output rendering (HTML, LaTeX, images, charts, tables)
- Jupyter-style keyboard shortcuts and command mode
- Runtime helpers (`mars_table`, `mars_csv`, `mars_display_*`) with autocomplete

Launch the desktop app:

```bash
cd packages/desktop
npm run tauri dev
```

Or from the workspace:

```bash
pnpm --filter @mars/desktop dev
```

## Explore examples

The repository includes example notebooks demonstrating different aspects of Project Mars:

| Example                                                                                                       | Description                                                                                                           |
| ------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| [01-getting-started](https://github.com/hieudoanm/mars/tree/main/examples/mars/rust/01-getting-started)       | A 57-cell tour of basic Rust: variables, types, control flow, collections, structs, enums, traits, closures, and more |
| [02-machine-learning](https://github.com/hieudoanm/mars/tree/main/examples/mars/rust/02-machine-learning)     | Machine learning workflow using Rust libraries (polars, ndarray, linfa)                                               |
| [03-plotting](https://github.com/hieudoanm/mars/tree/main/examples/mars/rust/03-plotting)                     | Vector plotting with plotters, rendered as SVG image output                                                           |
| [r-intro.mars](https://github.com/hieudoanm/mars/tree/main/examples/mars/r)                                   | R language introduction with plot capture                                                                             |
| [julia-intro.mars](https://github.com/hieudoanm/mars/tree/main/examples/mars/julia)                           | Julia language introduction with MIME bundle output                                                                   |
| [python getting-started](https://github.com/hieudoanm/mars/tree/main/examples/mars/python/01-getting-started) | Comprehensive Python tour: types, collections, functions, exceptions, classes, and stdlib tools                       |
| [julia getting-started](https://github.com/hieudoanm/mars/tree/main/examples/mars/julia/01-getting-started)   | Comprehensive Julia tour: arrays, broadcasting, dispatch, structs, and stdlib tools                                   |
| [r getting-started](https://github.com/hieudoanm/mars/tree/main/examples/mars/r/01-getting-started)           | Comprehensive R tour: vectors, data frames, apply family, environments, and statistics                                |
| [cpp getting-started](https://github.com/hieudoanm/mars/tree/main/examples/mars/cpp/01-getting-started)       | C++ curriculum documentation (not yet executable — no C++ backend)                                                    |

Run all Project Mars examples from the examples directory:

```bash
cd examples/mars
./run.sh
```

Or run a single example:

```bash
mars notebook run examples/mars/rust/01-getting-started/getting-started.mars --report
```

## Runtime helpers

Project Mars injects helper functions into Rust cells for rich output:

- `mars_table(&[columns], &[vec![rows]])` — Render a table
- `mars_csv(r#"csv"#)` — Parse CSV and render as a table
- `mars_display_html(&str)` — Render HTML (sanitized)
- `mars_display_latex(&str)` — Render LaTeX equations
- `mars_display_image(mime, &[u8])` — Render an image
- `mars_chart(spec_json)` — Render a Vega-Lite chart
- `mars_display_markdown(&str)` — Render Markdown
- `mars_display_mime(mime, data)` — Render arbitrary MIME bundles

These helpers appear in code-cell autocomplete with snippet placeholders.
