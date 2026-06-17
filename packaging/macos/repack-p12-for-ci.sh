#!/usr/bin/env bash
# Repack a macOS-exported .p12 for GitHub Actions (OpenSSL 3 / modern runners).
# Usage: bash packaging/macos/repack-p12-for-ci.sh input.p12 output.p12
set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "usage: $0 input.p12 output.p12" >&2
  exit 1
fi

IN="$1"
OUT="$2"
DIR="$(mktemp -d /tmp/popsicle-p12-repack.XXXXXX)"
trap 'rm -rf "$DIR"' EXIT

read -rsp "Export password for input .p12: " PASS_IN
echo
read -rsp "Password for output .p12 (CI secret): " PASS_OUT
echo

extract_p12() {
  if openssl pkcs12 -info -in "$IN" -passin "pass:$PASS_IN" -noout 2>/dev/null; then
    openssl pkcs12 -in "$IN" -clcerts -nokeys -out "$DIR/cert.pem" -passin "pass:$PASS_IN"
    openssl pkcs12 -in "$IN" -nocerts -nodes -out "$DIR/key.pem" -passin "pass:$PASS_IN"
  else
    openssl pkcs12 -in "$IN" -clcerts -nokeys -out "$DIR/cert.pem" -passin "pass:$PASS_IN" -legacy
    openssl pkcs12 -in "$IN" -nocerts -nodes -out "$DIR/key.pem" -passin "pass:$PASS_IN" -legacy
  fi
}

extract_p12

if ! grep -q "BEGIN.*PRIVATE KEY" "$DIR/key.pem"; then
  echo "error: input .p12 has no private key; export certificate + private key from Keychain Access" >&2
  exit 1
fi

openssl pkcs12 -export \
  -inkey "$DIR/key.pem" \
  -in "$DIR/cert.pem" \
  -out "$OUT" \
  -passout "pass:$PASS_OUT"

echo "==> repacked: $OUT"
openssl pkcs12 -info -in "$OUT" -passin "pass:$PASS_OUT" -noout | head -5
echo
echo "Update GitHub Secret APPLE_CERTIFICATE:"
echo "  base64 -i \"$OUT\" | tr -d '\\n' | pbcopy"
