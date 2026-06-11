#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
BIN=./target/debug/popsicle
[ -x "$BIN" ] || cargo build -p cli-ux -q

set +e
ERR="$($BIN memory list --format json 2>&1 >/dev/null)"
CODE=$?
set -e
[ "$CODE" -eq 2 ] || { echo "FAIL: expected exit 2, got $CODE" >&2; exit 1; }
echo "$ERR" | grep -q '"status":"error"'
echo "$ERR" | grep -q '"category":"deferred"'
echo "$ERR" | grep -q '"next":'
echo "golden-004 ok"
