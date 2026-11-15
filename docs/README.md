# Project Mars documentation

This directory contains the project documentation for **Project Mars**, a Rust-powered
interactive notebook platform with multi-language support (Rust, Python, R, Julia, C++, SQL),
Model Context Protocol integration, and the **Mars Lab** machine learning studio.
Documents are grouped into numbered chapter folders (`01-foundations` through
`06-appendix`) that follow the recommended reading order; within each folder the
file order is alphabetical.

## Chapters

### 01-foundations/

| File | Description |
| --- | --- |
| [getting-started.md](./01-foundations/getting-started.md) | Install, first notebook, examples |
| [notebook-format.md](./01-foundations/notebook-format.md) | `.mars` notebook document format |
| [architecture.md](./01-foundations/architecture.md) | Repository layout and layering |

### 02-platform/

| File | Description |
| --- | --- |
| [protocol.md](./02-platform/protocol.md) | Frontend↔kernel protocol |
| [kernel-protocol.md](./02-platform/kernel-protocol.md) | Kernel protocol and future architecture |
| [runtime.md](./02-platform/runtime.md) | Runtime architecture overview |
| [runtime-rust.md](./02-platform/runtime-rust.md) | Rust replay and Evcxr backends |
| [runtime-python.md](./02-platform/runtime-python.md) | Python persistent driver |
| [runtime-r.md](./02-platform/runtime-r.md) | R `.GlobalEnv` persistence |
| [runtime-julia.md](./02-platform/runtime-julia.md) | Julia `MarsKernel` module |
| [runtime-cpp.md](./02-platform/runtime-cpp.md) | C++ replay compilation |
| [runtime-sql.md](./02-platform/runtime-sql.md) | In-process SQLite runtime |
| [runtime-helpers.md](./02-platform/runtime-helpers.md) | Runtime helpers, directives, attachments |
| [cli.md](./02-platform/cli.md) | Command-line interface reference |
| [mcp.md](./02-platform/mcp.md) | Model Context Protocol integration |

### 03-lab/

| File | Description |
| --- | --- |
| [lab-overview.md](./03-lab/lab-overview.md) | Mars Lab features and architecture |
| [lab-cli.md](./03-lab/lab-cli.md) | Mars Lab CLI commands and examples |
| [lab-algorithms.md](./03-lab/lab-algorithms.md) | Complete algorithm catalogue |
| [lab-workflows.md](./03-lab/lab-workflows.md) | Common training workflows |

### 04-extensions/

| File | Description |
| --- | --- |
| [extension-vscode.md](./04-extensions/extension-vscode.md) | VS Code extension guide |
| [extension-zed.md](./04-extensions/extension-zed.md) | Zed extension guide |
| [extension-jetbrains.md](./04-extensions/extension-jetbrains.md) | JetBrains plugin guide |

### 05-development/

| File | Description |
| --- | --- |
| [development.md](./05-development/development.md) | Local development commands |
| [release.md](./05-development/release.md) | Release process and checks |

### 06-appendix/

| File | Description |
| --- | --- |
| [evcxr-experiment.md](./06-appendix/evcxr-experiment.md) | Evcxr runtime experiment |
| [branding.md](./06-appendix/branding.md) | Visual identity and logo usage |

## Related

- [Root README](../README.md)
- [Contributing guide](../CONTRIBUTING.md)
- [Agent guide](../AGENTS.md)
