## Project Mars — latest build

Mars is an interactive computing platform: **Mars Notebook** (a
Jupyter alternative with a native `.mars` format, multi-language
kernels, and HTML/Markdown export) and **Mars Lab** (ML studio:
CSV → trained ONNX models, 83 algorithms). Ships a `mars` CLI, desktop
app, MCP server, and VS Code/Zed/JetBrains extensions.

### Download

- **CLI**: `mars-x86_64-unknown-linux-gnu`,
  `mars-aarch64-apple-darwin`, `mars-x86_64-pc-windows-msvc.exe`
- **Desktop**: macOS app bundle/zip, Linux `.deb`/`.rpm`/`.AppImage`,
  Windows `.msi`/`.exe` — see assets
- **Extensions**: VS Code `.vsix`, Zed `.wasm`, JetBrains plugin zip

### Quick start

```sh
mars notebook create demo.mars
mars notebook run demo.mars
mars lab train --dataset data.csv --algorithm random_forest
mars mcp
```

### Documentation

- [README](https://github.com/hieudoanm/mars#readme)
- [Docs](https://github.com/hieudoanm/mars/tree/master/docs)
