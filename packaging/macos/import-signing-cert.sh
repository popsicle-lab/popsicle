#!/usr/bin/env bash
# Import Apple Developer ID .p12 into a temporary CI keychain.
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "import-signing-cert.sh requires macOS" >&2
  exit 1
fi

: "${APPLE_CERTIFICATE:?APPLE_CERTIFICATE is required}"
: "${APPLE_CERTIFICATE_PASSWORD:?APPLE_CERTIFICATE_PASSWORD is required}"
: "${KEYCHAIN_PASSWORD:?KEYCHAIN_PASSWORD is required}"

CERT_PATH="$(mktemp /tmp/popsicle-cert.XXXXXX.p12)"
trap 'rm -f "$CERT_PATH"' EXIT

echo "$APPLE_CERTIFICATE" | base64 --decode >"$CERT_PATH"

security create-keychain -p "$KEYCHAIN_PASSWORD" build.keychain
security default-keychain -s build.keychain
security unlock-keychain -p "$KEYCHAIN_PASSWORD" build.keychain
security set-keychain-settings -t 3600 -u build.keychain
security import "$CERT_PATH" -k build.keychain \
  -P "$APPLE_CERTIFICATE_PASSWORD" \
  -T /usr/bin/codesign \
  -T /usr/bin/productsign \
  -T /usr/bin/xcrun
security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k "$KEYCHAIN_PASSWORD" build.keychain

echo "==> available signing identities"
security find-identity -v -p codesigning build.keychain
