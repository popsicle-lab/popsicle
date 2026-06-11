#!/usr/bin/env bash
# Generate minimal Tauri icon set as RGBA PNGs (python3 + iconutil only).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
ICON_DIR="$ROOT/crates/cli-ux/icons"
mkdir -p "$ICON_DIR"

python3 - "$ICON_DIR" <<'PY'
import os
import struct
import sys
import zlib

R, G, B, A = 56, 132, 255, 255


def png_rgba(w: int, h: int, r: int = R, g: int = G, b: int = B, a: int = A) -> bytes:
    def chunk(tag: bytes, data: bytes) -> bytes:
        return (
            struct.pack(">I", len(data))
            + tag
            + data
            + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)
        )

    raw = b"".join(b"\x00" + bytes([r, g, b, a]) * w for _ in range(h))
    ihdr = struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0)  # color type 6 = RGBA
    return (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", ihdr)
        + chunk(b"IDAT", zlib.compress(raw, 9))
        + chunk(b"IEND", b"")
    )


def write_png(path: str, size: int) -> None:
    with open(path, "wb") as f:
        f.write(png_rgba(size, size))


icon_dir = sys.argv[1]
for name, size in (
    ("32x32.png", 32),
    ("128x128.png", 128),
    ("128x128@2x.png", 256),
    ("256x256.png", 256),
    ("icon.png", 512),
):
    write_png(os.path.join(icon_dir, name), size)

iconset = os.path.join(icon_dir, "icon.iconset")
os.makedirs(iconset, exist_ok=True)
for name, size in (
    ("icon_16x16.png", 16),
    ("icon_16x16@2x.png", 32),
    ("icon_32x32.png", 32),
    ("icon_32x32@2x.png", 64),
    ("icon_128x128.png", 128),
    ("icon_128x128@2x.png", 256),
    ("icon_256x256.png", 256),
    ("icon_256x256@2x.png", 512),
    ("icon_512x512.png", 512),
    ("icon_512x512@2x.png", 1024),
):
    write_png(os.path.join(iconset, name), size)
PY

ICONSET="$ICON_DIR/icon.iconset"
iconutil -c icns "$ICONSET" -o "$ICON_DIR/icon.icns"
rm -rf "$ICONSET"
echo "icons ready in $ICON_DIR"
