---
doc_type: equivalence-baseline
id: doc-64
pipeline_run_id: 0000001e-0000-401e-8001-1e00000000001e
status: active
title: PROJ-30 project UI equivalence
version: 1
---

# PROJ-30 project UI equivalence

## Summary

Established golden baselines for slice-4-ui project registry integration:
`global.json` recents, Tauri open/switch commands, and UI build. No legacy
byte-level parity target — greenfield UI behavior per ADR-015/016.

## Scope

| Area | New implementation |
|---|---|
| Recents | `ProjectEntry.last_opened_at` + `open_project()` |
| Startup | `resolve_ui_startup_root` + `.app` zero-arg UI launch |
| Tauri IPC | `open_project_cmd`, `list_registered_projects`, `pick_project_directory` |
| UI | `ProjectSwitcher`, welcome `ProjectPicker`, `AppHeader` |

## Golden inventory

Baseline: `docs/baseline/2026-06-11/cli-ux-project-ui/`

| ID | Script | Result |
|---|---|---|
| G-001 | `golden-001-open-project-recents.sh` | pass |
| G-002 | `golden-002-startup-resolution.sh` | pass |
| G-003 | `golden-003-ui-build.sh` | pass |
| G-004 | `golden-004-tauri-project-commands.sh` | pass |

Related chain (PROJ-29 global CLI): `docs/baseline/2026-06-11/cli-ux-global/` (2 scripts).

## Divergence register

| ID | Legacy | New | ADR |
|---|---|---|---|
| D-601 | N/A (no global.json recents in legacy UI) | MRU in `global.json` | ADR-016 |
| D-602 | Legacy 14-page shell | MVP+ switcher only | ADR-015 D-501 |

## Verification

- [x] All project-ui golden scripts executed locally
- [x] `make check` green
- [x] `migration/traceability.md` rows drafted for ADR-016

## Traceability rows

See `migration/traceability.md` — slice-4-ui project switcher + CLI registry bridge.
