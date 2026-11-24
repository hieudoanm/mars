---
title: Install CLI on macOS (Apple Silicon)
description: Install the Project Mars CLI on macOS (Apple Silicon).
order: 20
author: Project Mars Team
tags: [install, cli]
---

# How to install the mars CLI on macOS (Apple Silicon)

## Option 1: install script (recommended)

```sh
curl -fsSL https://raw.githubusercontent.com/hieudoanm/mars/master/scripts/sh/install.sh | bash -- --release --cli
```

The script downloads `cli-mars-aarch64-apple-darwin` from the
[latest release](https://github.com/hieudoanm/mars/releases/latest) and
installs `mars` to `~/.local/bin`.

## Option 2: manual download

1. Open the [downloads page](https://hieudoanm.github.io/mars/downloads/) and download **macOS (Apple Silicon)**
   under **CLI** — `cli-mars-aarch64-apple-darwin`.
2. Make it executable and move it onto your `PATH`:

```sh
chmod +x cli-mars-aarch64-apple-darwin
mv cli-mars-aarch64-apple-darwin ~/.local/bin/mars    # Windows: rename to mars.exe on PATH
```

3. Verify:

```sh
mars --version
```

## Troubleshooting

- `mars: command not found` — add `~/.local/bin` to your `PATH`.
- SmartScreen/Gatekeeper — the binaries are unsigned; approve the first run.
