# Architecture

Mars is organized as a modular Rust workspace with independent React
applications. The desktop product is a Tauri 2 shell around a React/Vite
frontend; the CLI and the desktop backend share the same Rust domain crates.

## Repository layers

### Rust workspace

- `mars-core` defines shared IDs, cell types, output variants, MIME bundles,
  metadata, and execution status.
- `mars-document` owns the `.mars` notebook model, validation, migrations,
  cell operations, serialization, and atomic persistence.
- `mars-runtime` executes Rust cells in isolated temporary child processes,
  preserves bindings through replay, captures output, and handles cancellation.
- `mars-kernel` owns a runtime session and translates typed requests into
  execution, reset, and status operations.
- `mars-protocol` defines transport-neutral request, response, and event
  messages.
- `mars-renderer` produces sanitized standalone HTML with offline fallbacks for
  supported math and chart output.
- `mars-jupyter` converts format-4 Jupyter notebooks and preserves unsupported
  metadata/MIME values where possible.
- `mars-cli` exposes notebook creation, inspection, execution, conversion, and
  kernel lifecycle operations.

### Node applications

- `@mars/desktop` provides the canonical interactive notebook UI. Tauri
  commands call the Rust kernel and document services; React does not execute
  user code or persist notebooks directly.
- `@mars/vscode` is an optional editor adapter for `.mars` syntax, cell
  navigation, preview/export, and CLI-backed execution.
- `@mars/landing` is the React/Vite public marketing site and does not share
  runtime behavior with the notebook editor.

## Data flow

```text
.mars file
   |
   v
mars-document  <---->  desktop persistence hooks / mars-cli
   |
   v
mars-core domain types
   |
   v
mars-protocol request/response types
   |
   v
mars-kernel
   |
   v
mars-runtime ---> isolated rustc/Cargo child process ---> captured outputs
   |
   +--> mars-renderer for standalone HTML
   +--> desktop OutputView for interactive rich rendering
```

The CLI bypasses the desktop transport but uses the same document, runtime,
Jupyter, and renderer crates. The desktop uses Tauri as an adapter rather than
placing business logic in the Tauri command handlers.
- `apps/desktop` provides the Tauri/React editor. Its UI orchestration is
  separated from domain operations: `useNotebookEditor.ts` owns notebook state,
  selection, history, and mutations; `useNotebookPersistence.ts` owns file
  state, autosave, and close/open flows; `useKernelExecution.ts` owns runtime
  execution; and `useNotebookExecutionCommands.ts` owns derived run ranges.
  `useEditorPreferences.ts` owns persisted editor settings,
  `notebookOperations.ts` contains pure cell mutations, `kernelService.ts`
  owns Tauri kernel/file calls, `useKeyboardShortcuts.ts` owns command-mode
  shortcuts, `diagnostics.ts` translates compiler locations for CodeMirror,
  and `OutputView.tsx` owns rich output rendering.
- The optional `apps/vscode` extension is a thin editor adapter. It provides
  syntax recognition, cell navigation, and CLI-backed preview/export while
  keeping execution and persistence in Mars itself.

`App.tsx` now composes these hooks and renders the desktop surface; it does not
own notebook history, persistence, execution orchestration, editor preference
storage, or derived execution-range logic. The desktop app calls Tauri
commands backed by `mars-kernel`; it does not reimplement persistence or
execution. The CLI uses the same document and runtime crates.

## Boundary rules

1. Notebook persistence belongs in `mars-document`.
2. Runtime execution belongs in `mars-runtime` and must remain outside the
   desktop process.
3. Request/response translation belongs in `mars-kernel` and
   `mars-protocol`.
4. Rendering must escape or sanitize untrusted notebook content.
5. React components may compose state and present data, but must not become the
   source of truth for document persistence or runtime semantics.
6. New language runtimes should implement a runtime boundary without changing
   the `.mars` document schema.
