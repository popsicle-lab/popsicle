#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
BIN=./target/debug/popsicle
[ -x "$BIN" ] || cargo build -p cli-ux -q

# Must resolve the workspace's own intent-coder tool definition (not a sibling
# checkout outside the repo) and pass against all product intents.
$BIN tool run intent-validate path=products | grep -q "exit_code: 0"
echo "golden-005 ok"
