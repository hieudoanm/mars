---
title: Install CLI on Linux
description: Install the Project Mars CLI on Linux.
order: 22
author: Project Mars Team
tags: [install, cli]
---

# How to install the mars CLI on Linux

## Option 1: install script (recommended)

```sh
curl -fsSL https://raw.githubusercontent.com/hieudoanm/mars/master/scripts/sh/install.sh | bash -- --release --cli
```

The script downloads `cli-mars-x86_64-unknown-linux-gnu` from the
[latest release](https://github.com/hieudoanm/mars/releases/latest) and
installs `mars` to `~/.local/bin`.

## Option 2: manual download

1. Open the [downloads page](https://hieudoanm.github.io/mars/downloads/) and download **Linux**
   under **CLI** — `cli-mars-x86_64-unknown-linux-gnu`.
2. Make it executable and move it onto your `PATH`:

```sh
chmod +x cli-mars-x86_64-unknown-linux-gnu
mv cli-mars-x86_64-unknown-linux-gnu ~/.local/bin/mars    # Windows: rename to mars.exe on PATH
```

3. Verify:

```sh
mars --version
```

## Troubleshooting

- `mars: command not found` — add `~/.local/bin` to your `PATH`.
- SmartScreen/Gatekeeper — the binaries are unsigned; approve the first run.
