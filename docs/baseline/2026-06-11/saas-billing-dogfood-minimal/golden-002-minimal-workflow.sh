#!/usr/bin/env bash
# PROJ-8 dogfood minimal: doctor → issue → start → debate stage → intent-validate
# Uses popsicle-new ./target/debug/popsicle only (not ../target/debug/popsicle).
set -euo pipefail
cd "$(dirname "$0")/../../../.."
cargo build -p cli-ux -q
POP="./target/debug/popsicle"

field() {
  local key="$1"
  awk -F': ' -v k="$key" '$1 == k { print $2; exit }'
}

echo "==> doctor"
$POP doctor --format json | grep -q '"current_workspace_binary_match":"true"'

echo "==> issue create"
CREATED="$($POP issue create \
  --type product \
  --title "SaaS billing dogfood minimal baseline" \
  --spec saas-billing-module \
  --pipeline greenfield-product-spec \
  --description "Baseline: debate stage only via self-host TSV backend")"
KEY="$(echo "$CREATED" | field key)"

echo "==> issue start"
STARTED="$($POP issue start "$KEY" --spec saas-billing-module --pipeline greenfield-product-spec)"
RUN_ID="$(echo "$STARTED" | field run_id)"
[[ -n "$RUN_ID" ]]

echo "==> pipeline next + doc + stage complete debate"
$POP pipeline next --run "$RUN_ID" | grep -q -- '--confirm'
$POP doc create product-debate \
  --title "SaaS billing product debate (baseline)" \
  --run "$RUN_ID"
$POP pipeline stage complete debate --run "$RUN_ID" --confirm

echo "==> pipeline status"
STATUS="$($POP pipeline status --run "$RUN_ID")"
echo "$STATUS" | field run_status | grep -q in_progress
echo "$STATUS" | field current_stage | grep -q prd
echo "$STATUS" | field stage_0_status | grep -q completed

echo "==> intent-validate"
$POP tool run intent-validate path=products | grep -q 'exit_code: 0'

echo "Minimal SaaS billing dogfood passed (issue=$KEY run=$RUN_ID)"
