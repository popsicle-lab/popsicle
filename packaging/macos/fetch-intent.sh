#!/usr/bin/env bash
# Download pinned intent-lang release binary for DMG / local packaging.
# Reads packaging/intent-lang-pin.toml; does not use vender/intent-lang.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PIN="$ROOT/packaging/intent-lang-pin.toml"
OUT_DIR="${INTENT_FETCH_OUT_DIR:-$(mktemp -d /tmp/popsicle-intent-fetch.XXXXXX)}"

if [[ ! -f "$PIN" ]]; then
  echo "fetch-intent.sh: missing $PIN" >&2
  exit 1
fi

read_pin() {
  local section="$1"
  local key="$2"
  local header="[${section}]"
  awk -v header="$header" -v key="$key" '
    $0 == header { in_section=1; next }
    /^\[/ { in_section=0 }
    in_section && $1 == key && $2 == "=" {
      val=$3
      gsub(/^"|"$/, "", val)
      print val
      exit
    }
  ' "$PIN"
}

RELEASE_TAG="$(read_pin pin release_tag)"
REPO="$(read_pin pin repo)"

resolve_arch() {
  if [[ -n "${POPSICLE_DMG_ARCH:-}" ]]; then
    case "${POPSICLE_DMG_ARCH}" in
      aarch64|arm64) echo "aarch64" ;;
      x86_64) echo "x86_64" ;;
      *) echo "fetch-intent.sh: unsupported POPSICLE_DMG_ARCH=${POPSICLE_DMG_ARCH}" >&2; exit 1 ;;
    esac
    return
  fi
  case "$(uname -m)" in
    arm64) echo "aarch64" ;;
    x86_64) echo "x86_64" ;;
    *) echo "fetch-intent.sh: unsupported arch $(uname -m)" >&2; exit 1 ;;
  esac
}

if [[ "${POPSICLE_DMG_SKIP_INTENT:-}" == "1" ]]; then
  echo "fetch-intent.sh: skipped (POPSICLE_DMG_SKIP_INTENT=1)" >&2
  exit 2
fi

ARCH="$(resolve_arch)"
ASSET="$(read_pin "assets.macos" "$ARCH")"
SHA256="$(read_pin "assets.macos.sha256" "$ARCH")"

if [[ -z "$ASSET" ]]; then
  echo "fetch-intent.sh: no macOS release asset for arch=$ARCH in $PIN" >&2
  exit 2
fi

mkdir -p "$OUT_DIR"
DEST="$OUT_DIR/intent"
URL="${REPO}/releases/download/${RELEASE_TAG}/${ASSET}"

echo "==> fetching intent-lang ${RELEASE_TAG} (${ASSET})" >&2
if command -v curl >/dev/null 2>&1; then
  curl -fsSL -o "$DEST" "$URL"
elif command -v wget >/dev/null 2>&1; then
  wget -q -O "$DEST" "$URL"
else
  echo "fetch-intent.sh: need curl or wget" >&2
  exit 1
fi
chmod +x "$DEST"

if [[ -n "$SHA256" ]]; then
  if command -v shasum >/dev/null 2>&1; then
    ACTUAL="$(shasum -a 256 "$DEST" | awk '{print $1}')"
  else
    ACTUAL="$(sha256sum "$DEST" | awk '{print $1}')"
  fi
  if [[ "$ACTUAL" != "$SHA256" ]]; then
    echo "fetch-intent.sh: sha256 mismatch for $ASSET" >&2
    echo "  expected: $SHA256" >&2
    echo "  actual:   $ACTUAL" >&2
    exit 1
  fi
  echo "==> sha256 ok" >&2
fi

if ! "$DEST" --version >/dev/null 2>&1; then
  echo "fetch-intent.sh: downloaded binary failed --version smoke check" >&2
  exit 1
fi

echo "$DEST"
