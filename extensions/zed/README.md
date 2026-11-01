# Mars Notebooks for Zed

The Mars Zed extension provides `.mars` language recognition, Tree-sitter
highlighting, and an MCP context server for the Mars CLI. It is a small Rust
WebAssembly extension; it never evaluates notebook code in Zed's extension
process.

## Features

- Associates `.mars` files with the Mars language.
- Provides Tree-sitter parsing and highlighting for the Mars notebook format,
  including embedded Rust and Markdown conventions.
- Registers `mars` as a Zed **context server** (`context_server_command`),
  launching `mars mcp serve` so Zed's Agent Panel gets notebook, cell, and
  kernel tools. Requires the `mars` CLI on `PATH`.

## Limitations

Zed has no notebook editor UI, and `mars kernel serve` speaks the Mars kernel
protocol rather than LSP, so this extension does not provide cell execution,
completions, or diagnostics. To run a notebook from Zed, use the Mars CLI or a
Zed task, for example a `.zed/tasks.json` entry:

```json
{
  "tasks": {
    "run notebook": {
      "command": "mars",
      "args": ["run", "${ZED_FILE}"],
      "use_new_terminal": true
    }
  }
}
```

For full notebook editing and execution, use the Mars desktop app or the VS
Code extension.

## Install locally

1. Build or install the Mars CLI:

   ```bash
   cargo install --path crates/mars-cli
   ```

2. In Zed, open **Settings → Extensions**, choose **Load Local Extension**, and
   select `extensions/zed` from this repository.

3. Open a `.mars` notebook to get Mars highlighting.

## Development

The extension is a `cdylib` compiled for `wasm32-wasip1`, using the pinned
`zed_extension_api` version required by Zed's extension ABI.

From the repository root, run:

```bash
cargo fmt --all --check
cargo clippy -p mars-zed --all-targets -- -D warnings
cargo test -p mars-zed
cargo build --target wasm32-wasip1 -p mars-zed
```

## Project layout

```text
extensions/zed/
├── extension.toml                    Extension manifest and grammar registration
├── src/lib.rs                        WebAssembly extension implementation
├── languages/mars/config.toml        `.mars` language association and editor defaults
└── grammars/tree-sitter-mars/        Grammar source and highlight queries
```

When changing the grammar, update `grammar.js`, `highlights.scm`, and the
language configuration together, then inspect a representative `.mars` file in
Zed. Do not commit `target/`, generated WebAssembly files, or JavaScript
dependency directories. See [AGENTS.md](AGENTS.md) for compatibility and
contributor guidance.
