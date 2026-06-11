#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
cargo build -p cli-ux -q
OUT="$(./target/debug/popsicle doctor --format json)"
echo "$OUT" | grep -q '"current_workspace_binary_match":"true"'
echo "$OUT" | grep -q '"used_parent_binary":"false"'
echo "$OUT" | grep -q '"workspace_root"'
! echo "$OUT" | grep -q '"json":'
