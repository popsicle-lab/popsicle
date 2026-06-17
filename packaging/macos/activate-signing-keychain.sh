#!/usr/bin/env bash
# Re-unlock the CI signing keychain and verify codesign can see the identity.
set -euo pipefail

KEYCHAIN="$HOME/Library/Keychains/build.keychain-db"
if [[ ! -f "$KEYCHAIN" ]]; then
  echo "==> no CI signing keychain ($KEYCHAIN); signing disabled"
  return 0 2>/dev/null || exit 0
fi

: "${KEYCHAIN_PASSWORD:?KEYCHAIN_PASSWORD is required to unlock the signing keychain}"

security unlock-keychain -p "$KEYCHAIN_PASSWORD" "$KEYCHAIN"
security set-keychain-settings -t 3600 -u "$KEYCHAIN"

# codesign searches the keychain list, not only the default keychain.
OTHER_KEYCHAINS=()
while IFS= read -r line; do
  path="${line#\"}"
  path="${path%\"}"
  [[ "$path" == "$KEYCHAIN" ]] && continue
  OTHER_KEYCHAINS+=("$path")
done < <(security list-keychains -d user)
security list-keychains -d user -s "$KEYCHAIN" "${OTHER_KEYCHAINS[@]}"
security default-keychain -s "$KEYCHAIN"

echo "==> keychain search list"
security list-keychains -d user

echo "==> available signing identities"
security find-identity -v -p codesigning

if [[ -z "${APPLE_SIGNING_IDENTITY:-}" ]]; then
  APPLE_SIGNING_IDENTITY="$(
    security find-identity -v -p codesigning \
      | awk -F'"' '/Developer ID Application/ { print $2; exit }'
  )"
  export APPLE_SIGNING_IDENTITY
fi

if [[ -z "${APPLE_SIGNING_IDENTITY:-}" ]]; then
  echo "error: no Developer ID Application signing identity in keychain" >&2
  exit 1
fi

if ! security find-identity -v -p codesigning | grep -Fq "\"${APPLE_SIGNING_IDENTITY}\""; then
  echo "error: signing identity not found: ${APPLE_SIGNING_IDENTITY}" >&2
  exit 1
fi

echo "==> using signing identity: ${APPLE_SIGNING_IDENTITY}"
export APPLE_SIGNING_IDENTITY
