# Mars Notebooks for VS Code

![Mars](assets/mars-og.png)

This companion extension provides `.mars` syntax recognition, Rust/Markdown
embedded highlighting, cell navigation, configurable CLI-backed HTML export,
notebook preview, and headless notebook execution. It intentionally reuses Mars rather than duplicating
notebook execution or persistence.

## Development

The extension is the `@mars/vscode` package in the root pnpm workspace. The
package-local npm commands remain supported for contributors who work only on
the extension.

```bash
pnpm install
pnpm run vscode:compile
pnpm run vscode:package
```

Equivalent package-local commands are:

```bash
cd apps/vscode
npm install
npm run compile
npm run package
```

Install the Mars CLI first and ensure it is available on `PATH`, or configure
`mars.cliPath`. Use **Mars: Preview Notebook** or **Mars: Export Notebook to
HTML** from an open `.mars` file. `npm run package` creates a `.vsix` file for
local installation or marketplace review.

Use **Mars: Run Notebook** to execute the active notebook through the Mars CLI.
Optional arguments can be configured with `mars.runArguments`, while
`mars.cliPath` selects a project-local or globally installed CLI.

The extension package includes the Mars project mark in
[`assets/mars-logo.png`](assets/mars-logo.png) and the notebook-compatible
file mark in [`assets/mars-file-logo.png`](assets/mars-file-logo.png).
The package also includes the Open Graph artwork at
[`assets/mars-og.png`](assets/mars-og.png) for project documentation.
