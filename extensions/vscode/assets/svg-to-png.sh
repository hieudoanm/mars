#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"
mkdir -p images
for svg in svg/*.svg; do
  name="$(basename "$svg" .svg)"
  png="images/$name.png"
  rsvg-convert --keep-aspect-ratio "$svg" -o "$png"
  echo "Created $png"
done
