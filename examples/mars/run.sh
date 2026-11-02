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

echo "Running Mars example notebooks..."
echo "=================================="
echo ""

# Find and run all .mars files
for file in "$SCRIPT_DIR"/*.mars; do
    if [ -f "$file" ]; then
        filename=$(basename "$file")
        echo "Running: $filename"
        echo "-------------------"
        mars run "$file" --report
        echo ""
    fi
done

echo "=================================="
echo "All examples completed successfully!"
