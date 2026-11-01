# Mars documentation

This directory contains the project documentation for **Mars**, a Rust-powered
interactive notebook platform. Files are prefixed with a number to suggest a
reading order.

## Reading order

| # | File | Description |
|---|------|-------------|
| 01 | [README.md](./README.md) | Docs index (this page) |
| 02 | [02-branding.md](./02-branding.md) | Visual identity and logo usage |
| 03 | [03-notebook-format.md](./03-notebook-format.md) | `.mars` notebook document format |
| 04 | [04-architecture.md](./04-architecture.md) | Repository layout and layering |
| 05 | [05-protocol.md](./05-protocol.md) | Frontend↔kernel protocol |
| 06 | [06-runtime.md](./06-runtime.md) | Persistent Rust execution runtime |
| 07 | [07-cli.md](./07-cli.md) | Command-line interface reference |
| 08 | [08-development.md](./08-development.md) | Local development commands |
| 09 | [09-release.md](./09-release.md) | Release process and checks |
| 10 | [10-evcxr-experiment.md](./10-evcxr-experiment.md) | Evcxr runtime experiment |

## Screenshots

See [screenshots/README.md](./screenshots/README.md) for captured and
recommended release screenshots.

## Related

- Landing app docs: `apps/landing/src/lib/docs.ts` (serves these files at `/docs`)
- Desktop app: `apps/desktop/`
- VS Code extension: `apps/vscode/`
- Rust crates: `crates/`
