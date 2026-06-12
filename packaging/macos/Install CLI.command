#!/bin/bash
set -euo pipefail

DMG_DIR="$(cd "$(dirname "$0")" && pwd)"
CLI_SRC="$DMG_DIR/popsicle"
INTENT_SRC="$DMG_DIR/intent"
DEST_DIR="${HOME}/.local/bin"
GLOBAL_DIR="${HOME}/.popsicle"
ZSHRC="${HOME}/.zshrc"

if [[ ! -f "$CLI_SRC" ]]; then
  osascript -e 'display alert "Popsicle" message "popsicle binary not found next to this installer."'
  exit 1
fi

mkdir -p "$DEST_DIR" "$GLOBAL_DIR"
cp "$CLI_SRC" "$DEST_DIR/popsicle"
chmod +x "$DEST_DIR/popsicle"

if [[ -f "$INTENT_SRC" ]]; then
  cp "$INTENT_SRC" "$DEST_DIR/intent"
  chmod +x "$DEST_DIR/intent"
fi

PATH_LINE='export PATH="$HOME/.local/bin:$PATH"'
if [[ -f "$ZSHRC" ]] && ! grep -qF '.local/bin' "$ZSHRC"; then
  printf '\n# Popsicle toolchain (DMG install)\n%s\n' "$PATH_LINE" >>"$ZSHRC"
elif [[ ! -f "$ZSHRC" ]]; then
  printf '# ~/.zshrc\n%s\n' "$PATH_LINE" >"$ZSHRC"
fi

echo "Installed popsicle to $DEST_DIR/popsicle"
if [[ -f "$DEST_DIR/intent" ]]; then
  echo "Installed intent to $DEST_DIR/intent"
fi
echo "Restart your terminal or run: source ~/.zshrc"
