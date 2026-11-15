# Project Mars documentation

This directory contains the project documentation for **Project Mars**, a Rust-powered
interactive notebook platform with multi-language support (Rust, Python, R, Julia, C++)
and Model Context Protocol integration. Files are prefixed with a number to suggest a
reading order.

## Reading order

| #   | File                                               | Description                              |
| --- | -------------------------------------------------- | ---------------------------------------- |
| 01  | [README.md](./README.md)                           | Docs index (this page)                   |
| 02  | [02-branding.md](./02-branding.md)                 | Visual identity and logo usage           |
| 03  | [03-notebook-format.md](./03-notebook-format.md)   | `.mars` notebook document format         |
| 04  | [04-architecture.md](./04-architecture.md)         | Repository layout and layering           |
| 05  | [05-protocol.md](./05-protocol.md)                 | Frontend↔kernel protocol                 |
| 06  | [06-runtime.md](./06-runtime.md)                   | Runtime architecture overview            |
| 07  | [07-runtime-rust.md](./07-runtime-rust.md)         | Rust replay and Evcxr backends           |
| 08  | [08-runtime-python.md](./08-runtime-python.md)     | Python persistent driver                 |
| 09  | [09-runtime-r.md](./09-runtime-r.md)               | R .GlobalEnv persistence                 |
| 10  | [10-runtime-julia.md](./10-runtime-julia.md)       | Julia MarsKernel module                  |
| 11  | [11-runtime-cpp.md](./11-runtime-cpp.md)           | C++ replay compilation                   |
| 12  | [12-cli.md](./12-cli.md)                           | Command-line interface reference         |
| 13  | [13-development.md](./13-development.md)           | Local development commands               |
| 14  | [14-release.md](./14-release.md)                   | Release process and checks               |
| 15  | [15-evcxr-experiment.md](./15-evcxr-experiment.md) | Evcxr runtime experiment                 |
| 16  | [16-runtime-helpers.md](./16-runtime-helpers.md)   | Runtime helpers, directives, attachments |
| 17  | [17-kernel-protocol.md](./17-kernel-protocol.md)   | Kernel protocol and future architecture  |
| 18  | [18-mcp.md](./18-mcp.md)                           | Model Context Protocol integration       |

## Related

- Landing app docs: `packages/landing/md/docs/` (comprehensive multi-language and extension documentation)
- Desktop app: `packages/desktop/`
- VS Code extension: `extensions/vscode/`
- Zed extension: `extensions/zed/`
- Rust crates: `crates/`
