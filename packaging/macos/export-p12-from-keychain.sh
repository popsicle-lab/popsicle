#!/usr/bin/env bash
# Export Developer ID identity (cert + private key) from the login keychain to .p12.
# Usage: bash packaging/macos/export-p12-from-keychain.sh output.p12
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 output.p12" >&2
  exit 1
fi

OUT="$1"
LOGIN_KC="$HOME/Library/Keychains/login.keychain-db"

echo "==> signing identities in login keychain"
LINE="$(security find-identity -v -p codesigning "$LOGIN_KC" | grep "Developer ID Application" | head -1 || true)"
if [[ -z "$LINE" ]]; then
  echo "error: no Developer ID Application identity in login keychain" >&2
  echo "hint: security find-identity -v -p codesigning" >&2
  exit 1
fi
HASH="$(sed -E 's/^[[:space:]]*[0-9]+\) ([0-9A-F]+) .*/\1/' <<<"$LINE")"
IDENTITY="$(sed -E 's/^[[:space:]]*[0-9]+\) [0-9A-F]+ "(.+)"$/\1/' <<<"$LINE")"

echo "==> exporting: $IDENTITY"
read -rsp "Password for output .p12 (use for APPLE_CERTIFICATE_PASSWORD): " PASS
echo

security export -k "$LOGIN_KC" -t identities -f pkcs12 -P "$PASS" -o "$OUT" "$HASH"

echo "==> exported $(wc -c <"$OUT" | tr -d ' ') bytes to $OUT"
openssl pkcs12 -info -in "$OUT" -passin "pass:$PASS" -noout 2>/dev/null \
  || openssl pkcs12 -info -in "$OUT" -passin "pass:$PASS" -noout -legacy

echo
echo "Optional repack for CI (recommended):"
echo "  bash packaging/macos/repack-p12-for-ci.sh \"$OUT\" \"${OUT%.p12}-ci.p12\""
echo
echo "GitHub Secret APPLE_CERTIFICATE:"
echo "  base64 -i \"${OUT%.p12}-ci.p12\" 2>/dev/null | tr -d '\\n' | pbcopy"
echo "  # or use $OUT directly if repack skipped"
