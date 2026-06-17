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

KEYCHAIN="$HOME/Library/Keychains/build.keychain-db"
CERT_PATH="$(mktemp /tmp/popsicle-cert.XXXXXX.p12)"
trap 'rm -f "$CERT_PATH"' EXIT

# GitHub Secret paste may include stray whitespace/newlines.
echo "$APPLE_CERTIFICATE" | tr -d ' \n\r\t' | base64 --decode >"$CERT_PATH"

security delete-keychain "$KEYCHAIN" 2>/dev/null || true
security create-keychain -p "$KEYCHAIN_PASSWORD" build.keychain
security unlock-keychain -p "$KEYCHAIN_PASSWORD" "$KEYCHAIN"
security set-keychain-settings -t 3600 -u "$KEYCHAIN"
security import "$CERT_PATH" -k "$KEYCHAIN" \
  -P "$APPLE_CERTIFICATE_PASSWORD" \
  -T /usr/bin/codesign \
  -T /usr/bin/productsign \
  -T /usr/bin/xcrun
security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k "$KEYCHAIN_PASSWORD" "$KEYCHAIN"

OTHER_KEYCHAINS=()
while IFS= read -r line; do
  path="${line#\"}"
  path="${path%\"}"
  [[ "$path" == "$KEYCHAIN" ]] && continue
  OTHER_KEYCHAINS+=("$path")
done < <(security list-keychains -d user)
security list-keychains -d user -s "$KEYCHAIN" "${OTHER_KEYCHAINS[@]}"
security default-keychain -s "$KEYCHAIN"

IDENTITY="$(
  security find-identity -v -p codesigning "$KEYCHAIN" \
    | awk -F'"' '/Developer ID Application/ { print $2; exit }'
)"
if [[ -z "$IDENTITY" ]]; then
  echo "error: no Developer ID Application identity after .p12 import" >&2
  security find-identity -v -p codesigning "$KEYCHAIN" >&2 || true
  exit 1
fi

echo "==> imported signing identity: $IDENTITY"
export APPLE_SIGNING_IDENTITY="$IDENTITY"
if [[ -n "${GITHUB_ENV:-}" ]]; then
  echo "APPLE_SIGNING_IDENTITY=$IDENTITY" >>"$GITHUB_ENV"
fi
