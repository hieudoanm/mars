# Mars Scripts

This directory contains installation and setup scripts for the Mars project.

## install.sh

Installs the Mars CLI and VSCode extension:

```bash
./scripts/install.sh
```

This script will:
1. Check for required dependencies (Rust/Cargo, VSCode)
2. Build the Mars CLI from source using `cargo build --release`
3. Install the Mars CLI to `~/.local/bin/mars`
4. Build the VSCode extension
5. Install the extension in VSCode

### Requirements

- Rust and Cargo (install from https://rustup.rs/)
- Visual Studio Code with the `code` command in PATH
- Node.js and npm (for building the VSCode extension)

### Notes

The script will add `~/.local/bin` to your PATH for the current session. To make this permanent, add the following to your shell profile (`~/.zshrc` or `~/.bashrc`):

```bash
export PATH="$HOME/.local/bin:$PATH"
```
