#!/bin/bash

# Run all Mars example notebooks
# This script executes all .mars files in the current directory

set -e

# Source cargo environment to ensure mars CLI is available
if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
fi

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Change to the script directory
cd "$SCRIPT_DIR"

echo "Running Mars example notebooks..."
echo "=================================="
echo ""

# Find and run all .mars files in sorted order
for file in $(ls -1 *.mars 2>/dev/null | sort); do
    if [ -f "$file" ]; then
        filename=$(basename "$file")
        echo "Running: $filename"
        echo "-------------------"
        if command -v mars >/dev/null 2>&1; then
            mars run "$file" --report
        else
            echo "Error: mars CLI not found. Please build and install it first."
            echo "  cd /path/to/mars && cargo install --path crates/mars-cli"
            exit 1
        fi
        echo ""
    fi
done

echo "=================================="
echo "All examples completed successfully!"
