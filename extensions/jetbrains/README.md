# Project Mars for JetBrains IDEs

IntelliJ Platform plugin for [Project Mars](https://github.com/hieudoanm/mars) notebooks
(`.mars`). Kotlin, built with the `org.jetbrains.intellij` Gradle plugin.

## Features

- **`.mars` file type** — registered with the JSON language, so notebooks get
  JSON-aware syntax highlighting, folding, and structure view for free.
- **Project Mars Kernel tool window** — right-side tool window showing kernel status,
  execution count, live session variables, cell execution output, and kernel
  stderr. Refresh and Interrupt buttons included.
- **Cell execution** — `Project Mars → Execute Cell at Caret` (Ctrl+Alt+Enter) locates
  the code cell under the caret and runs it through `mars kernel serve` over
  the newline-delimited JSON kernel protocol (same protocol as the VS Code
  extension). Honors per-cell `language` metadata (`Execute.language`),
  falling back to `metadata.language`.
- **Cell navigation** — `Project Mars → Next Cell` / `Previous Cell` move the caret
  between `cells[]` entries.
- **Notebook actions** — Execute All Cells, Check Notebook (compile-check via
  `diagnose`), Send Input to Kernel (stdin), Create/Restore Checkpoint,
  Run Notebook (`mars run`), Export to HTML / Markdown / `.ipynb`, Inspect
  (`mars inspect`), Restart Kernel, Interrupt Execution.
- **Configurable CLI path** — Settings → Tools → Project Mars.

## Requirements

- A JetBrains IDE 2024.3+ (IntelliJ IDEA, PyCharm, etc.).
- The `mars` CLI on `PATH`, or set the path in Settings → Tools → Project Mars.
- For Python/R cells: the corresponding Project Mars runtime backend available to the
  CLI (`python3`, `Rscript`).

## Known limitations

- JetBrains IDEs have **no notebook-cell editor API** (unlike VS Code's
  Notebook API). Cells are edited as raw JSON inside the `.mars` document;
  there is no per-cell gutter, output pane, or markdown rendering. Execution
  results are shown in the Project Mars Kernel tool window and are **not** written
  back into the file's `outputs` arrays — use `mars run` / the desktop app
  for full output persistence.
- "Cell at caret" resolution pairs `cells[]` entries with `"source"` keys in
  document order; it assumes `.mars` files keep the canonical cell structure.
- No per-notebook trust model for rich output; outputs are displayed as text.

## Build & run

```bash
cd extensions/jetbrains
./gradlew runIde   # launches a sandbox IDE with the plugin installed
```

To build a distributable plugin zip:

```bash
./gradlew buildPlugin
```

The plugin targets the 2024.3 IntelliJ Platform (`IC`); adjust
`platformVersion` in `gradle.properties`/`build.gradle.kts` to match your IDE.

## Layout

```text
src/main/
├── resources/META-INF/plugin.xml     # extensions, actions, tool window
└── kotlin/com/mars/idea/
    ├── MarsFileType.kt               # .mars file type (JSON language)
    ├── MarsSettings*.kt              # CLI path setting
    ├── actions/                      # run/export/inspect/kernel actions
    ├── kernel/
    │   ├── MarsKernelClient.kt       # JSON-lines kernel protocol client
    │   ├── MarsKernelManager.kt      # per-notebook client registry
    │   └── MarsCellLocator.kt        # caret → cell resolution
    └── ui/MarsToolWindow.kt          # status/variables/output panel
```
