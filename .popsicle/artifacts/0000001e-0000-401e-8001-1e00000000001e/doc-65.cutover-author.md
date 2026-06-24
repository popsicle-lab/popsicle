---
doc_type: cutover-author
id: doc-65
pipeline_run_id: 0000001e-0000-401e-8001-1e00000000001e
status: active
title: PROJ-30 project UI cutover
version: 1
---

# PROJ-30 project UI cutover

## Summary

Cutover ADR-016 accepts UI integration with `~/.popsicle/global.json`:
recent projects, in-app switching, modern desktop shell, and `.app` auto-launch.

## Gates

| Gate | Evidence | Status |
|---|---|---|
| Golden ≥4 | `cli-ux-project-ui/run-all.sh` | pass |
| `make check` | fmt + clippy + test | pass |
| Intent Z3 | `make intent` | pass |
| Cutover ADR | `ADR-016-ui-project-switcher.md` Accepted | pass |

## Decision record

Promoted to `products/cli-ux/decisions/adr/ADR-016-ui-project-switcher.md`.

## Post-cutover

- [x] `migration/traceability.md` updated (slice-4-ui rows)
- [x] `migration/progress.md` Last-Decision-Ref → ADR-016
- [x] Golden chain includes `cli-ux-global` + `cli-ux-project-ui`

## Approval

User confirms `pipeline stage complete cutover --confirm` after review.
