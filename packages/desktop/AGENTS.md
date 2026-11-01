# Mars desktop application

Scope: `packages/desktop/**`. Follow the repository-level `AGENTS.md` first; these
instructions describe the Tauri desktop client specifically.

## Architecture and ownership

- This is the Mars notebook desktop client: React 19 + TypeScript in `src/`,
  with a Tauri 2 Rust host in `src-tauri/`.
- Keep the frontend thin. Document mutations belong in `src/lib/notebookOperations.ts`;
  notebook UI state and side effects belong in focused hooks under `src/hooks/`.
  React components should render state and delegate actions rather than own
  persistence or kernel business logic.
- `src/types/types.ts` mirrors the public notebook and kernel payloads. Treat
  changes there as protocol changes: update the Rust commands, frontend callers,
  and tests together.
- `src/lib/kernelService.ts` is the frontend boundary for every Tauri command.
  Do not call `invoke` directly from components or hooks. Keep command names and
  argument casing synchronized with `src-tauri/src/lib.rs`.
- The Rust host owns filesystem access and kernel lifecycle. Use
  `mars-document` for parsing and atomic persistence, `mars-kernel` for
  execution, and `mars-renderer` for export; do not duplicate those concerns in
  the UI or execute notebook source in the Tauri process outside the kernel.
- Keep `App.tsx` as composition only. Add reusable visual primitives to
  `src/components/shared/`, domain components to their existing feature folder,
  and colocated tests to the matching `__tests__/` directory.
- Keep production source files at or below 200 lines where practical (220 is
  the absolute sensible exception). Extract a component, hook, or pure helper
  instead of accumulating unrelated responsibilities.

## UI, outputs, and security

- Preserve the default dark Mars visual language and existing Tailwind class
  ordering; format with the local Prettier configuration after style edits.
- Rich output is untrusted notebook content. Route HTML through
  `sanitizeHtml.ts`; never add `dangerouslySetInnerHTML` or relax the Tauri CSP
  merely to render an output.
- Preserve all output variants in `Output`, including MIME bundles and their
  metadata. Do not collapse unknown MIME data or notebook metadata while editing
  a cell or saving a notebook.
- Maintain keyboard shortcut behaviour in `useKeyboardShortcuts.ts`, including
  platform-aware Ctrl/Cmd handling and avoiding interception while text inputs
  or CodeMirror own the event.
- Use `unknown` plus narrowing for external data. Do not introduce `any`,
  unchecked JSON assumptions, `unwrap`, or `expect` in production Rust.

## Tauri boundary

- Every new command needs explicit error conversion and belongs in the
  `generate_handler!` list. Keep paths, collisions, and user-visible file
  errors validated in Rust rather than trusting the renderer.
- Preserve cancellation, timeout, and reset semantics when changing execution.
  Kernel state is intentionally process-local and must not be recreated by
  unrelated UI changes.
- Update `src-tauri/tauri.conf.json` deliberately. The CSP must remain as
  restrictive as possible, and frontend build commands/ports must match the
  package scripts.

## Validation

Run the smallest relevant checks from the repository root:

```bash
pnpm --filter @mars/desktop check
pnpm --filter @mars/desktop lint
pnpm --filter @mars/desktop test
pnpm --filter @mars/desktop build
cargo fmt --all --check
cargo clippy -p mars-tauri --all-targets -- -D warnings
cargo test -p mars-tauri
```

Run `pnpm --filter @mars/desktop test:e2e` for changed end-user notebook flows
when the local browser/runtime prerequisites are available. Do not commit
`dist/`, `coverage/`, or other generated build artifacts.
