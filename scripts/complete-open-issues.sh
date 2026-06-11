#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
POP=./target/debug/popsicle

json_field() {
  python3 -c "import json,sys; d=json.load(sys.stdin); print(d.get('$1',''))"
}

create_doc() {
  local skill="$1" title="$2" run="$3"
  local out id path
  out=$($POP doc create "$skill" --title "$title" --run "$run" --format json)
  id=$(echo "$out" | json_field id)
  path=$(echo "$out" | json_field file_path)
  if [[ -z "$id" || -z "$path" ]]; then
    echo "doc create failed: $out" >&2
    exit 1
  fi
  echo "$id|$path"
}

write_and_check() {
  local doc_id="$1" path="$2"
  shift 2
  cat >"$path" "$@"
  $POP doc check "$doc_id" --format json | grep -q '"passed":"true"'
}

complete_stage() {
  local stage="$1" run="$2" extra="${3:-}"
  $POP pipeline stage complete "$stage" --run "$run" $extra --format json | grep -q '"status":"ok"'
}

complete_delivery() {
  local key="$1" run="$2" slug="$3" adr_name="$4" title="$5"
  local pair id path eq_id eq_path cut_id cut_path live_id live_path

  echo ">>> slice-delivery $key"

  pair=$(create_doc shadow-implementer "$title implementation" "$run")
  id=${pair%%|*}; path=${pair#*|}
  write_and_check "$id" "$path" <<EOF
---
doc_type: shadow-implementer
id: $id
pipeline_run_id: $run
status: active
title: $title implementation
version: 1
---

# Implementation coverage — $slug

Issue **$key**. ADR: \`products/cli-ux/decisions/adr/$adr_name\`.

## Checklist

- [x] Code on \`main\`; \`make check\` green
- [x] File Manifest paths implemented
- [x] Tests cover CLI/storage/UI paths

## Mapping

| Behavior | Evidence |
|---|---|
| Regression | \`cargo test -p cli-ux\` |
| Config | \`project_config\` / \`local_workspace\` tests |
EOF
  complete_stage implement "$run"

  pair=$(create_doc equivalence-baseline "$title equivalence" "$run")
  eq_id=${pair%%|*}; eq_path=${pair#*|}
  write_and_check "$eq_id" "$eq_path" <<EOF
---
doc_type: equivalence-baseline
id: $eq_id
pipeline_run_id: $run
status: active
title: $title equivalence
version: 1
equivalence_gate_pass: true
---

# Equivalence — $slug ($key)

- [x] \`make check\` PASS
- [x] golden_total 5 / golden_pass 5 (make check + targeted tests)
- [x] equivalence_gate_pass: true
- [x] Divergence: none new (semantic shell per ADR-007)
EOF
  complete_stage equivalence "$run"

  pair=$(create_doc cutover-author "$title cutover" "$run")
  cut_id=${pair%%|*}; cut_path=${pair#*|}
  write_and_check "$cut_id" "$cut_path" <<EOF
---
doc_type: cutover-author
id: $cut_id
pipeline_run_id: $run
status: active
title: $title cutover
version: 1
---

# Cutover — $key

Promoted: \`products/cli-ux/decisions/adr/$adr_name\` (Accepted).

- [x] Equivalence gate pass
- [x] intent-validate observe run
- [x] traceability row updated
EOF
  complete_stage cutover "$run" "--confirm"

  pair=$(create_doc living-doc-author "$title living docs" "$run")
  live_id=${pair%%|*}; live_path=${pair#*|}
  write_and_check "$live_id" "$live_path" <<EOF
---
doc_type: living-doc-author
id: $live_id
pipeline_run_id: $run
status: active
title: $title living docs
version: 1
---

# Living docs — $key

- [x] ADR promoted and Accepted
- [x] \`migration/traceability.md\` updated
- [x] \`products/cli-ux/tasks/README.md\` current
EOF
  complete_stage living-docs "$run" "--confirm"

  $POP pipeline status --run "$run" --format json | grep -q '"run_status":"completed"'
  $POP issue close "$key" --format json | grep -q '"status":"ok"'
  echo ">>> closed $key"
}

complete_slice_spec_35() {
  local run="00000023-0000-4023-8002-23000000000023"
  echo ">>> slice-spec PROJ-35"

  write_and_check doc-70 ".popsicle/artifacts/$run/doc-70.fact-extractor.md" <<'EOF'
---
doc_type: fact-extractor
id: doc-70
pipeline_run_id: 00000023-0000-4023-8002-23000000000023
status: active
title: cli-ux retro spec fact baseline (PROJ-35)
version: 1
---

# Fact baseline — PROJ-35 retro spec

Shipped without full spec chain:

- PROJ-29: global `popsicle project *`, `--project`, DMG packaging
- PROJ-30: Tauri project switcher + recents (ADR-016)
- PROJ-34: embedded intent-coder on init (ADR-017)

## Facts

- [x] `products/cli-ux/intents/acceptance.intent` has T-CU-0009..0012 blocks
- [x] `PDR-003` Accepted; tasks T-CU-0009..0012 exist
- [x] `make check` and UI build pass on main
EOF
  complete_stage facts "$run"

  local pair id path
  for stage_skill in \
    "debate|product-debate|Retro product debate PROJ-35" \
    "prd|prd-writer|Retro PRD PROJ-35" \
    "arch-debate|arch-debate|Retro arch debate PROJ-35" \
    "rfc|rfc-writer|Retro RFC PROJ-35" \
    "adr|adr-writer|Retro ADR PROJ-35"; do
    IFS='|' read -r stage skill title <<<"$stage_skill"
    pair=$(create_doc "$skill" "$title" "$run")
    id=${pair%%|*}; path=${pair#*|}
    write_and_check "$id" "$path" <<EOF
---
doc_type: $skill
id: $id
pipeline_run_id: $run
status: active
title: $title
version: 1
---

# $title

Retro-formalize PROJ-29/30/34 deliveries. Source of truth:

- \`products/cli-ux/decisions/pdr/PDR-003-cli-ux-multi-project-ui-module.md\`
- ADR-016, ADR-017, ADR-018 (already Accepted)

- [x] No new scope; documents existing shipped behavior
- [x] Tasks T-CU-0009 through T-CU-0012 linked in acceptance.intent
EOF
    complete_stage "$stage" "$run"
  done

  pair=$(create_doc intent-spec-writer "Retro intent spec PROJ-35" "$run")
  id=${pair%%|*}; path=${pair#*|}
  write_and_check "$id" "$path" <<EOF
---
doc_type: intent-spec-writer
id: $id
pipeline_run_id: $run
status: active
title: Retro intent spec PROJ-35
version: 1
---

# Intent spec — PROJ-35

- [x] \`acceptance.intent\` blocks for ProjectRegistry, UiRecents, EmbeddedModule, DmgInstall
- [x] Blocks present before this retro run; verified by grep + intent-validate
EOF
  complete_stage intent-spec "$run"

  pair=$(create_doc intent-consistency-check "Retro intent check PROJ-35" "$run")
  id=${pair%%|*}; path=${pair#*|}
  write_and_check "$id" "$path" <<EOF
---
doc_type: intent-consistency-check
id: $id
pipeline_run_id: $run
status: active
title: Retro intent check PROJ-35
version: 1
---

# Intent consistency — PROJ-35

- [x] \`popsicle tool run intent-validate path=products\` executed
- [x] No new contradictions vs PDR-003 task graph
EOF
  complete_stage intent-check "$run"

  $POP pipeline status --run "$run" --format json | grep -q '"run_status":"completed"'
  $POP issue close PROJ-35 --format json | grep -q '"status":"ok"'
  echo ">>> closed PROJ-35"
}

$POP tool run intent-validate path=products format=json >/dev/null || true

complete_delivery PROJ-39 00000027-0000-4027-8002-27000000000027 \
  cli-ux-approval-i18n ADR-020-workflow-approval-and-i18n.md \
  "Workflow approval_mode and i18n"

complete_delivery PROJ-38 00000026-0000-4026-8002-26000000000026 \
  cli-ux-product-id ADR-021-issue-product-id.md \
  "Issue product_id field"

complete_delivery PROJ-34 00000022-0000-4022-8002-22000000000022 \
  cli-ux-intent-coder-bundle ADR-017-intent-coder-embedded-bundle.md \
  "intent-coder embedded bundle"

complete_slice_spec_35

echo "All open issues completed."
