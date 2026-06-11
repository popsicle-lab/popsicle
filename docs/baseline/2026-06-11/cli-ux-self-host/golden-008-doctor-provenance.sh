#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
cargo build -p cli-ux -q
OUT="$(./target/debug/popsicle doctor --format json)"
echo "$OUT" | grep -q '"current_workspace_binary_match":"true"'
echo "$OUT" | grep -q '"storage_backend":"tsv (.popsicle/self-host/state.tsv)"'
echo "$OUT" | grep -q '"phase_2_issue":"PROJ-11"'
