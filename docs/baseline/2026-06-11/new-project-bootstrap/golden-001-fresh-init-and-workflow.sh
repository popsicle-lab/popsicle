#!/usr/bin/env bash
# New-project bootstrap golden: popsicle init in an empty directory installs
# bundled pipelines, numbers issues from PROJ-1, and runs the IDD main path
# with an installed-style binary (dev_workspace=false, doctor status ok).
set -euo pipefail
cd "$(dirname "$0")/../../../.."
cargo build -p cli-ux -q
BIN="$PWD/target/debug/popsicle"

WORK="$(mktemp -d /tmp/popsicle-bootstrap-golden.XXXXXX)"
trap 'rm -rf "$WORK"' EXIT
cd "$WORK"

field() {
  local key="$1"
  awk -F': ' -v k="$key" '$1 == k { print $2; exit }'
}

echo "==> help works without a workspace"
"$BIN" help | grep -q '^status: ok'

echo "==> init bootstraps a fresh workspace"
"$BIN" init | grep -q 'workspace_ready: true'
test -f .popsicle/state.db
test -f .popsicle/pipelines/product-greenfield-spec.pipeline.yaml
test -f .popsicle/pipelines/migration-bootstrap.pipeline.yaml

echo "==> init is idempotent"
"$BIN" init | grep -q 'workspace_ready: true'

echo "==> doctor is ok for installed binary outside dev workspace"
DOCTOR="$("$BIN" doctor --format json)"
echo "$DOCTOR" | grep -q '"status":"ok"'
echo "$DOCTOR" | grep -q '"dev_workspace":"false"'

echo "==> issues number from PROJ-1"
CREATED="$("$BIN" issue create \
  --type technical \
  --title "bootstrap golden issue" \
  --spec bootstrap-slice-1 \
  --pipeline migration-bootstrap)"
KEY="$(echo "$CREATED" | field key)"
[[ "$KEY" == "PROJ-1" ]]

echo "==> bundled pipeline drives the IDD main path"
STARTED="$("$BIN" issue start "$KEY")"
RUN_ID="$(echo "$STARTED" | field run_id)"
"$BIN" pipeline next --run "$RUN_ID" | grep -q -- '--confirm'
"$BIN" doc create project-init --title "bootstrap plan" --run "$RUN_ID" | grep -q 'artifact_file_exists: true'
"$BIN" pipeline stage complete init --run "$RUN_ID" --confirm
"$BIN" pipeline status --run "$RUN_ID" | field current_stage | grep -q facts

echo "New-project bootstrap golden passed (workdir=$WORK)"
