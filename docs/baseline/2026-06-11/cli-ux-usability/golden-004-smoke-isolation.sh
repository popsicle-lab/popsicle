#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."

# O-102: the workflow smoke must not mutate the real workspace state.
BEFORE="$(grep -c '^issue' .popsicle/self-host/state.tsv || true)"
cargo test -p cli-ux self_host_workflow_smoke_passes -q >/dev/null
AFTER="$(grep -c '^issue' .popsicle/self-host/state.tsv || true)"
if [ "$BEFORE" != "$AFTER" ]; then
  echo "FAIL: smoke test mutated real workspace (issues $BEFORE -> $AFTER)" >&2
  exit 1
fi
echo "golden-004 ok (issues stayed at $BEFORE)"
