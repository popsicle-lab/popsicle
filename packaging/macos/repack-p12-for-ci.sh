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

extract_pem_from_p12() {
  local use_legacy="$1"
  local cert_err="$DIR/cert.err" key_err="$DIR/key.err"

  rm -f "$DIR/cert.pem" "$DIR/key.pem" "$cert_err" "$key_err"
  if [[ "$use_legacy" == "true" ]]; then
    openssl pkcs12 -in "$IN" -nokeys -out "$DIR/cert.pem" \
      -passin "pass:$PASS_IN" -legacy 2>"$cert_err" || true
    openssl pkcs12 -in "$IN" -nocerts -nodes -out "$DIR/key.pem" \
      -passin "pass:$PASS_IN" -legacy 2>"$key_err" || true
  else
    openssl pkcs12 -in "$IN" -nokeys -out "$DIR/cert.pem" \
      -passin "pass:$PASS_IN" 2>"$cert_err" || true
    openssl pkcs12 -in "$IN" -nocerts -nodes -out "$DIR/key.pem" \
      -passin "pass:$PASS_IN" 2>"$key_err" || true
  fi

  if grep -q "BEGIN CERTIFICATE" "$DIR/cert.pem" \
    && grep -q "BEGIN.*PRIVATE KEY" "$DIR/key.pem"; then
    return 0
  fi

  [[ -s "$cert_err" ]] && echo "cert extract: $(head -1 "$cert_err")" >&2
  [[ -s "$key_err" ]] && echo "key extract: $(head -1 "$key_err")" >&2
  [[ -f "$DIR/cert.pem" ]] && echo "cert.pem size: $(wc -c <"$DIR/cert.pem" | tr -d ' ') bytes" >&2
  [[ -f "$DIR/key.pem" ]] && echo "key.pem size: $(wc -c <"$DIR/key.pem" | tr -d ' ') bytes" >&2
  return 1
}

if extract_pem_from_p12 true; then
  echo "==> extracted certificate + private key (legacy OpenSSL)"
elif extract_pem_from_p12 false; then
  echo "==> extracted certificate + private key (modern OpenSSL)"
else
  echo "error: could not extract certificate and private key from .p12" >&2
  echo "hint: wrong password, or .p12 was exported without the private key" >&2
  echo "hint: bash packaging/macos/export-p12-from-keychain.sh ~/Documents/popsicle-cert/popsicle-ci.p12" >&2
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
