#!/usr/bin/env bash
# Build Popsicle.app + popsicle CLI + custom DMG (macOS).
# When APPLE_SIGNING_IDENTITY is set, signs staging artifacts and notarizes the DMG
# if Apple notarization credentials are also present.
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "build-dmg.sh requires macOS" >&2
  exit 1
fi

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

TARGET="${CARGO_BUILD_TARGET:-}"

if [[ -f "$HOME/Library/Keychains/build.keychain-db" ]]; then
  # shellcheck source=/dev/null
  source "$(dirname "$0")/activate-signing-keychain.sh"
fi

bash packaging/macos/generate-icons.sh

echo "==> npm build (ui/)"
(cd ui && npm ci && npm run build)

echo "==> cargo build --release -p cli-ux (default features include ui)"
if [[ -n "$TARGET" ]]; then
  cargo build --release -p cli-ux --target "$TARGET"
else
  cargo build --release -p cli-ux
fi

tauri_build() {
  if [[ -n "$TARGET" ]]; then
    npx --yes @tauri-apps/cli@2 build --features ui --bundles app --target "$TARGET"
  else
    npx --yes @tauri-apps/cli@2 build --features ui --bundles app
  fi
}

echo "==> tauri build (app bundle; ui feature from tauri.conf.json + crate defaults)"
# Sign during bundle; defer notarization until after intent injection + DMG packaging.
if [[ -n "${APPLE_SIGNING_IDENTITY:-}" ]]; then
  echo "==> code signing enabled ($APPLE_SIGNING_IDENTITY)"
  unset APPLE_ID APPLE_PASSWORD APPLE_API_KEY APPLE_API_ISSUER APPLE_API_KEY_PATH
  (cd crates/cli-ux && tauri_build)
else
  echo "==> code signing disabled (set APPLE_SIGNING_IDENTITY to enable)"
  (cd crates/cli-ux && tauri_build)
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

ENTITLEMENTS="$ROOT/crates/cli-ux/Entitlements.plist"

sign_staging_if_configured() {
  local identity="${APPLE_SIGNING_IDENTITY:-}"
  [[ -n "$identity" ]] || return 0

  echo "==> signing DMG staging artifacts"
  codesign --force --options runtime --sign "$identity" "$STAGING/popsicle"
  if [[ -x "$STAGING/intent" ]]; then
    codesign --force --options runtime --sign "$identity" "$STAGING/intent"
  fi
  codesign --force --options runtime --entitlements "$ENTITLEMENTS" \
    --sign "$identity" --deep "$STAGING/Popsicle.app"
  codesign --verify --deep --strict "$STAGING/Popsicle.app"
}

notarize_dmg_if_configured() {
  local dmg="$1"
  if [[ -n "${APPLE_API_KEY:-}" && -n "${APPLE_API_ISSUER:-}" && -f "${APPLE_API_KEY_PATH:-}" ]]; then
    echo "==> notarizing DMG (App Store Connect API key)"
    xcrun notarytool submit "$dmg" \
      --key "$APPLE_API_KEY_PATH" \
      --key-id "$APPLE_API_KEY" \
      --issuer "$APPLE_API_ISSUER" \
      --wait
  elif [[ -n "${APPLE_ID:-}" && -n "${APPLE_PASSWORD:-}" && -n "${APPLE_TEAM_ID:-}" ]]; then
    echo "==> notarizing DMG (Apple ID)"
    xcrun notarytool submit "$dmg" \
      --apple-id "$APPLE_ID" \
      --team-id "$APPLE_TEAM_ID" \
      --password "$APPLE_PASSWORD" \
      --wait
  else
    echo "==> skipping DMG notarization (no Apple notarization credentials)"
    return 0
  fi
  xcrun stapler staple "$dmg"
  xcrun stapler validate "$dmg"
}

sign_staging_if_configured

echo "==> creating DMG: $OUT"
hdiutil create -volname "Popsicle" -srcfolder "$STAGING" -ov -format UDZO "$OUT" >/dev/null

notarize_dmg_if_configured "$OUT"

echo "DMG ready: $OUT"
ls -lh "$OUT"
if [[ -n "${APPLE_SIGNING_IDENTITY:-}" ]]; then
  spctl -a -vv "$OUT" 2>&1 || true
fi
echo ""
echo "Note: make build-dmg does NOT install into your shell PATH."
echo "  • Open Popsicle.app from Applications once (double-click), or"
echo "  • Mount the DMG and run Install CLI.command"
echo "  • Or: make install-intent"
