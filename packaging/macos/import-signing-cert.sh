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
WORKDIR="$(mktemp -d /tmp/popsicle-cert-work.XXXXXX)"
trap 'rm -f "$CERT_PATH"; rm -rf "$WORKDIR"' EXIT

decode_certificate() {
  printf '%s' "$APPLE_CERTIFICATE" | tr -d ' \n\r\t' | base64 --decode -o "$CERT_PATH"
  local size
  size="$(wc -c <"$CERT_PATH" | tr -d ' ')"
  if [[ "$size" -lt 100 ]]; then
    echo "error: decoded .p12 is only ${size} bytes; APPLE_CERTIFICATE base64 is likely wrong" >&2
    exit 1
  fi
  echo "==> decoded .p12 (${size} bytes)"
}

p12_openssl_args() {
  if openssl pkcs12 -info -in "$CERT_PATH" -passin "pass:$APPLE_CERTIFICATE_PASSWORD" -noout 2>/dev/null; then
    echo ""
    return 0
  fi
  if openssl pkcs12 -info -in "$CERT_PATH" -passin "pass:$APPLE_CERTIFICATE_PASSWORD" -noout -legacy 2>/dev/null; then
    echo "-legacy"
    return 0
  fi
  echo "error: cannot decrypt .p12; check APPLE_CERTIFICATE_PASSWORD and base64 secret" >&2
  exit 1
}

repack_for_modern_openssl() {
  local legacy_flag="$1"
  local repacked="$WORKDIR/repacked.p12"
  echo "==> repacking .p12 with modern encryption for CI import"

  openssl pkcs12 -in "$CERT_PATH" -clcerts -nokeys -out "$WORKDIR/cert.pem" \
    -passin "pass:$APPLE_CERTIFICATE_PASSWORD" $legacy_flag
  openssl pkcs12 -in "$CERT_PATH" -nocerts -nodes -out "$WORKDIR/key.pem" \
    -passin "pass:$APPLE_CERTIFICATE_PASSWORD" $legacy_flag

  if ! grep -q "BEGIN.*PRIVATE KEY" "$WORKDIR/key.pem"; then
    echo "error: .p12 contains no private key" >&2
    echo "hint: in Keychain Access export Developer ID Application with its private key (Export 2 items)" >&2
    exit 1
  fi

  openssl pkcs12 -export \
    -inkey "$WORKDIR/key.pem" \
    -in "$WORKDIR/cert.pem" \
    -out "$repacked" \
    -passout "pass:$APPLE_CERTIFICATE_PASSWORD"
  mv "$repacked" "$CERT_PATH"
}

prepare_certificate() {
  decode_certificate
  echo "==> verifying .p12 password"
  local legacy_flag
  legacy_flag="$(p12_openssl_args)"
  if [[ "$legacy_flag" == "-legacy" ]]; then
    repack_for_modern_openssl "$legacy_flag"
  fi
}

import_into_keychain() {
  security delete-keychain "$KEYCHAIN" 2>/dev/null || true
  security create-keychain -p "$KEYCHAIN_PASSWORD" build.keychain
  security unlock-keychain -p "$KEYCHAIN_PASSWORD" "$KEYCHAIN"
  security set-keychain-settings -t 3600 -u "$KEYCHAIN"

  security import "$CERT_PATH" -P "$APPLE_CERTIFICATE_PASSWORD" \
    -A -t cert -f pkcs12 -k "$KEYCHAIN" \
    -T /usr/bin/codesign \
    -T /usr/bin/security \
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
}

resolve_identity() {
  local identity
  identity="$(
    security find-identity -v -p codesigning "$KEYCHAIN" \
      | awk -F'"' '/Developer ID Application/ { print $2; exit }'
  )"
  if [[ -z "$identity" ]]; then
    echo "error: no Developer ID Application identity after .p12 import" >&2
    echo "==> find-identity output:" >&2
    security find-identity -v -p codesigning "$KEYCHAIN" >&2 || true
    echo "==> keychain dump (cert subjects):" >&2
    security find-certificate -a -p "$KEYCHAIN" 2>/dev/null \
      | openssl x509 -noout -subject 2>/dev/null || true
    echo "hint: run locally: bash packaging/macos/repack-p12-for-ci.sh your.p12 ci.p12" >&2
    echo "hint: then refresh APPLE_CERTIFICATE base64 + APPLE_CERTIFICATE_PASSWORD" >&2
    exit 1
  fi

  echo "==> imported signing identity: $identity"
  export APPLE_SIGNING_IDENTITY="$identity"
  if [[ -n "${GITHUB_ENV:-}" ]]; then
    echo "APPLE_SIGNING_IDENTITY=$identity" >>"$GITHUB_ENV"
  fi
}

prepare_certificate
import_into_keychain
resolve_identity
