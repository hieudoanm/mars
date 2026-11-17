# Project Mars (latest)

Mars is an interactive computing platform: **Mars Notebook** (a Jupyter alternative with a native `.mars` format, multi-language kernels, and HTML/Markdown export) and **Mars Lab** (ML studio: CSV → trained ONNX models, 83 algorithms). Ships a `mars` CLI, desktop app, MCP server, and VS Code/Zed/JetBrains extensions.

## Download

> **macOS:** the desktop builds are not signed or notarized. On first launch,
> right-click the app → **Open**, or run
> `xattr -dr com.apple.quarantine /Applications/Project\ Mars.app`.

Or install everything prebuilt: `curl -fsSL https://raw.githubusercontent.com/hieudoanm/mars/master/scripts/sh/install.sh | bash -s -- --release`

- [All assets][release]
- **CLI**:
  - [Linux x86_64][cli-linux]
  - [macOS Apple Silicon][cli-macos-arm]
  - [macOS Intel][cli-macos-intel]
  - [Windows x64 (.exe)][cli-windows]
- **Desktop**:
  - [macOS Apple Silicon (.dmg)][dmg-arm] / [.app.zip][appzip-arm]
  - [macOS Intel (.dmg)][dmg-intel] / [.app.zip][appzip-intel]
  - [Linux (.deb)][deb] / [.rpm][rpm] / [.AppImage][appimage]
  - [Windows (.msi)][msi] / [NSIS setup (.exe)][nsis]
- **Extensions**:
  - [JetBrains plugin zip][release]
  - [VS Code (.vsix)][vsix]
  - [Zed (.wasm)][wasm]

## Quick start

```sh
mars notebook new demo.mars
mars notebook run demo.mars
mars lab dataset data.csv    # prints the dataset UUID
mars lab train --dataset-id <uuid> --task classification \
  --algorithm random_forest_classifier --features age,income --label target
mars lab export --job-id <uuid> --output model.onnx
mars mcp
```

## Documentation

- [README][readme]
- [Docs][docs]

<!-- Link references -->

[release]: https://github.com/hieudoanm/mars/releases/latest
[cli-linux]: https://github.com/hieudoanm/mars/releases/latest/download/cli-mars-x86_64-unknown-linux-gnu
[cli-macos-arm]: https://github.com/hieudoanm/mars/releases/latest/download/cli-mars-aarch64-apple-darwin
[cli-macos-intel]: https://github.com/hieudoanm/mars/releases/latest/download/cli-mars-x86_64-apple-darwin
[cli-windows]: https://github.com/hieudoanm/mars/releases/latest/download/cli-mars-x86_64-pc-windows-msvc.exe
[dmg-arm]: https://github.com/hieudoanm/mars/releases/latest/download/desktop-mars-aarch64-apple-darwin.dmg
[appzip-arm]: https://github.com/hieudoanm/mars/releases/latest/download/desktop-mars-aarch64-apple-darwin.app.zip
[dmg-intel]: https://github.com/hieudoanm/mars/releases/latest/download/desktop-mars-x86_64-apple-darwin.dmg
[appzip-intel]: https://github.com/hieudoanm/mars/releases/latest/download/desktop-mars-x86_64-apple-darwin.app.zip
[deb]: https://github.com/hieudoanm/mars/releases/latest/download/desktop-mars-x86_64-unknown-linux-gnu.deb
[rpm]: https://github.com/hieudoanm/mars/releases/latest/download/desktop-mars-x86_64-unknown-linux-gnu.rpm
[appimage]: https://github.com/hieudoanm/mars/releases/latest/download/desktop-mars-x86_64-unknown-linux-gnu.AppImage
[msi]: https://github.com/hieudoanm/mars/releases/latest/download/desktop-mars-x86_64-pc-windows-msvc.msi
[nsis]: https://github.com/hieudoanm/mars/releases/latest/download/desktop-mars-x86_64-pc-windows-msvc-setup.exe
[vsix]: https://github.com/hieudoanm/mars/releases/latest/download/extension-vscode-mars.vsix
[wasm]: https://github.com/hieudoanm/mars/releases/latest/download/extension-zed-mars.wasm
[readme]: https://github.com/hieudoanm/mars#readme
[docs]: https://github.com/hieudoanm/mars/tree/master/docs
