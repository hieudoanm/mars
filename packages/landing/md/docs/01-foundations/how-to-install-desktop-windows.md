---
title: Install desktop app on Windows
description: Install the Project Mars desktop app on Windows.
order: 33
author: Project Mars Team
tags: [install, desktop]
---

# How to install the Project Mars desktop app on Windows

For 64-bit Windows.

## Install script

```sh
curl -fsSL https://raw.githubusercontent.com/hieudoanm/mars/master/scripts/sh/install.sh | bash -- --release --desktop
```

The script detects your platform and downloads the matching release asset.

## Option 1: MSI installer

1. Open the [downloads page](https://hieudoanm.github.io/mars/downloads/) and download **Windows**
   under **Desktop** — `desktop-mars-x86_64-pc-windows-msvc.msi`.
2. Double-click the `.msi` and follow the installer wizard.

## Option 2: setup executable

Download `desktop-mars-x86_64-pc-windows-msvc-setup.exe` instead and run it — it is the NSIS installer equivalent.

## SmartScreen

The installer is unsigned, so Windows SmartScreen may warn on first run.
Choose **More info → Run anyway**.

## Verifying

Launch Project Mars from the Start menu.
