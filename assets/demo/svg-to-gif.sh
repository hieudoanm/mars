#!/usr/bin/env bash
#
# svg-to-gif.sh — Regenerate assets/demo/{svg,png,gif} for the terminal demo.
#
# The terminal session is defined in scripts/demo-sessions/mars.json and
# rendered by hieudoanm/scripts/demo/terminal_gif.py (shared across repos).
# Requires: rsvg-convert, ffmpeg.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
SESSION="$REPO_ROOT/scripts/demo-sessions/mars.json"
GENERATOR="$(cd "$REPO_ROOT/.." && pwd)/hieudoanm/scripts/demo/terminal_gif.py"

python3 "$GENERATOR" "$REPO_ROOT" "mars — zsh" "$SESSION"
