#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
cargo build -p cli-ux -q
BIN="$PWD/target/debug/popsicle"

ROOT="$(mktemp -d /tmp/popsicle-weekly-health.XXXXXX)"
trap 'rm -rf "$ROOT"' EXIT

cd "$ROOT"
"$BIN" init --format json >/dev/null

test -f .popsicle/pipelines/weekly-health-check.pipeline.yaml
grep -q 'health-sync' .popsicle/pipelines/weekly-health-check.pipeline.yaml
grep -q 'living-doc-author' .popsicle/pipelines/weekly-health-check.pipeline.yaml
! grep -q 'requires_approval' .popsicle/pipelines/weekly-health-check.pipeline.yaml

echo "golden-001 ok (weekly-health-check pipeline installed on init)"
