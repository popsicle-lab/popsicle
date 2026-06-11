#!/usr/bin/env bash
# Generate minimal Tauri icon set from a solid-color PNG (no external deps beyond python3/sips).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
ICON_DIR="$ROOT/crates/cli-ux/icons"
mkdir -p "$ICON_DIR"

python3 - "$ICON_DIR/icon.png" <<'PY'
import struct, sys, zlib

def png(w, h, r, g, b):
    def chunk(tag, data):
        return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)
    raw = b"".join(b"\x00" + bytes([r, g, b]) * w for _ in range(h))
    ihdr = struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0)
    return b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", ihdr) + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b"")

path = sys.argv[1]
with open(path, "wb") as f:
    f.write(png(512, 512, 56, 132, 255))
PY

for size in 32 128 256; do
  sips -z "$size" "$size" "$ICON_DIR/icon.png" --out "$ICON_DIR/${size}x${size}.png" >/dev/null
done
cp "$ICON_DIR/256x256.png" "$ICON_DIR/128x128@2x.png"
sips -z 128 128 "$ICON_DIR/icon.png" --out "$ICON_DIR/128x128.png" >/dev/null
sips -z 32 32 "$ICON_DIR/icon.png" --out "$ICON_DIR/32x32.png" >/dev/null

ICONSET="$ICON_DIR/icon.iconset"
rm -rf "$ICONSET"
mkdir -p "$ICONSET"
sips -z 16 16     "$ICON_DIR/icon.png" --out "$ICONSET/icon_16x16.png" >/dev/null
sips -z 32 32     "$ICON_DIR/icon.png" --out "$ICONSET/icon_16x16@2x.png" >/dev/null
sips -z 32 32     "$ICON_DIR/icon.png" --out "$ICONSET/icon_32x32.png" >/dev/null
sips -z 64 64     "$ICON_DIR/icon.png" --out "$ICONSET/icon_32x32@2x.png" >/dev/null
sips -z 128 128   "$ICON_DIR/icon.png" --out "$ICONSET/icon_128x128.png" >/dev/null
sips -z 256 256   "$ICON_DIR/icon.png" --out "$ICONSET/icon_128x128@2x.png" >/dev/null
sips -z 256 256   "$ICON_DIR/icon.png" --out "$ICONSET/icon_256x256.png" >/dev/null
sips -z 512 512   "$ICON_DIR/icon.png" --out "$ICONSET/icon_256x256@2x.png" >/dev/null
sips -z 512 512   "$ICON_DIR/icon.png" --out "$ICONSET/icon_512x512.png" >/dev/null
sips -z 1024 1024 "$ICON_DIR/icon.png" --out "$ICONSET/icon_512x512@2x.png" >/dev/null
iconutil -c icns "$ICONSET" -o "$ICON_DIR/icon.icns"
rm -rf "$ICONSET"
echo "icons ready in $ICON_DIR"
