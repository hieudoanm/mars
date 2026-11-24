---
title: Install zed extension
description: Install the Project Mars zed extension.
order: 41
author: Project Mars Team
tags: [install, extension]
---

# How to install the Mars extension for Zed

## Option 1: install script (recommended)

```sh
curl -fsSL https://raw.githubusercontent.com/hieudoanm/mars/master/scripts/sh/install.sh | bash -- --release --zed
```

The script downloads `extension-zed-mars.tar.gz` from the
[latest release](https://github.com/hieudoanm/mars/releases/latest).

## Option 2: manual download

1. Open the [downloads page](https://hieudoanm.github.io/mars/downloads/) and download **Zed**
   under **Editor Extensions** — `extension-zed-mars.tar.gz`.
2. Extract and install:

```sh
tar -xzf extension-zed-mars.tar.gz -C ~/zed-mars-extension
```

3. In Zed, run **zed: install** from the command palette and select
   `~/zed-mars-extension` (the directory containing `extension.toml`).
