#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."

# O-102: the workflow smoke must not mutate the real workspace state.
# Amended by ADR-013 (PROJ-25): count via `issue list` instead of grepping
# state.tsv, so the check is backend-agnostic (tsv or sqlite).
cargo build -p cli-ux -q
COUNT() { ./target/debug/popsicle issue list --format json | python3 -c "import json,sys; print(json.load(sys.stdin)['count'])"; }
BEFORE="$(COUNT)"
cargo test -p cli-ux workspace_workflow_smoke_passes -q >/dev/null
AFTER="$(COUNT)"
if [ "$BEFORE" != "$AFTER" ]; then
  echo "FAIL: smoke test mutated real workspace (issues $BEFORE -> $AFTER)" >&2
  exit 1
fi
echo "golden-004 ok (issues stayed at $BEFORE)"
