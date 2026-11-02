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
  artifact verification, and CI guidance in `docs/release.md` and
  `docs/development.md`.
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
  plus `docs/evcxr-experiment.md` with setup, compatibility notes, and the
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
  remains the default; see `docs/evcxr-experiment.md`.

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

Known limitation: the runtime recompiles the accumulated source after each cell. This provides persistent bindings while remaining transparent and isolated, but side effects from earlier cells are replayed. Runtime compilation and execution can be bounded with a timeout or cancelled from the desktop; cancellation now terminates the dedicated process group/tree, but does not provide fine-grained cooperative hooks inside arbitrary user code. Automatic expression output supports multiline final expressions and trailing line comments, and retries with `Display` when a value does not implement `Debug`; item declarations and statement-only cells still need explicit output. Cargo dependencies support registry versions and inline Cargo table specifications, with optional notebook-embedded lockfiles and `--locked`/`--offline` controls. Jupyter conversion supports format 4 notebooks, standard text/errors/HTML/images/tables, LaTeX, Vega/Vega-Lite chart specs, display metadata, image variants, widget preservation, and unknown MIME bundles; execution-specific widget behavior and structured table semantics remain compatibility follow-up work. CLI HTML exports are self-contained and render common line, point, and bar charts as SVG with static math fallbacks. The browser smoke harness does not yet launch the native Tauri shell; native artifact build succeeds locally, while official signing/notarization and full save/reopen launch validation remain platform release checks.
