#!/usr/bin/env bash
# Install mdread for the current user: ~/.local/bin/mdread + desktop entry.
set -euo pipefail
cd "$(dirname "$0")"
if [ ! -x target/release/mdread ]; then
  cargo build --release
fi
mkdir -p "$HOME/.local/bin" "$HOME/.local/share/applications"
install -m 755 target/release/mdread "$HOME/.local/bin/mdread"
install -m 644 mdread.desktop "$HOME/.local/share/applications/mdread.desktop"
update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true
echo "Installed ~/.local/bin/mdread and the desktop entry."
case ":$PATH:" in *":$HOME/.local/bin:"*) ;; *) echo "Note: ~/.local/bin is not on your PATH." ;; esac
echo "To make it the default viewer for Markdown files:"
echo "  xdg-mime default mdread.desktop text/markdown"
