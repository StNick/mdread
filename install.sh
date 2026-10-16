#!/usr/bin/env bash
# Install mdread for the current user: ~/.local/bin/mdread + desktop entry,
# and register it as the default application for Markdown files.
set -euo pipefail
cd "$(dirname "$0")"
if [ ! -x target/release/mdread ]; then
  cargo build --release
fi
BIN_DIR="$HOME/.local/bin"
APP_DIR="$HOME/.local/share/applications"
BIN="$BIN_DIR/mdread"
mkdir -p "$BIN_DIR" "$APP_DIR"
install -m 755 target/release/mdread "$BIN"
# The desktop entry must carry an absolute Exec path: GNOME's session PATH
# does not include ~/.local/bin, and GLib ignores entries it cannot resolve.
sed "s|@BIN@|$BIN|g" mdread.desktop > "$APP_DIR/mdread.desktop"
chmod 644 "$APP_DIR/mdread.desktop"
update-desktop-database "$APP_DIR" 2>/dev/null || true
echo "Installed $BIN and $APP_DIR/mdread.desktop."
if command -v xdg-mime >/dev/null 2>&1; then
  xdg-mime default mdread.desktop text/markdown text/x-markdown
  echo "Registered mdread as the default application for Markdown files."
else
  echo "To make it the default viewer for Markdown files:"
  echo "  xdg-mime default mdread.desktop text/markdown"
fi
case ":$PATH:" in *":$BIN_DIR:"*) ;; *) echo "Note: $BIN_DIR is not on your PATH (needed only for running 'mdread' from a shell)." ;; esac
