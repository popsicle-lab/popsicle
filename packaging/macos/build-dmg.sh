#!/usr/bin/env bash
# Build Popsicle.app + popsicle CLI + custom DMG (macOS only, unsigned MVP).
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "build-dmg.sh requires macOS" >&2
  exit 1
fi

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

TARGET="${CARGO_BUILD_TARGET:-}"

bash packaging/macos/generate-icons.sh

echo "==> npm build (ui/)"
(cd ui && npm ci && npm run build)

echo "==> cargo build --release --features ui -p cli-ux"
if [[ -n "$TARGET" ]]; then
  cargo build --release --features ui -p cli-ux --target "$TARGET"
else
  cargo build --release --features ui -p cli-ux
fi

if ! command -v cargo-tauri >/dev/null 2>&1; then
  echo "==> installing tauri-cli"
  cargo install tauri-cli --locked --version "^2.0.0"
fi

echo "==> tauri build (app bundle)"
if [[ -n "$TARGET" ]]; then
  (cd crates/cli-ux && cargo tauri build --features ui --bundles app --target "$TARGET")
else
  (cd crates/cli-ux && cargo tauri build --features ui --bundles app)
fi

if [[ -n "$TARGET" ]]; then
  APP="$ROOT/target/$TARGET/release/bundle/macos/Popsicle.app"
  CLI="$ROOT/target/$TARGET/release/popsicle"
else
  APP="$ROOT/target/release/bundle/macos/Popsicle.app"
  CLI="$ROOT/target/release/popsicle"
fi

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

echo "==> fetch intent-lang (packaging/intent-lang-pin.toml)"
INTENT_FETCH_OUT_DIR="$(mktemp -d /tmp/popsicle-intent-staging.XXXXXX)"
export INTENT_FETCH_OUT_DIR
if INTENT_BIN="$(bash packaging/macos/fetch-intent.sh)"; then
  cp "$INTENT_BIN" "$STAGING/intent"
  chmod +x "$STAGING/intent"
  # Also patch the built .app under target/ (not only DMG staging).
  mkdir -p "$APP/Contents/Resources"
  cp "$INTENT_BIN" "$APP/Contents/Resources/intent"
  chmod +x "$APP/Contents/Resources/intent"
  mkdir -p "$STAGING/Popsicle.app/Contents/Resources"
  cp "$INTENT_BIN" "$STAGING/Popsicle.app/Contents/Resources/intent"
  chmod +x "$STAGING/Popsicle.app/Contents/Resources/intent"
  INTENT_VER="$("$STAGING/intent" --version 2>/dev/null || true)"
  echo "==> bundled intent-lang ${INTENT_VER:-unknown}"
else
  fetch_status=$?
  if [[ $fetch_status -eq 2 ]]; then
    echo "==> intent-lang not bundled (fetch skipped or no asset for arch)" >&2
  else
    exit "$fetch_status"
  fi
fi
rm -rf "$INTENT_FETCH_OUT_DIR"

cp "packaging/macos/Install CLI.command" "$STAGING/Install CLI.command"
chmod +x "$STAGING/Install CLI.command"
ln -s /Applications "$STAGING/Applications"

VERSION="$(awk -F'"' '/^version = / {print $2; exit}' crates/cli-ux/Cargo.toml)"
ARCH="${POPSICLE_DMG_ARCH:-$(uname -m)}"
OUT_DIR="${POPSICLE_DMG_OUT_DIR:-$ROOT/target/release/bundle/dmg}"
OUT="$OUT_DIR/Popsicle_${VERSION}_${ARCH}.dmg"
mkdir -p "$OUT_DIR"
rm -f "$OUT"

echo "==> creating DMG: $OUT"
hdiutil create -volname "Popsicle" -srcfolder "$STAGING" -ov -format UDZO "$OUT" >/dev/null

echo "DMG ready: $OUT"
ls -lh "$OUT"
echo ""
echo "Note: make build-dmg does NOT install into your shell PATH."
echo "  • Open Popsicle.app from Applications once (double-click), or"
echo "  • Mount the DMG and run Install CLI.command"
echo "  • Or: make install-intent"
