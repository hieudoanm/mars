## Project Mars — latest build

Mars is an interactive computing platform: **Mars Notebook** (a
Jupyter alternative with a native `.mars` format, multi-language
kernels, and HTML/Markdown export) and **Mars Lab** (ML studio:
CSV → trained ONNX models, 83 algorithms). Ships a `mars` CLI, desktop
app, MCP server, and VS Code/Zed/JetBrains extensions.

### Download

- **CLI**:
  - `mars-x86_64-unknown-linux-gnu`
  - `mars-aarch64-apple-darwin`
  - `mars-x86_64-pc-windows-msvc.exe`
- **Desktop**:
  - macOS app bundle/zip
  - Linux `.deb`/`.rpm`/`.AppImage`
  - Windows `.msi`/`.exe`
- **Extensions**:
  - JetBrains plugin zip
  - VS Code `.vsix`
  - Zed `.wasm`

### Quick start

```sh
mars notebook new demo.mars
mars notebook run demo.mars
mars lab dataset data.csv    # prints the dataset UUID
mars lab train --dataset-id <uuid> --task classification \
  --algorithm random_forest_classifier --features age,income --label target
mars lab export --job-id <uuid> --output model.onnx
mars mcp
```

### Documentation

- [README](https://github.com/hieudoanm/mars#readme)
- [Docs](https://github.com/hieudoanm/mars/tree/master/docs)
