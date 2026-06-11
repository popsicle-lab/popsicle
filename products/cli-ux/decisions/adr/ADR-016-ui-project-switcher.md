# ADR-016 · UI project switcher + recents (global.json bridge)

> **Status**: Accepted
> **Date**: 2026-06-11
> **Product**: cli-ux
> **Generated-by**: cutover-author（PROJ-30）
> **Extends**: ADR-015（Tauri UI self-host bridge）

## Context

PROJ-29 delivered CLI-side multi-project registry (`~/.popsicle/global.json`,
`popsicle project *`). The Tauri UI (ADR-015) still used in-memory
`set_project_dir` only: no recents, no switcher, no persistence across app
restarts, and double-clicking `Popsicle.app` printed CLI help instead of
opening the window.

## Decision

1. **Schema extension**: `ProjectEntry.last_opened_at` (unix seconds) in
   `global.json`; `open_project()` registers, touches MRU, and sets default.
2. **Startup**: `resolve_ui_startup_root` — `--project` → default → MRU → cwd;
   `.app` launch with no argv opens UI directly (`main.rs`).
3. **Tauri IPC**: `open_project_cmd`, `list_registered_projects`,
   `get_active_project`, `pick_project_directory` (rfd), `remove_registered_project`,
   `resolve_startup_project`.
4. **UI shell**: `ProjectSwitcher` in sidebar, modern welcome `ProjectPicker`
   with recent cards + native browse, `AppHeader` for page context.
5. **Golden**: `docs/baseline/2026-06-11/cli-ux-project-ui/` (4 scripts);
   chained from `cli-ux-sqlite-phase2/run-all.sh` together with
   `cli-ux-global/`.

## Divergences / Deferred

- **D-601**: No URL deep-link routing for issues/docs (still in-app state only).
- **D-602**: `ProjectWatcher` fs debounce still deferred (ADR-015 O-501).
- **D-603**: Unsigned DMG Gatekeeper flow unchanged; no Apple signing.

## Compliance

| 门禁 | 证据 | 结果 |
|---|---|---|
| `make check` | fmt + clippy + test | pass |
| Project UI golden 4/4 | `cli-ux-project-ui/run-all.sh` | pass |
| Global golden 2/2 | `cli-ux-global/run-all.sh` | pass |

## Approval

- **Status**: Accepted
- **Approved by**: PROJ-30 slice-delivery
- **Approval date**: 2026-06-11
