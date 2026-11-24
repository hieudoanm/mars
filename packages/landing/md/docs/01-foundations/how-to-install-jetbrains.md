---
title: Install jetbrains extension
description: Install the Project Mars jetbrains extension.
order: 42
author: Project Mars Team
tags: [install, extension]
---

# How to install the Mars plugin for JetBrains IDEs

The plugin needs the `mars` CLI on your `PATH` —
[install it first](/docs/how-to-install-cli-macos-silicon/) (or set the binary
path in settings).

## Option 1: install script (recommended)

```sh
curl -fsSL https://raw.githubusercontent.com/hieudoanm/mars/master/scripts/sh/install.sh | bash -- --release --jetbrains
```

## Option 2: manual download

1. Open the [downloads page](https://hieudoanm.github.io/mars/downloads/) and download **JetBrains**
   under **Editor Extensions** — `extension-jetbrains-mars-0.0.1.zip`.
2. Open **Settings → Plugins → ⚙ → Install Plugin from Disk…** and pick the zip.

## Configuring the CLI path

If `mars` is not on your `PATH`, set it under the plugin's settings to the
absolute binary path (e.g. `~/.local/bin/mars`).
