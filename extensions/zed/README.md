# Mars Notebooks for Zed

This extension provides language support for `.mars` notebook files in the Zed editor, including syntax highlighting and integration with the Mars CLI for notebook execution.

## Features

- **Language support**: Associate `.mars` files with the Mars language for syntax highlighting
- **Tree-sitter grammar**: Syntax highlighting for Mars notebook JSON structure
- **CLI integration**: Launches `mars kernel serve` as the language server for notebook execution
- **Configuration**: Configurable Mars CLI path and execution settings

## Installation

1. Install the Mars CLI:

   ```bash
   cargo install --path crates/mars-cli
   ```

2. Add `~/.local/bin` to your PATH (or wherever the Mars CLI is installed):

   ```bash
   export PATH="$HOME/.local/bin:$PATH"
   ```

3. Load the extension in Zed:

   - Open Zed
   - Go to Settings → Extensions
   - Click "Load Local Extension"
   - Select `extensions/zed`

## Development

The extension is written in Rust and compiled to WebAssembly using the [`zed_extension_api`](https://crates.io/crates/zed_extension_api) crate.

```bash
# From the repository root
cargo build --target wasm32-wasip1 -p mars-zed
cargo test -p mars-zed
cargo clippy -p mars-zed
```

## Project Structure

```
extensions/zed/
├── extension.toml          # Extension manifest
├── Cargo.toml              # Rust project manifest
├── src/
│   └── lib.rs              # Extension implementation
├── languages/
│   └── mars/
│       └── config.toml     # Mars language configuration
├── grammars/
│   └── tree-sitter-mars/
│       ├── grammar.js      # Tree-sitter grammar source
│       ├── highlights.scm  # Syntax highlighting queries
│       └── package.json    # Grammar metadata
├── tests/
├── README.md
└── .gitignore
```

## Usage

1. Open any `.mars` file in Zed
2. The file will be recognized as a Mars notebook
3. Syntax highlighting is provided by the Tree-sitter grammar
4. The Mars language server (`mars kernel serve`) handles notebook execution

## Configuration

Configure the Mars CLI path in Zed's `settings.json`:

```json
{
  "mars": {
    "cliPath": "mars",
    "executionTimeout": 30000,
    "runtime": "replay"
  }
}
```
