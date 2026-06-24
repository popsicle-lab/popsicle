---
doc_type: living-doc-author
id: doc-66
pipeline_run_id: 0000001e-0000-401e-8001-1e00000000001e
status: active
title: PROJ-30 living docs sync
version: 1
---

# PROJ-30 living docs sync

## Summary

Refreshed migration living docs and packaging README for UI project workflow.

## Documents updated

| Document | Change |
|---|---|
| `migration/traceability.md` | ADR-016 rows for project UI + global bridge |
| `migration/progress.md` | PROJ-30 note, ADR-016 ref |
| `packaging/macos/README.md` | DMG install + UI launch steps (prior commit) |
| `products/cli-ux/decisions/adr/ADR-016-*` | New Accepted ADR |

## UI operator notes

1. Open Popsicle from Applications (not inside DMG).
2. Welcome screen lists recent projects from `global.json`.
3. Sidebar switcher changes workspace; writes default + MRU.
4. CLI `popsicle project list` stays in sync with UI registry.

## Checklist

- [x] traceability rows present
- [x] progress.md Last-Decision-Ref current
- [x] ADR-016 on disk under products/cli-ux/decisions/adr/
