# Project Mars VS Code extension

Scope: `extensions/vscode/**`. Follow the repository-level `AGENTS.md` first;
these rules cover the VS Code notebook integration.

## Architecture and extension contracts

- This package is the `.mars` notebook integration for VS Code's Notebook API.
  Its extension manifest in `package.json` is public API: keep command IDs,
  activation events, notebook type (`mars-notebook`), configuration keys, and
  contribution points synchronized with `src/` implementations.
- `extension.ts` is only activation wiring. Put command workflows in
  `commands.ts`, CLI process concerns in `cli.ts`, notebook execution and kernel
  lifecycle in `notebookController.ts`/`kernelService.ts`, and file conversion
  in `notebookSerializer.ts`.
- Keep output conversion behind `output.ts` and the focused `output/` modules.
  Preserve all recognized MIME bundles and a safe `text/plain` fallback; do not
  render executable HTML or SVG in untrusted workspaces.
- Serialization must round-trip Project Mars cell IDs, cell types, sources, metadata,
  execution counts, outputs, notebook metadata, dependencies, and format
  version. Unknown metadata is intentionally preserved.
- The extension delegates execution to the Project Mars CLI (`mars kernel serve`);
  never execute Rust code inside the extension host. Respect the configured CLI
  path, arguments, runtime, timeout, cancellation, and restart behaviour.
- Keep production source files at or below 200 lines where practical (220 is
  the absolute sensible exception). Prefer small modules with explicit exports
  over a monolithic activation or serializer file.

## Compatibility and safety

- Build against the API version in `engines.vscode` and use `@types/vscode`, not
  private or proposed APIs, unless the manifest is explicitly updated.
- Do not introduce `any`. Narrow `unknown` values from notebook metadata, JSON,
  configuration, and process output before use.
- Maintain cancellation-token support for VS Code callbacks and dispose every
  registration, controller, process, or listener through the extension context.
- Treat workspace files and CLI output as untrusted. Quote process arguments by
  passing argument arrays, validate paths, and return actionable errors rather
  than silently swallowing failures.
- Keep language grammar, language configuration, icon theme, assets, and
  extension manifest contributions aligned when introducing syntax or file-type
  changes.

## Validation

Run scoped checks from the repository root:

```bash
pnpm --filter mars-vscode check
pnpm --filter mars-vscode lint
pnpm --filter mars-vscode test
pnpm --filter mars-vscode build
```

Run `pnpm --filter mars-vscode package` when modifying manifest contributions,
packaging assets, or release behaviour. Do not commit generated `out/`, `vsix/`,
coverage, or packaged `.vsix` files.
