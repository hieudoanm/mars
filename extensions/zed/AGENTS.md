# Mars Zed extension

Scope: `extensions/zed/**`. Follow the repository-level `AGENTS.md` first;
these instructions apply to the Zed WebAssembly extension and its grammar.

## Architecture and compatibility

- This is a Zed extension compiled as a `cdylib` for `wasm32-wasip1`. Keep its
  runtime surface minimal: `src/lib.rs` implements `zed_extension_api::Extension`
  and launches the external Mars CLI rather than evaluating notebooks itself.
- `extension.toml` is part of the published integration contract. Keep its
  extension ID, language path, grammar registration, and language-server name
  aligned with `languages/mars/config.toml` and the Rust implementation.
- The pinned `zed_extension_api` version exists for Zed ABI compatibility. Do
  not update it casually or use APIs unavailable at that version.
- The language server command must use an argument vector, validate that the CLI
  can be found in the current worktree, and return an actionable error when it
  cannot. Never assume a shell, PATH mutation, or host-specific installation
  path.
- Keep production Rust files at or below 200 lines where practical (220 is the
  absolute sensible exception). Extract command construction or configuration
  parsing into focused modules if the extension grows.

## Grammar and language support

- The Tree-sitter grammar in `grammars/tree-sitter-mars/grammar.js` describes
  the Mars JSON notebook format. Make grammar changes together with
  `highlights.scm`, language configuration, fixture coverage where added, and
  the public extension description when behaviour changes.
- Preserve support for Markdown and embedded Rust semantics. Avoid claiming full
  notebook execution or language-server functionality unless the CLI protocol
  actually provides it.
- Treat configuration and worktree data as untrusted. Use typed parsing and
  descriptive errors; avoid `unwrap()` and `expect()` in production code.

## Validation

Run Rust checks from the repository root:

```bash
cargo fmt --all --check
cargo clippy -p mars-zed --all-targets -- -D warnings
cargo test -p mars-zed
cargo build --target wasm32-wasip1 -p mars-zed
```

If grammar files change, also run the grammar package's available test/generate
command from `grammars/tree-sitter-mars/` and review highlighting in Zed. Do not
commit `target/`, generated WebAssembly output, or npm dependency directories.
