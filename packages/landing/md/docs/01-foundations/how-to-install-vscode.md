---
title: Install vscode extension
description: Install the Project Mars vscode extension.
order: 40
author: Project Mars Team
tags: [install, extension]
---

# How to install the Mars extension for VS Code

## Option 1: install script (recommended)

```sh
curl -fsSL https://raw.githubusercontent.com/hieudoanm/mars/master/scripts/sh/install.sh | bash -- --release --vscode
```

The script downloads `extension-vscode-mars.vsix` from the
[latest release](https://github.com/hieudoanm/mars/releases/latest).

## Option 2: manual install

1. Open the [downloads page](https://hieudoanm.github.io/mars/downloads/) and download **VS Code**
   under **Editor Extensions** — `extension-vscode-mars.vsix`.
2. In VS Code, open the Extensions view (`Cmd+Shift+X` / `Ctrl+Shift+X`),
   click **… → Install from VSIX…**, and pick the file.

## Verifying

Open the Command Palette and run a `mars:` command, or look for the extension
in the Extensions view.
