---
title: Install desktop app on macOS (Intel)
description: Install the Project Mars desktop app on macOS (Intel).
order: 31
author: Project Mars Team
tags: [install, desktop]
---

# How to install the Project Mars desktop app on macOS (Intel)

For Macs with an Intel chip.

## Install script

```sh
curl -fsSL https://raw.githubusercontent.com/hieudoanm/mars/master/scripts/sh/install.sh | bash -- --release --desktop
```

The script detects your platform and downloads the matching release asset.

## Option 1: disk image (recommended)

1. Open the [downloads page](https://hieudoanm.github.io/mars/downloads/) and download **macOS (Intel)**
   under **Desktop** — `desktop-mars-x86_64-apple-darwin.dmg`.
2. Open the `.dmg` and drag **Project Mars.app** into `/Applications`.
3. Eject the disk image and launch Project Mars from `/Applications`.

## Option 2: zipped app bundle

If the `.dmg` does not work, download `desktop-mars-x86_64-apple-darwin.app.zip` instead, unzip it, and move
**Project Mars.app** to `/Applications`:

```sh
unzip desktop-mars-x86_64-apple-darwin.app.zip
mv "Project Mars.app" /Applications/   # adjust to the unzipped name
```

## First launch (Gatekeeper)

The app is unsigned, so macOS may block the first launch. Either right-click
**Project Mars.app** → **Open** → **Open**, or strip the quarantine attribute:

```sh
xattr -dr com.apple.quarantine /Applications/Project Mars.app
```

## Verifying

Launch Project Mars from `/Applications`, or from a terminal:

```sh
open -a "Project Mars"
```
