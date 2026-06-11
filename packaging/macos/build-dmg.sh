#!/usr/bin/env bash
# Build Popsicle.app + popsicle CLI + custom DMG (macOS only, unsigned MVP).
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "build-dmg.sh requires macOS" >&2
  exit 1
fi

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

bash packaging/macos/generate-icons.sh

echo "==> npm build (ui/)"
(cd ui && npm ci && npm run build)

echo "==> cargo build --release --features ui -p cli-ux"
cargo build --release --features ui -p cli-ux

if ! command -v cargo-tauri >/dev/null 2>&1; then
  echo "==> installing tauri-cli"
  cargo install tauri-cli --locked --version "^2.0.0"
fi

echo "==> tauri build (app bundle)"
(cd crates/cli-ux && cargo tauri build --features ui --bundles app)

APP="$ROOT/target/release/bundle/macos/Popsicle.app"
CLI="$ROOT/target/release/popsicle"
if [[ ! -d "$APP" ]]; then
  echo "missing app bundle: $APP" >&2
  exit 1
fi
if [[ ! -x "$CLI" ]]; then
  echo "missing CLI binary: $CLI" >&2
  exit 1
fi

STAGING="$(mktemp -d /tmp/popsicle-dmg-staging.XXXXXX)"
trap 'rm -rf "$STAGING"' EXIT
cp -R "$APP" "$STAGING/Popsicle.app"
cp "$CLI" "$STAGING/popsicle"
chmod +x "$STAGING/popsicle"
cp "packaging/macos/Install CLI.command" "$STAGING/Install CLI.command"
chmod +x "$STAGING/Install CLI.command"
ln -s /Applications "$STAGING/Applications"

VERSION="$(awk -F'"' '/^version = / {print $2; exit}' crates/cli-ux/Cargo.toml)"
ARCH="$(uname -m)"
OUT="$ROOT/target/release/bundle/dmg/Popsicle_${VERSION}_${ARCH}.dmg"
mkdir -p "$(dirname "$OUT")"
rm -f "$OUT"

echo "==> creating DMG: $OUT"
hdiutil create -volname "Popsicle" -srcfolder "$STAGING" -ov -format UDZO "$OUT" >/dev/null

echo "DMG ready: $OUT"
ls -lh "$OUT"
