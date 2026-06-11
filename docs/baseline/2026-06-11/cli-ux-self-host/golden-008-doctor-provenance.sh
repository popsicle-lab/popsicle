#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
cargo build -p cli-ux -q
OUT="$(./target/debug/popsicle doctor --format json)"
echo "$OUT" | grep -q '"current_workspace_binary_match":"true"'
# Amended by ADR-013 (PROJ-25): storage_backend is now dynamic — tsv for
# legacy workspaces, sqlite after Phase 2 migration. Original ADR-010
# assertion pinned the Phase 1 tsv string.
echo "$OUT" | grep -Eq '"storage_backend":"(sqlite \(.popsicle/self-host/state.db\)|tsv \(.popsicle/self-host/state.tsv\))"'
echo "$OUT" | grep -q '"phase_2_issue":"PROJ-11"'
