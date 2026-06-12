#!/usr/bin/env bash
# Install pinned intent-lang release to ~/.local/bin/intent (macOS).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
DEST_DIR="${HOME}/.local/bin"
DEST="${DEST_DIR}/intent"

INTENT_BIN="$(bash "$ROOT/packaging/macos/fetch-intent.sh")"
mkdir -p "$DEST_DIR"
cp "$INTENT_BIN" "$DEST"
chmod +x "$DEST"

VER="$("$DEST" --version 2>/dev/null || true)"
echo "Installed intent to $DEST (${VER:-unknown})"
if ! echo ":$PATH:" | grep -q ":${DEST_DIR}:"; then
  echo "Add to PATH: export PATH=\"\$HOME/.local/bin:\$PATH\""
fi
