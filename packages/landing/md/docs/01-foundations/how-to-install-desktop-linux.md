---
title: Install desktop app on Linux
description: Install the Project Mars desktop app on Linux.
order: 32
author: Project Mars Team
tags: [install, desktop]
---

# How to install the Project Mars desktop app on Linux

For x86_64 Linux distributions.

## Install script

```sh
curl -fsSL https://raw.githubusercontent.com/hieudoanm/mars/master/scripts/sh/install.sh | bash -- --release --desktop
```

The script detects your platform and downloads the matching release asset.

Choose the package format that matches your distribution — all are
listed under **Linux** on the
[downloads page](https://hieudoanm.github.io/mars/downloads/).

## Debian / Ubuntu (`.deb`)

```sh
sudo apt install ./desktop-mars-x86_64-unknown-linux-gnu.deb
```

(or `sudo dpkg -i desktop-mars-x86_64-unknown-linux-gnu.deb && sudo apt --fix-broken install`)

## Fedora / RHEL / openSUSE (`.rpm`)

```sh
sudo dnf install ./desktop-mars-x86_64-unknown-linux-gnu.rpm        # Fedora / RHEL
sudo zypper install ./desktop-mars-x86_64-unknown-linux-gnu.rpm     # openSUSE
```

## Any distribution (`.AppImage`)

```sh
chmod +x desktop-mars-x86_64-unknown-linux-gnu.AppImage
sudo mv desktop-mars-x86_64-unknown-linux-gnu.AppImage /opt/project-mars
sudo ln -s /opt/project-mars/desktop-mars-x86_64-unknown-linux-gnu.AppImage ~/.local/bin/project-mars
```

Then run `project-mars` — no installation needed, the
AppImage is self-contained.

## Verifying

Launch Project Mars from your application menu, or run the binary/AppImage
from a terminal.
