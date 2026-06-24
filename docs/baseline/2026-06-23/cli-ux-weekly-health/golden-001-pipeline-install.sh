#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
cargo build -p cli-ux -q
BIN="$PWD/target/debug/popsicle"

ROOT="$(mktemp -d /tmp/popsicle-weekly-health.XXXXXX)"
trap 'rm -rf "$ROOT"' EXIT

cd "$ROOT"
"$BIN" init --format json >/dev/null

test -f .popsicle/pipelines/doc-sync-weekly.pipeline.yaml
grep -q 'health-sync' .popsicle/pipelines/doc-sync-weekly.pipeline.yaml
grep -q 'living-doc-author' .popsicle/pipelines/doc-sync-weekly.pipeline.yaml
! grep -q 'requires_approval' .popsicle/pipelines/doc-sync-weekly.pipeline.yaml

echo "golden-001 ok (doc-sync-weekly pipeline installed on init)"
