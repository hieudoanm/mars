#!/bin/bash

# Run every Mars example notebook in this directory tree, then export each
# result to Markdown, PDF, and HTML formats.
# Each example lives in its own folder alongside its exports.

set -e

# Source cargo environment to ensure mars CLI is available
if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
fi

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Change to the script directory
cd "$SCRIPT_DIR"

if ! command -v mars >/dev/null 2>&1; then
    echo "Error: mars CLI not found. Please build and install it first." >&2
    echo "  cd /path/to/mars && ./scripts/install.sh --cli" >&2
    exit 1
fi

echo "Running Mars example notebooks..."
echo "=================================="
echo ""

# Find and run all .mars files recursively in sorted order.
while IFS= read -r file; do
    relative_path="${file#"$SCRIPT_DIR"/}"
    echo "Running: $relative_path"
    echo "-------------------"
    mars run "$file" --report

    base="${file%.mars}"
    echo "Exporting: $(basename "$base").md / .pdf / .html"
    mars export "$file" md "${base}.md"
    mars export "$file" pdf "${base}.pdf"
    mars export "$file" html "${base}.html"
    echo ""
done < <(find "$SCRIPT_DIR" -type f -name '*.mars' -print | LC_ALL=C sort)

echo "=================================="
echo "All examples completed successfully!"
