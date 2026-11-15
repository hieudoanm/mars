# Project Mars Editor Extensions

Project Mars provides editor extensions for popular code editors to enable `.mars` notebook editing and execution directly within your preferred development environment.

## Overview

Project Mars extensions integrate with:

- **VS Code** — Full notebook editor with execution, variables inspection, diagnostics, checkpoints, and export
- **Zed** — Syntax highlighting, grammar support, and MCP context-server integration
- **JetBrains IDEs** — Kotlin plugin with file type, cell execution via `mars kernel serve`, and a kernel tool window

All extensions are designed to work with the Project Mars CLI (`mars`) and share the same notebook format, ensuring consistency across editors.

## VS Code Extension

### Features

The VS Code extension provides the most comprehensive notebook experience:

- **Native notebook editor** — Open `.mars` files with Rust code and Markdown cells
- **Cell execution** — Run selected cells or entire notebooks with persistent kernel
- **Kernel integration** — Connects to `mars kernel serve` for execution
- **Variables inspector** — View live session variables in the sidebar
- **Diagnostics** — Compile-check cells without execution
- **Checkpoints** — Create and restore notebook snapshots
- **Export** — Export to HTML or Markdown with preview
- **Templates** — Create notebooks from built-in templates
- **Runtime selection** — Choose between `replay` and `evcxr` backends
- **Rich output** — Support for HTML, images, LaTeX, tables, and charts
- **Language configuration** — `.mars` language contribution and grammar
- **Completions** — Live kernel variables and Rust keywords
- **Multi-language execution** — `Execute.language` is sent per cell; cell-level `language` metadata and notebook-level `metadata.language` are honored (Rust remains the default)

### Installation

From the VS Code Marketplace:

```bash
code --install-extension mars.mars
```

Or from source:

```bash
cd extensions/vscode
pnpm install
pnpm --filter mars-vscode package
code --install-extension vsix/mars-vscode.vsix
```

### Requirements

- VS Code 1.145.2 or later
- Project Mars CLI on `PATH` or configured via `mars.cliPath`
- For development: Node.js and Corepack-managed pnpm

### Configuration

```json
{
  "mars.cliPath": "mars",
  "mars.executionTimeout": 30000,
  "mars.runArguments": [],
  "mars.previewAfterExport": true,
  "mars.runtime": "replay"
}
```

### Current Status

✅ Fully functional — Rust execution plus multi-language runtimes (Python, R, Julia, C++) via per-cell `Execute.language`

### Documentation

See [extensions/vscode/README.md](./vscode/README.md) for detailed usage instructions and [extensions/vscode/AGENTS.md](./vscode/AGENTS.md) for development rules.

## Zed Extension

### Features

The Zed extension provides:

- **Syntax highlighting** — `.mars` file syntax highlighting
- **Grammar support** — Rust code cell and Markdown cell highlighting
- **File icon** — Project Mars file icon in the file explorer
- **MCP context server** — `context_server_command` launches the Project Mars MCP server so Zed's agent can inspect and work with Project Mars notebooks

### Installation

From the Zed extension marketplace (when published):

```bash
# Via Zed's extension manager
# Search for "Project Mars"
```

Or from source:

```bash
cd extensions/zed
# Copy the extension directory to your Zed extensions folder
```

### Current Status

⚠️ Grammar, language config, and MCP integration implemented
❌ Notebook cell execution is not available — Zed has no notebook UI API, and `mars kernel serve` speaks the Project Mars kernel protocol rather than LSP

### MCP context server

The extension registers `mars` as a Zed context server that spawns the Project Mars CLI's MCP server, giving Zed's agent access to Project Mars notebook inspection and related tools. It requires the `mars` CLI on `PATH` (or a resolvable location) since the context server command launches it directly.

### Workaround

For notebook execution in Zed, use the CLI:

```bash
mars notebook run notebook.mars --report
```

Or configure an external tool in Zed to run `mars` on the current file.

### Documentation

See [extensions/zed/README.md](./zed/README.md) for details.

## JetBrains Extension

### Features

The JetBrains plugin (Kotlin) provides:

- **`.mars` file type** — JSON-aware syntax highlighting, folding, and structure view
- **Cell execution** — `Project Mars → Execute Cell at Caret` runs the code cell under the caret through `mars kernel serve`, honoring per-cell `language` metadata
- **Project Mars Kernel tool window** — kernel status, live variables, execution output, and stderr
- **Notebook actions** — New Notebook, Run Notebook, Export (HTML/Markdown/`.ipynb`), Inspect, Navigate Cells, Diagnostics, Checkpoints, Interrupt Execution, Restart Kernel
- **Configurable CLI path** — Settings → Tools → Project Mars

### Installation

```bash
cd extensions/jetbrains
./gradlew buildPlugin   # produces build/distributions/*.zip
```

Then install the zip via **Settings → Plugins → ⚙ → Install Plugin from Disk…**, or run `./gradlew runIde` for a sandbox IDE.

`scripts/install.sh --jetbrains` also builds the plugin and installs it into detected JetBrains IDEs.

### Current Status

✅ Functional plugin — file type, kernel protocol client, cell execution, tool window, and CLI-backed actions
⚠️ JetBrains IDEs have no notebook-cell editor API: cells are edited as raw JSON, and execution results appear in the tool window rather than inline outputs

### Supported IDEs

IntelliJ IDEA 2024.3+ Community/Ultimate, PyCharm, CLion, RustRover, and other JetBrains IDEs on the 2024.3 platform or newer (adjust `platformVersion` in `build.gradle.kts`).

### Documentation

See [extensions/jetbrains/README.md](./jetbrains/README.md) for development status and contribution guidelines.

## Common Requirements

All Project Mars extensions require:

- **Project Mars CLI** — Install from source or build the workspace:

```bash
cd /path/to/mars
cargo install --path crates/mars-cli
```

- **`.mars` format** — Extensions read and write the Project Mars notebook format
- **Runtime backends** — Language execution requires appropriate runtimes (Rust, Python, R, Julia, C++)

## Editor Compatibility

| Feature                | VS Code     | Zed | JetBrains |
| ---------------------- | ----------- | --- | --------- |
| Syntax highlighting    | ✅           | ✅   | ✅         |
| Notebook editor        | ✅           | ❌   | ⚠️ Raw JSON |
| Cell execution         | ✅           | ❌   | ✅         |
| Variables inspector    | ✅           | ❌   | ✅         |
| Diagnostics            | ✅           | ❌   | ✅         |
| Checkpoints            | ✅           | ❌   | ✅         |
| Export (HTML/MD)       | ✅           | ❌   | ✅         |
| Multi-language support | ✅           | ❌   | ✅         |
| MCP integration        | ❌           | ✅   | ❌         |

Notes:

- JetBrains cell execution is caret-based (no notebook UI); results display in the tool window and are not persisted to `outputs`.
- Zed has no notebook UI API and no LSP frontend; it exposes the Project Mars MCP server as a context server instead of executing cells.

Legend:

- ✅ Implemented
- ⚠️ Partially implemented
- ❌ Not implemented
- 📋 Planned

## Development

### VS Code Extension

```bash
cd extensions/vscode
pnpm install
pnpm --filter mars-vscode check
pnpm --filter mars-vscode lint
pnpm --filter mars-vscode test
pnpm --filter mars-vscode build
pnpm --filter mars-vscode package
```

### Zed Extension

```bash
cargo build --release --target wasm32-wasip1 -p mars-zed
# Then load it as a local extension in Zed, or use scripts/install.sh --zed
```

### JetBrains Extension

```bash
cd extensions/jetbrains
./gradlew compileKotlin
./gradlew buildPlugin
```

## Community Contribution

Contributions are welcome for all extensions:

- **VS Code** — Report issues, suggest features, submit PRs
- **Zed** — Improve grammar coverage and MCP integration; notebook execution would require an upstream notebook UI API
- **JetBrains** — Improve cell navigation, output persistence, and notebook editing UX within platform limits

See individual extension directories for contribution guidelines and development rules.

## Related Documentation

- [Landing site extensions documentation](../packages/landing/md/docs/)
  - [VS Code extension](../packages/landing/md/docs/10-extension-vscode.md)
  - [Zed extension](../packages/landing/md/docs/11-extension-zed.md)
  - [JetBrains extension](../packages/landing/md/docs/12-extension-jetbrains.md)
- [Project Mars CLI documentation](../docs/12-cli.md)
- [Runtime documentation](../docs/06-runtime.md)
- [Notebook format specification](../docs/03-notebook-format.md)

## Desktop Application

For the full interactive notebook experience with multi-language support, LSP integration, and rich UI features, consider using the **Project Mars desktop application**:

```bash
cd packages/desktop
npm install
npm run tauri dev
```

The desktop app provides the canonical notebook experience with features not available in editor extensions.
