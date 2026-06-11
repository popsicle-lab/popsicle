#!/usr/bin/env bash
# Full greenfield-product-spec (8 stages) via ./target/debug/popsicle self-host backend.
set -euo pipefail
cd "$(dirname "$0")/../../../.."
cargo build -p cli-ux -q
POP="./target/debug/popsicle"

field() {
  local key="$1"
  awk -F': ' -v k="$key" '$1 == k { print $2; exit }'
}

advance_gated() {
  local stage="$1" skill="$2" title="$3"
  $POP pipeline next --run "$RUN_ID" | grep -q -- '--confirm'
  $POP doc create "$skill" --title "$title" --run "$RUN_ID"
  $POP pipeline stage complete "$stage" --run "$RUN_ID" --confirm
}

advance_open() {
  local stage="$1" skill="$2" title="$3"
  $POP pipeline next --run "$RUN_ID"
  $POP doc create "$skill" --title "$title" --run "$RUN_ID"
  $POP pipeline stage complete "$stage" --run "$RUN_ID"
}

echo "==> issue create + start"
CREATED="$($POP issue create \
  --type product \
  --title "SaaS billing full greenfield baseline" \
  --spec saas-billing-module \
  --pipeline greenfield-product-spec \
  --description "Baseline: all 8 greenfield-product-spec stages")"
KEY="$(echo "$CREATED" | field key)"
STARTED="$($POP issue start "$KEY" --spec saas-billing-module --pipeline greenfield-product-spec)"
RUN_ID="$(echo "$STARTED" | field run_id)"

advance_gated debate product-debate "debate baseline"
advance_gated prd prd-writer "prd baseline"
advance_gated arch-debate arch-debate "arch-debate baseline"
advance_gated rfc rfc-writer "rfc baseline"
advance_gated adr adr-writer "adr baseline"
advance_open intent-spec intent-spec-writer "intent-spec baseline"
advance_open intent-check intent-consistency-check "intent-check baseline"
advance_gated living-docs living-doc-author "living-docs baseline"

STATUS="$($POP pipeline status --run "$RUN_ID")"
echo "$STATUS" | field run_status | grep -q completed
DOCS="$($POP doc list --run "$RUN_ID")"
echo "$DOCS" | field count | grep -q '^8$'
$POP tool run intent-validate path=products | grep -q 'exit_code: 0'

echo "Full greenfield dogfood passed (issue=$KEY run=$RUN_ID)"
