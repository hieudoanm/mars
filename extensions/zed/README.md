# Mars Notebooks for Zed

The Mars Zed extension provides `.mars` language recognition, Tree-sitter
highlighting, and a Mars CLI-backed language-server command for Zed. It is a
small Rust WebAssembly extension; it never evaluates notebook code in Zed's
extension process.

## Features

- Associates `.mars` files with the Mars language.
- Provides Tree-sitter parsing and highlighting for the Mars JSON notebook
  format, with Mars notebook conventions for Markdown and Rust content.
- Starts `mars kernel serve` after confirming that the Mars CLI is available in
  the active worktree.
- Gives an actionable error when the CLI is not installed or is absent from
  `PATH`.

## Install locally

1. Build or install the Mars CLI:

   ```bash
   cargo install --path crates/mars-cli
   ```

2. Ensure the CLI's installation directory is on `PATH`.

3. In Zed, open **Settings → Extensions**, choose **Load Local Extension**, and
   select `extensions/zed` from this repository.

4. Open a `.mars` notebook. Zed recognizes it as Mars and starts the CLI-backed
   language-server command when needed.

The extension currently launches the default CLI command `mars kernel serve`.
Its language-server integration is intentionally thin while the Mars kernel
protocol evolves.

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
├── extension.toml                    Extension manifest and language-server registration
├── src/lib.rs                        WebAssembly extension implementation
├── languages/mars/config.toml        `.mars` language association and editor defaults
└── grammars/tree-sitter-mars/        Grammar source and highlight queries
```

When changing the grammar, update `grammar.js`, `highlights.scm`, and the
language configuration together, then inspect a representative `.mars` file in
Zed. Do not commit `target/`, generated WebAssembly files, or JavaScript
dependency directories. See [AGENTS.md](AGENTS.md) for compatibility and
contributor guidance.
