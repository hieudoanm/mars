# Mars release guide

This guide describes a release as a set of independently verifiable Rust,
Node, and native desktop artifacts. A green frontend build is not sufficient
evidence that a Tauri bundle launches on a target operating system.

## Release checks

Run the checks from a clean checkout:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

corepack enable
corepack prepare pnpm@10.12.1 --activate
pnpm install --frozen-lockfile
pnpm exec turbo run check
pnpm exec turbo run build
pnpm run desktop:e2e

cd packages/desktop
npm run tauri info
npm run tauri build
```

The browser tests validate the Vite/React surface. They do not prove that a
native Tauri bundle launches; perform a native smoke test on each target
platform before publishing.

## Building desktop artifacts

From `packages/desktop`, build the platform-native bundle:

```bash
npm run tauri build
```

Tauri writes artifacts under `packages/desktop/src-tauri/target/release/bundle/`.
The exact formats depend on the host and configured Tauri targets:

- macOS: `.app` and, when configured, `.dmg`
- Linux: distribution-specific bundles such as `.AppImage`, `.deb`, or `.rpm`
- Windows: `.msi` and/or NSIS installers

The current development environment has the frontend dependencies but may not
have the Tauri CLI binary installed. In that case `npm run tauri info` and
`npm run tauri build` must be run on a configured release machine; do not treat
the browser build as a substitute for native artifact validation.

The landing page can be deployed from `packages/landing/out` after:

```bash
pnpm --filter @mars/landing build
```

The build statically exports the marketing page and the Markdown documentation
served at `/docs`, including one page per document in `packages/landing/docs`.

The VS Code package is built with:

```bash
pnpm run vscode:package
```

The resulting `.vsix` is a companion artifact, not part of the native desktop
bundle.

Do not publish unsigned artifacts as an official release. Configure platform
signing and notarization credentials in the CI secret store, never in the
repository or local configuration files.

## Versioning

Keep the application version in `packages/desktop/src-tauri/tauri.conf.json` and
the package version in `packages/desktop/package.json` aligned for a desktop
release. Update the workspace/crate version when publishing Rust packages.
Record user-visible changes in the release notes and update `PROGRESS.md` when
the supported workflow changes.

## Artifact verification

Before publishing:

1. Inspect the generated bundle names and target architecture.
2. Install or launch the artifact on the target operating system.
3. Create a notebook, execute Rust cells, save it, close Mars, and reopen it.
4. Verify that `.mars` files, Jupyter import/export, rich outputs, and native
   dialogs work.
5. Record known platform limitations and checksums with the release.

For the landing page, verify that the generated site serves the favicon,
Open Graph image, logo assets, JavaScript bundle, and CSS bundle from the
deployed base path, and that `/docs` and every `/docs/<slug>/` page load
without missing assets. For the VS Code extension, install the `.vsix` into a
clean VS Code profile and verify `.mars` language activation, preview/export,
cell navigation, and CLI-backed execution.

## CI guidance

Use separate jobs for Rust checks, frontend checks, and native packaging.
Native packaging should run on the operating system it targets. Keep signing
steps isolated from ordinary pull-request checks and require explicit release
tag approval.

The local macOS build currently produces an ad-hoc-signed `.app` and a DMG.
The artifact checksum can be recorded with:

```bash
shasum -a 256 target/release/bundle/dmg/*.dmg
```

Ad-hoc signatures are suitable for local launch checks only. Official
distribution still requires a Developer ID signing identity and notarization.
