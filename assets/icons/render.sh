#!/usr/bin/env bash
# Regenerate the PNG icon set under hicolor/ from the SVG sources (needs inkscape).
# mdread.svg is the full design (128 px grid), used at 32 px and up; mdread-small.svg
# is a simplified, pixel-aligned version (16 px grid) for the 16-24 px sizes, where
# the folded corner, text lines and arrow would otherwise blur into noise.
set -euo pipefail
cd "$(dirname "$0")"
render() {
  mkdir -p "hicolor/${2}x${2}/apps"
  inkscape "$1" --export-type=png --export-width="$2" --export-height="$2" \
    --export-filename="hicolor/${2}x${2}/apps/mdread.png"
}
for n in 32 48 64 128 256 512; do render mdread.svg "$n"; done
for n in 16 22 24; do render mdread-small.svg "$n"; done
