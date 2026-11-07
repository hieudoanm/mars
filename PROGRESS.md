# Mars progress

## Current phase

Phase 7 — Desktop and rendering polish

## Completed

- Initialized a modular Cargo workspace.
- Added `mars-core` shared IDs, cell types, outputs, and execution status.
- Added `mars-document` notebook model and `.mars` JSON serialization.
- Added document validation, cell operations, and atomic saving.
- Added initial `mars new` and `mars inspect` CLI commands.
- Added unit tests for serialization, cell operations, and validation.
- Added an isolated replay-based Rust execution session.
- Added persistent bindings across successful code cells.
- Added compilation/runtime output capture and `mars run`.
- Added `mars-jupyter` for basic `.ipynb` import and export.
- Preserved Jupyter notebook metadata and supported output types during conversion.
- Added `mars import` and `mars export <file.mars> ipynb <output.ipynb>`.
- Added `mars-renderer` with Markdown, code, text, errors, tables, images, and sanitized HTML output rendering.
- Added functional `mars export <file.mars> html <output.html>` with standalone
  styling.
- Added a shared `mars export <file.mars> md <output.md>` Markdown exporter used
  by the CLI, desktop export command, and VS Code **Export Notebook to
  Markdown** command.
- Converted `apps/landing` to a Next.js static export and added Markdown
  documentation under `apps/landing/docs`, served at `/docs` with a generated
  sidebar and per-document static pages.
- Added `mars-protocol` typed kernel requests, responses, and events.
- Added `mars-kernel` service boundary over the isolated Rust runtime.
- Added a Tauri 2 desktop shell with a React/TypeScript notebook editor.
- Connected frontend cell execution and runtime reset to typed Tauri commands.
- Added notebook keyboard shortcuts and cell operations: insert, duplicate, delete, move, and type conversion.
- Documented the VS Code companion scope and CLI/protocol reuse strategy.
- Replaced textarea cells with CodeMirror 6 editors.
- Added Rust and Markdown syntax modes, dark theme, line wrapping, and editor focus integration.
- Added desktop execute-all, execute-above, execute-below, and clear-output workflows.
- Added serialized dirty-state tracking and debounced autosave for opened notebooks.
- Added explicit Save and open without saving choices before replacing dirty notebooks.
- Sanitized desktop HTML outputs with a strict DOMPurify allowlist before rendering.
- Added the same unsaved-change protection to desktop window close requests.
- Added per-cell CLI execution status, timing, output summaries, and failure reporting.
- Added runtime timeouts and interruption reporting for long-running CLI cells.
- Applied timeout limits across both Rust compilation and compiled-cell execution.
- Added optional execution timeouts to the typed kernel protocol and Tauri command layer.
- Added an optional desktop execution-timeout control with unlimited as the default.
- Added external execution cancellation through the desktop Stop control.
- Added conservative automatic output extraction for final Rust expressions.
- Added contributor, architecture, notebook-format, runtime, and CLI documentation.
- Added notebook-declared Cargo dependencies through temporary Cargo projects.
- Added LaTeX, chart-spec, and arbitrary MIME-bundle output variants across the
  core model, Jupyter conversion, standalone renderer, CLI, and desktop UI.
- Added MIME-bundle image rendering for base64 PNG/JPEG/GIF/WebP and vector SVG
  across the standalone HTML renderer, desktop output view, and VS Code
  notebook renderer.
- Promoted Evcxr/plotters SVG HTML output to `image/svg+xml` MIME bundles and
  allowed sanitized SVG in desktop HTML output.
- Added a Playwright browser smoke-test harness for the desktop editor.
- Added KaTeX equation rendering and Vega/Vega-Lite chart rendering to the
  desktop output view.
- Added CodeMirror folding, bracket matching, autocomplete, diagnostics,
  configurable font size, and persisted editor preferences.
- Added format-version migration from `.mars` version 1 to version 2 plus an
  undo/redo history foundation.
- Added richer Cargo dependency table specifications and desktop kernel
  reconfiguration when opening notebooks.
- Added a persistent newline-delimited JSON kernel transport through
  `mars kernel serve`.
- Kept standalone HTML rich output self-contained with escaped LaTeX/chart
  fallbacks while the desktop uses bundled interactive renderers.
- Added source-position-aware editor diagnostics when compiler messages include
  line and column locations.
- Added a representative Jupyter rich-output fixture and round-trip coverage.
- Added installation, platform prerequisites, release packaging, signing,
  artifact verification, and CI guidance in `docs/09-release.md` and
  `docs/08-development.md`.
- Added a screenshot capture checklist under `docs/screenshots/` and replaced
  the README screenshot placeholder with a documented release workflow.
- Added `mars open`, kernel lifecycle commands, development documentation,
  representative examples, and a rich Jupyter fixture.
- Extracted desktop notebook mutations, kernel/file services, keyboard
  shortcuts, diagnostics, and output rendering from `App.tsx`.
- Added offline SVG chart rendering and self-contained static math output for
  CLI HTML exports.
- Added structured rustc JSON diagnostic formatting with primary multi-span
  locations and severity labels.
- Added deterministic CLI execution controls: `--deterministic`, `--no-save`,
  and `--report`.
- Added the optional VS Code companion under `apps/vscode` with `.mars`
  language support, cell navigation, and CLI-backed HTML export.
- Added and linked a privacy-reviewed desktop overview screenshot.
- Installed the Tauri CLI, built macOS `.app` and `.dmg` artifacts, and
  recorded the ad-hoc signing/checksum limitation in the release guide.
- Added notebook-embedded Cargo.lock persistence, dependency-name validation,
  and reproducible `--locked`/`--offline` execution controls.
- Added desktop undo/redo shortcuts and Playwright coverage for history and
  persisted editor preferences.
- Added documentation for lockfile behavior, offline execution, and dependency
  reproducibility.
- Extracted the remaining desktop orchestration from `App.tsx` into focused
  hooks for notebook state/history/mutations, editor preferences, and derived
  execution commands; `App.tsx` is now composition and presentation.
- Added a root `Makefile` with formatting, lint, test, desktop, E2E, and runtime
  benchmark targets.
- Added a reproducible replay benchmark and documented the measured linear
  scaling and decision to retain replay isolation over an unvalidated
  long-lived compiler process.
- Hardened runtime cancellation with Unix process groups and Windows process
  tree termination, preventing compiler/runtime descendants from outliving a
  cancelled cell.
- Added a representative Jupyter compatibility fixture covering display
  metadata, MIME preservation, PNG/JPEG/SVG images, widgets, errors, tables,
  and unknown MIME values.
- Synchronized migrated SVG/PNG branding assets across the repository, desktop
  metadata/Tauri icon configuration, and the VS Code package; the file mark is
  now a simplified planet without the `M`.
- Added a React/Vite landing page under `apps/landing` with responsive product,
  architecture, workflow, tooling, and getting-started sections plus local
  SVG/PNG branding assets.
- Added `.pre-commit-config.yaml` with repository hygiene checks, Rust
  formatting/lint/tests, desktop build, VS Code packaging, and landing-page
  asset validation hooks.
- Added a pnpm workspace for the desktop, VS Code, and React landing
  applications with root build, E2E, compile, package, and landing-page
  development scripts.
- Added Turborepo task orchestration for workspace builds, TypeScript checks,
  VS Code packaging, desktop E2E tests, and parallel development servers.
- Expanded README, contributor, architecture, runtime, notebook-format, CLI,
  protocol, development, release, branding, landing, and VS Code
  documentation with the current workspace commands, package names, runtime
  guarantees, format invariants, compatibility limits, and release workflow.
- Added a reusable desktop UI-state hook so `App.tsx` no longer owns status,
  shortcut command, or execution-timeout state.
- Expanded compiler diagnostic formatting with rustc error codes, secondary
  spans, rendered notes, and Unicode-aware CodeMirror offset conversion.
- Added offline static rendering for common LaTeX fractions and documented the
  renderer's supported scope and safe limitations.
- Added a VS Code notebook-run command, configurable CLI run arguments, output
  channel reporting, and packaged branding assets.
- Validated the branded native macOS `.app` and `.dmg` bundles after updating
  Tauri configuration and desktop metadata.
- Added the isolated `mars-runtime-evcxr` backend behind explicit CLI, kernel,
  and desktop opt-in, preserving replay as the default and leaving the notebook
  format unchanged.
- Added a standalone Evcxr behavior executable and paired CSV benchmark runner,
  plus `docs/10-evcxr-experiment.md` with setup, compatibility notes, and the
  experiment recommendation.
- Added timeout/cancellation handling that kills Evcxr's evaluator process,
  abandons a stuck controller call, and recreates the session; reset, process
  failure, and compilation errors map through the existing runtime outputs.
- Added target-aware installation (`--cli`, `--vscode`/`--vscode-only`,
  `--desktop`, and `--no-*` flags) so the VS Code extension can be rebuilt and
  installed without the desktop app.
- Fixed replay-runtime display capture: `EVCXR_BEGIN_CONTENT` markers are now
  parsed, standalone SVG HTML is promoted to an `image/svg+xml` MIME bundle, and
  `examples/mars/05-plotting.mars` stores the figure instead of marker text.
- Added `mars kernel serve --notebook <file.mars>` so the VS Code extension
  loads notebook Cargo dependencies and restarts the kernel when the active
  notebook changes.
- Renamed the Tauri host package from `mars-desktop` to `mars-tauri` and updated
  all references (`main.rs` lib call, docs, test temp-dir prefix).
- Started `crates/mars-desktop`, a pure-Rust GPUI desktop client as an
  alternative to the Tauri/React app: window shell, dark Mars theme, toolbar
  (New/Open/Save/Run all/Restart kernel), scrollable cell list, per-cell Run
  buttons, plain-text output rendering, kernel status bar, native file
  dialogs, and `cmd-n/o/s`, `cmd-shift-enter`, `cmd-r`, `cmd-q` shortcuts.
  Kernel execution runs on the GPUI background executor via `mars-kernel`.
  Verified with `cargo check`, `cargo clippy -D warnings`, `cargo fmt --check`,
  and a GUI smoke run. GPUI requires the macOS Metal Toolchain
  (`xcodebuild -downloadComponent MetalToolchain`).
- Migrated `crates/mars-desktop` from crates.io `gpui 0.2.2` to the
  `gpui-pre = "=0.3.9"` family (aliased as `gpui`, plus `gpui-pre-platform` as
  `gpui_platform`) so `gpui-kit-phosphor 0.1.0` icons can be used; icons are
  registered via `PhosphorAssets` in `main.rs` and rendered throughout the
  toolbar and cell controls. Added cell operations: insert Code/Markdown
  cells, delete, move up/down, and Code↔Markdown/Raw type toggle (outputs are
  cleared when converting to a non-code type). Removed remaining `expect`s:
  kernel creation failure and window-open failure now surface as status
  messages/process exit instead of panicking.
- Added real cell editing via `gpui-component 0.7.3` (the newest release
  compatible with `gpui-pre 0.3.9`; 0.8+/0.9 require newer gpui-pre and
  conflict with the phosphor pin). Each cell gets an auto-growing
  `TextareaState` synced to `Notebook.cells[].source` through
  `InputEvent::Change` subscriptions; edits to code cells clear stale
  outputs/execution counts. gpui-component is initialized with a dark theme.

- Added `.ipynb` import/export to the Tauri desktop: `import_notebook_ipynb`
  and `export_notebook_ipynb` commands backed by `mars-jupyter`, header
  Import/Export `.ipynb` actions, and extension-based dispatch in
  `loadNotebook` so the unsaved-changes flow covers Jupyter files. Imported
  notebooks open unsaved so they can be persisted as `.mars`.
- Added live execution streaming: `mars-runtime` emits `StreamEvent`
  phase (`compiling`/`running`) and stdout/stderr chunk events through a new
  `RuntimeBackend::execute_with_events` (default ignores events, so the
  experimental Evcxr backend is unchanged), `mars-kernel::Kernel::
  execute_with_events` forwards them, and the Tauri host emits them as
  `kernel-stream` events. The desktop accumulates chunks per cell and shows
  live output plus phase status while a cell runs.
- Added drag-and-drop cell reordering via a per-cell drag handle, drop-target
  highlighting, and `moveCellToIndex`/`moveCellTo` operations; added an
  "Add Raw" toolbar action and `r` command-mode conversion to raw cells.
- Code-split the desktop bundle: `CodeEditor` (CodeMirror), `OutputView`
  (KaTeX/Vega), and `MarkdownPreview` are now lazy chunks; the KaTeX
  stylesheet moved into the `OutputView` chunk. Main bundle is ~73 kB.
- Added `parseNotebookJson` validation for opened/imported documents
  (replaces unchecked `JSON.parse ... as Notebook`) with unit coverage.
- Markdown and raw cells can no longer execute or render outputs: run-all /
  run-above / run-below filter to code cells, and `NotebookCellView` only
  renders outputs for code cells.
- Added external file-change protection: a `notebook_file_mtime` Tauri
  command tracks the on-disk mtime at open/save time, and autosave pauses
  with a status warning when the file changed externally instead of
  silently overwriting it.
- Added per-cell notebook search: `@codemirror/search` panel with `Mod-f`
  keymap inside editors plus a Search button on each cell that focuses the
  editor and opens the panel.
- Added a Markdown preview visibility toggle (eye icon) on Markdown cells.
- Removed the experimental GPUI desktop client (`crates/mars-desktop`) and
  the vendored `block` patch (`vendor/`): the Tauri app is the single desktop
  client. The `gpui` workspace dependency and `[patch.crates-io]` entry were
  dropped from the root `Cargo.toml`.
- Fixed Markdown preview styling: the `prose` classes were dead because
  `@tailwindcss/typography` was never installed — added the plugin
  (0.5.20) and registered it in `styles.css`. Raw HTML inside Markdown
  cells now renders via `rehype-raw` followed by `rehype-sanitize` so
  embedded markup is displayed but sanitized.
- Run-all/run-above/run-below now halt at the first cell whose execution
  produces an error output: the status bar shows `Failed in N ms — <name>:
  <first message line>`, the failing cell is selected and scrolled into
  view via the `onExecutionHalted` callback, and its error output plus
  editor diagnostics remain visible. Previously only interruptions and
  thrown errors stopped the batch; compile/runtime error outputs did not.

### Jupyter catch-up batch (desktop)

- Multi-notebook workspace: `App.tsx` is now a tabbed shell hosting one
  `NotebookSession` per open notebook; kernels are per-session in the Tauri
  host (`session` arg on all execution commands, stream events carry the
  session id). Tabs track dirty state, dedupe by path, prompt before closing
  dirty tabs, and restore the last-opened path.
- File explorer CRUD: new notebook/folder, inline rename, recursive delete,
  hidden/dot entries and `*.checkpoints` dirs filtered.
- Output ergonomics: `CellOutput` wrapper adds per-output collapse, copy, and
  clear; cell header gained type dropdown, move up/down, merge-with-next.
- Global notebook search dialog (sidebar + palette), Rust-aware completion
  source (keywords, primitives, `let`/`fn`/type symbols collected from all
  code cells), and a variables/symbols inspector in the notebook sidebar.
- Dependencies dialog edits `metadata.dependencies`; checkpoints are stored in
  `<notebook>.mars.checkpoints/` via new Tauri commands with restore support.
- Command palette (Cmd+K) aggregates workspace + per-session commands;
  settings dialog (Cmd+, or header gear) offers mars-dark/mars-light
  DaisyUI themes (`data-theme`) and documents shortcuts.
- Session restore persists scroll position and selected cell per file path.
- Print stylesheet hides chrome and restyles cells for printing/PDF.
- Header gained HTML/PDF export; runtime restart variants (clear outputs,
  restart & run all) added.

Follow-up pass on the limitations:

- Cell ops completed: split-at-cursor (scissors button reads the CodeMirror
  cursor offset), comma-separated tag editing with tag badges, and
  shift-click range multi-select — range selections highlight, delete as a
  group, and can be run via the "Run selected cells" palette command.
- Shortcuts are now rebindable: `lib/shortcuts.ts` stores per-action combos
  in localStorage; `useKeyboardShortcuts` matches events through
  `comboMatches`; the settings dialog records new combos on click and offers
  per-action reset plus "Reset all".
- Styling migrated to DaisyUI 5 with custom `mars-dark` (default) and
  `mars-light` themes defined in `src/styles.css`; shared primitives
  (Button, IconButton, TextInput, Checkbox) use `btn`/`input`/`checkbox`
  classes so theme colors resolve from CSS variables in both themes.
- Light theme extended to the CodeMirror editor (light chrome, caret,
  placeholder, panels, search-match colors; `oneDark` skipped in light mode)
  via a `colorScheme` prop threaded App → session → cell → editor.
- Rust completion gained macro entries (`println!`, `vec!`, `assert_eq!`, …)
  that insert `name!()`.

Second gap-analysis batch (desktop + runtime):

- Live variable inspector: new `mars-runtime::inspect` module exposes
  `inspect_variables` (per-session bindings with type/value previews via a
  replayed probe program) and `diagnose_cell` (rustc JSON diagnostics mapped
  back to notebook cells through accumulated-source offsets). Wired through
  `mars-kernel` traits, Tauri commands `inspect_variables`/`diagnose_cell`,
  `kernelService.ts`, and the notebook sidebar's live variables section with
  manual refresh.
- Interactive stdin: runtime emits `StreamEvent::Prompt` via a
  `__MARS_PROMPT__` marker scanned across chunk boundaries (`PromptScan` in
  `execution.rs`, unit-tested); `mars_prompt()` is injected into replayed
  programs; stdin plumbing runs through `session_execution.rs`, the kernel
  trait, Tauri `send_kernel_input`, `useKernelExecution.submitInput`, and a
  `PromptDialog` in the notebook UI.
- Pre-execution diagnostics: selecting a code cell debounces a `diagnose_cell`
  call and maps compiler diagnostics onto the cell via `liveDiagnostics`.
- Cell magic directives: `cellDirectives.ts` parses `// mars: dep …` and
  `// mars: timeout …` comments; dependencies feed `editor.setDependencies`
  and timeout overrides the execution timeout for that run.
- Recents + templates + drag-drop: `lib/recents.ts` persists recent paths;
  `TemplatePickerDialog` offers starter notebooks; `App.tsx` accepts
  drag-dropped `.mars`/`.ipynb` files onto the workspace.
- Search & replace: `NotebookSearchDialog` gained replace mode that rewrites
  matching cell sources through `notebookOperations`.
- Virtualization: cell wrappers already use `content-visibility: auto`, which
  covers the large-notebook rendering cost without a virtualizer.
- Document collaboration and interactive widget/debugging protocols were
  reviewed and documented as out of scope for now.

Third gap-analysis batch (desktop + runtime):

- Execution queue: `useKernelExecution` now queues cell executions FIFO while
  a run is in flight instead of rejecting them; the status bar shows the
  queued count.
- Search & replace upgraded: case-sensitive / whole-word / regex toggles in
  `NotebookSearchDialog` (shared `lib/textSearch.ts` matcher), and replace-all
  is a single notebook mutation (`replaceInNotebookCells`) so it is one undo
  step.
- Session restore now persists per-cell collapse state alongside scroll
  position and selected cell (`useSessionRestore` hook).
- Kernel status bar shows the short session id and queued executions.
- Checkpoints gained a diff view: `read_notebook_checkpoint` Tauri command
  (path-validated), LCS line diff in `lib/textDiff.ts`, and an inline diff
  pane in `CheckpointsDialog`.
- `NotebookSession` was split: `useSessionRestore`, `useNotebookInspection`,
  and `NotebookSessionOverlays` extracted (585 → ~450 lines of wiring/JSX).
- rust-analyzer LSP: new `crates/mars-lsp` crate — a minimal stdio JSON-RPC
  client plus `LspSession` that maintains a scratch Cargo project per
  notebook session under the temp dir, wraps code cells in `fn main()`,
  syncs on edit (debounced), maps positions/diagnostics back to cells, and
  surfaces hover tooltips in CodeMirror plus live `publishDiagnostics` in
  the editor. Tauri commands: `lsp_sync`, `lsp_hover`, `lsp_completion`,
  `lsp_diagnostics`. Semantic completions via LSP are wired backend-side but
  the editor still uses the heuristic source (async LSP completion merge is
  a follow-up).
- Editor virtualization: `LazyMount` mounts CodeMirror only when a cell
  approaches the viewport (`content-visibility` already covered paint cost).
- "Open containing folder in terminal" header action (`open_in_terminal`
  Tauri command; macOS Terminal / Windows start / common Linux emulators).
- Debugger/breakpoints, a full extension ecosystem, real-time collaboration,
  and a widget protocol remain deliberately out of scope: they are
  multi-week protocol/runtime projects rather than gap-fillers, and the
  replay execution model has no debug hooks to attach to.

Remaining limitation: rust-analyzer LSP covers hover and live diagnostics;
completion still uses the heuristic source and go-to-definition/signature
help are not yet surfaced in the UI.

### Jupyter catch-up batch 4 (desktop)

- Editor line numbers enabled in the CodeMirror gutter.
- LSP completion merge: `lib/lspCompletions.ts` adapts rust-analyzer
  completion items into CodeMirror sources merged with the heuristic
  fallback; signature help is surfaced via a tooltip on `(` while typing;
  go-to-definition (Ctrl/Cmd+click) jumps to the target cell and position
  via `LspDefinitionTarget` plumbing through `NotebookSession`.
- Workspace persistence: open tab list and active tab are stored in
  localStorage (`lib/workspace.ts`) with defensive parsing, and sessions
  are restored on launch.
- `WorkspaceHeader` gained a visible recents dropdown (recent notebook
  paths), HTML5 drag-to-reorder tabs, and middle-click tab close.
- Sidebar problems panel: aggregated pre-execution + LSP diagnostics across
  cells with severity dots and click-to-select navigation.
- Variable inspector polish: per-variable copy-to-clipboard and full-value
  tooltips on truncated previews.
- Search dialog replace-all now shows an inline "Replace N occurrences?"
  confirmation before applying the single-undo mutation.
- Per-cell language metadata: code cells carry `metadata.language`
  (`rust` default, `text` for non-executed cells); the cell header exposes
  a language selector, non-Rust cells skip highlighting/LSP/execution, and
  `lib/cellLanguage.ts` centralizes the predicate.
- Output download action on every cell output (images and image MIME
  bundles download as real image files; other outputs download as text
  with type-appropriate extensions).
- True editor unmount virtualization remains a documented trade-off:
  `LazyMount` preserves CodeMirror undo history; windowed unmounting would
  discard it.

### Jupyter catch-up batch 5 (desktop)

- `NotebookSession.tsx` re-split: LSP sync/hover/signature/definition wiring
  extracted to `hooks/useLspIntegration.ts`, aggregated problems moved into
  `useNotebookInspection`, collapse/dialog state consolidated in
  `useNotebookUiState`, and cell selection/drag/range logic extracted to
  `hooks/useCellInteractions.ts` (558 → ~440 lines).
- Interrupt now drains the FIFO execution queue: `cancelExecution` resolves
  all queued cells as failed instead of letting them run after cancel.
- Per-cell execution badges: each code cell shows a persistent
  `✓/✗/‖ Nms` badge in its header (success/error/interrupted + duration),
  cleared on runtime restart.
- Sidebar "Contents" section: markdown `#`–`######` headings are extracted
  per cell and rendered as an indented, click-to-navigate table of contents.
- LSP unit coverage: `lib/lsp.ts` hover/signature-help formatting and the
  `lspCompletions` CodeMirror adapter are tested; live rust-analyzer
  signature/definition remains a manual smoke item.
- Editor polish: `rectangularSelection()` + `allowMultipleSelections` from
  `@codemirror/view` (Alt-drag block selection) and
  `@replit/codemirror-indentation-markers` indent guides for Rust cells.
- Workspace tab persistence now covers untitled notebooks: tab entries store
  `{key, path, untitledId}` with legacy `paths` format still readable, and
  untitled notebook content is snapshotted to localStorage (debounced) and
  restored on launch.
- Checkpoint diffs gained a per-cell view: `lib/notebookCellDiff.ts` matches
  cells by id across checkpoint/current and the dialog shows per-cell
  `+added -removed` chips that swap the diff pane to that cell's source diff.
- Playwright E2E (`tests/app.spec.ts`) now covers tab persistence across
  reload and was updated for the select-based font-size control; all 4
  specs pass in Chromium. Native Tauri-shell E2E (tauri-driver) remains a
  release-machine task.
- Deliberately not implemented (documented scope decisions): zen/focus
  mode, split-pane editing, cell attachments, drag-between-notebooks, and
  i18n — niche conveniences with low payoff for a Rust-first notebook tool;
  debugger/collaboration/widget-protocol/extension-ecosystem remain out of
  scope as multi-week protocol projects.

### Jupyter catch-up batch 6 (desktop + runtime)

- Editor preferences are reachable again: Autocomplete and Folding
  checkboxes were restored to `SettingsDialog` (the stored preferences had
  become UI-unreachable after the toolbar rework).
- `NotebookCellList` extracted from `NotebookSession.tsx`: the ~55-line
  per-cell JSX block and its ~30 props now live in
  `components/notebook/NotebookCellList.tsx`.
- Variable inspector depth: sidebar variables expand to show full values
  and offer copy/copy-as-JSON actions.
- Editor polish: standard CodeMirror `highlightActiveLine`,
  `highlightActiveLineGutter`, `highlightSelectionMatches`, and
  `drawSelection` extensions (rectangular selection already present).
- LSP quick fixes: `textDocument/codeAction` is implemented in `mars-lsp`
  with generated-document→cell edit mapping, exposed via Tauri
  `lsp_code_actions`, surfaced through `useLspIntegration`/`cellTextEdit`,
  and presented as a lightbulb menu in the editor gutter.
- Runtime error → cell navigation: generated `// Mars cell` markers now
  embed a djb2 source hash; `output::annotate_source_locations` rewrites
  `*.rs:LINE:COL` panic locations in stderr with
  `[mars:cell=HASH,line=N]` annotations, which `OutputView` renders as
  clickable links that select the matching cell and reveal the line
  (hash-based mapping survives out-of-order cell execution).
- Image attachments: pasting or dropping an image onto a Markdown cell
  stores it as a base64 data URL under `metadata.attachments` and inserts
  `![name](attachment:name)`; `MarkdownPreview` resolves attachment
  references and the sanitize schema/urlTransform allow `data:image/`
  sources only. Code/raw cells do not accept image pastes.
- Scope decisions documented: Mars remains Rust-first — a second language
  runtime is a deliberate future integration behind a subprocess/protocol
  boundary, not a Jupyter-parity requirement. Native Tauri E2E
  (tauri-driver launch/save/reopen) remains a release-machine validation
  task; browser Playwright covers workspace logic. Zen mode, split view,
  drag-between-notebooks, i18n, debugger, collaboration, widgets, and
  extensions remain out of scope.

### Jupyter catch-up batch 7 (desktop + runtime + LSP)

- LSP verified against a real rust-analyzer: new opt-in
  `crates/mars-lsp/tests/live_smoke.rs` spawns rust-analyzer (skipped when
  the binary is absent, `MARS_RUST_ANALYZER` env override supported) and
  asserts cell-mapped diagnostics, hover, completion, definition, and code
  actions. The smoke test caught three real bugs, all fixed:
  - `--disable-build-scripts` is not a valid rust-analyzer flag and killed
    the server during initialize (LSP was effectively dead).
  - rust-analyzer returns `null` for `textDocument/codeAction` unless the
    client declares the `codeAction` capability at initialize.
  - `build_document` wrapped cell sources verbatim, so a trailing bare
    expression (`value`) broke parsing for every following cell; cells now
    get a `;` appended via `mars_runtime::expression::split_final_expression`
    (positions preserved).
  - `code_actions` now expands a bare cursor position to the covering
    diagnostic's range, which quick fixes require.
- Toast notifications: `hooks/useToasts.ts` + `ToastViewport` (bottom-right,
  tone-coded, auto-dismiss); file-operation failures (open/save/import/
  export/rename/autosave/terminal) now surface as error toasts and saves
  confirm with a success toast.
- Cell context menu: right-click a cell for run/insert/duplicate/copy
  source/move/merge/convert/clear/delete (`CellContextMenu.tsx`,
  `editor.duplicateCellById`).
- Markdown alerts: `> [!NOTE|TIP|IMPORTANT|WARNING|CAUTION]` blockquotes
  render as styled callouts via a custom `blockquote` component.
- Global search now lands on the exact match: `SearchMatch` carries the
  column, and selection routes through the editor reveal mechanism
  (`onRevealMatch` → `pendingReveal`) instead of only scrolling to the
  cell. In-cell find already used the CodeMirror search panel.
- Keyboard shortcut customization was already implemented
  (`lib/shortcuts.ts` overrides + SettingsDialog rebind UI); verified, no
  change needed.
- `mars_table` helper: every replayed program now injects
  `mars_table(&[columns], &[vec![rows]])`, which emits a
  `__MARS_TABLE__=<json>` stdout line parsed into `Output::Table` — Rust
  cells can render the existing table output without new dependencies.

## Outstanding

- Longer-lived compiler/runtime strategies remain a future optimization; the
  current replay model is benchmarked, documented, and retained as the
  validated isolation boundary.
- Runtime result extraction now supports multiline expressions, trailing line
  comments, and a Display fallback for values without `Debug`.
- Marketplace publishing and richer cell-aware editing remain optional VS Code
  follow-up work; the extension now has packaged branding, preview/export, run,
  and cell navigation workflows.
- More complete Jupyter compatibility beyond the representative metadata/MIME
  fixture, especially execution-specific widget behavior and structured table
  semantics.
- Full Tauri interactive launch/save/reopen tests in addition to the browser
  smoke tests.
- WebSocket/JSON-RPC transport adapters beyond the persistent JSONL transport.
- CLI HTML exports use a reliable offline static renderer for common math and
  chart specifications rather than bundled KaTeX/Vega JavaScript; interactive
  desktop rendering remains available in Tauri.
- `App.tsx` is now a composition/presentation layer. Notebook state/history/
  mutations, persistence, execution, editor preferences, UI state, keyboard
  handling, diagnostics, output rendering, and derived execution ranges are
  implemented in focused hooks/services/components.
- Native Tauri launch workflow and official Developer ID signing/notarization
  remain release-machine validation tasks.
- The VS Code extension is functional and compile/package-capable; marketplace
  publishing and deeper protocol-backed editing remain optional follow-up work.
- The Evcxr experiment is not adoption-ready: on the current macOS ARM64 host,
  its behavior run stalled while evaluating a later cell that reads an earlier
  binding, and the paired Evcxr benchmark could not be completed. Replay
  remains the default; see `docs/10-evcxr-experiment.md`.

## Validation

Run:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm exec turbo run check
pnpm exec turbo run build
```

Latest baseline validation on 2026-11-01:

- `pnpm exec turbo run check` passed for `@mars/desktop`, `@mars/landing`,
  and `@mars/vscode`.
- `pnpm exec turbo run build` passed for all three Node packages.
- `cargo test --workspace` passed, including all Rust unit and doc tests.
- The desktop build still reports the existing large-chunk warning for the
  CodeMirror and application bundles; it does not fail the build.

Evcxr experiment validation on 2026-11-01:

- `cargo check -p mars-runtime-evcxr` passed after locking the compatible
  `salsa-macro-rules` 0.28.2 patch selected by Evcxr's Rust Analyzer dependency.
- `cargo check -p mars --features experimental-evcxr` passed.
- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D
  warnings`, and `cargo test --workspace` passed. The Evcxr process scenarios
  are explicitly opt-in and were not part of the passing workspace run.
- The opt-in `evcxr_runtime` process test was attempted. It did not complete:
  Evcxr's controller stalled during a cell that reads a previous cell's value,
  while the evaluator child was idle. No Evcxr benchmark percentiles are
  claimed.

GPUI desktop (`crates/mars-desktop`) update:

- Notebook cells now render in the GPUI window. Root cause was a `gpui-pre 0.3.9` bug: children of a scrollable element are not painted when they are direct siblings; wrapping cell content in a single inner div (matching `gpui-component`'s `Scrollable` structure) fixed it. Verified via screenshot + OCR: all 57 cells of `examples/mars/01-getting-started/getting-started.mars` render with editors, controls, and outputs.
- Also fixed earlier: `gpui-pre-platform` required the `font-kit` feature for text rendering on macOS.
- Markdown cells now have a preview mode: an Eye toggle in the cell header swaps the textarea for a `StyledText`-rendered preview (headings, bold/italic/inline code, fenced code blocks) via `src/markdown_view.rs`. Verified visually.
- Per-cell execution status: results stream per-cell over an mpsc channel; each executing cell shows a "running…" indicator and `run_all` updates incrementally.
- Rich outputs: raster images render via GPUI `img()`, SVG images and Vega-Lite `Chart` outputs render via `svg().data()` (charts reuse `mars_renderer::chart::render_chart_svg`), `MimeBundle` prefers embedded `image/*` entries. HTML/LaTeX/table outputs remain text placeholders.
- Autosave: silent `save_atomic` every 2s while dirty with a known path.
- Keyboard shortcuts: Jupyter-style command mode — `escape` toggles, `a`/`b` insert, `m`/`y` convert, `d d` delete, `r` run, `s` save, `enter` edit, `up`/`down` select, `shift-enter` run cell. Command keys are scoped to a `MarsCommand` key context so typing is unaffected.

Known limitation: the runtime recompiles the accumulated source after each cell. This provides persistent bindings while remaining transparent and isolated, but side effects from earlier cells are replayed. Runtime compilation and execution can be bounded with a timeout or cancelled from the desktop; cancellation now terminates the dedicated process group/tree, but does not provide fine-grained cooperative hooks inside arbitrary user code. Automatic expression output supports multiline final expressions and trailing line comments, and retries with `Display` when a value does not implement `Debug`; item declarations and statement-only cells still need explicit output. Cargo dependencies support registry versions and inline Cargo table specifications, with optional notebook-embedded lockfiles and `--locked`/`--offline` controls. Jupyter conversion supports format 4 notebooks, standard text/errors/HTML/images/tables, LaTeX, Vega/Vega-Lite chart specs, display metadata, image variants, widget preservation, and unknown MIME bundles; execution-specific widget behavior and structured table semantics remain compatibility follow-up work. CLI HTML exports are self-contained and render common line, point, and bar charts as SVG with static math fallbacks. The browser smoke harness does not yet launch the native Tauri shell; native artifact build succeeds locally, while official signing/notarization and full save/reopen launch validation remain platform release checks.
