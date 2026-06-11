# ADR-018 · UI modern layout + master-detail shell (PROJ-36)

> **Status**: Accepted
> **Date**: 2026-06-11
> **Product**: cli-ux / slice-4-ui
> **Generated-by**: cutover-author（PROJ-36）
> **Extends**: ADR-015, ADR-016

## Context

PROJ-30 delivered a functional project switcher and welcome shell. PROJ-36
refreshes **layout and interaction** without new IPC: collapsible sidebar,
breadcrumb header, Issues master-detail on wide screens, Pipeline graph +
stage inspector split, Products explorer split panes, shared CSS primitives.

Visual polish is intentionally **not** encoded in `acceptance.intent` (D-701).

## Decision

1. **Design tokens**: `ui/src/index.css` zinc palette + `.card` / `.btn` /
   `.tab-group` / `.badge` primitives.
2. **Navigation**: `lib/navigation.ts` breadcrumbs + back; sidebar collapse
   persisted in `localStorage`.
3. **Issues**: `useWideLayout` (≥1100px) enables list + detail split;
   narrow view keeps full-page navigation.
4. **Pipeline / Products**: `pipeline-split` and `explorer-split` layouts.
5. **Golden**: `docs/baseline/2026-06-11/cli-ux-ui-modern/` (5 scripts).

## Divergences

- **D-701**: Typography/spacing/aesthetics not in Z3 acceptance blocks;
  guarded by golden structural checks + `npm run build`.

## Compliance

| 门禁 | 证据 | 结果 |
|---|---|---|
| `make check` | fmt + clippy + test | pass |
| UI modern golden 5/5 | `cli-ux-ui-modern/run-all.sh` | pass |
| `npm run build` (ui/) | golden-001 | pass |
| No new IPC commands | diff scoped to `ui/` + `App.tsx` shell | pass |

## Approval

- **Status**: Accepted
- **Approved by**: PROJ-36 slice-delivery (user-approved pipeline advance)
- **Approval date**: 2026-06-11
