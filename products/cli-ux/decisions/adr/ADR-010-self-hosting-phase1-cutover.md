# ADR-010 · cli-ux self-hosting Phase 1 cutover

> **Status**: Accepted
> **Date**: 2026-06-11
> **Product**: cli-ux
> **Generated-by**: cutover-author（PROJ-10）
> **Source-Baseline**: `docs/baseline/2026-06-11/cli-ux-self-host/`
> **Supersedes**: ADR-009 consequences only（Phase 1 delivery；设计决策仍有效）

## Context

ADR-008 cut over the semantic CLI shell but deferred storage-backed workspace mutation (D-002).
PROJ-9 dogfood showed agents silently used `../target/debug/popsicle`. PROJ-10 delivers
ADR-009 Phase 1: TSV workspace, IDD workflow commands, doctor provenance, smoke test.

Phase 2 SQLite is tracked by **PROJ-11**; not in scope here.

## Decision

1. **切流范围（self-host Phase 1）**：
   - Binary entrypoint → `SelfHostDomain` + `TsvWorkspace`（`.popsicle/self-host/`）
   - IDD workflow：`issue create/list/show/start`, `pipeline status/next/stage complete`, `doc create/list/show`
   - Provenance：`popsicle doctor --format json`
   - Tool：`tool run intent-validate path=products`
2. **Golden**：`docs/baseline/2026-06-11/cli-ux-self-host/run-all.sh` 8/8 pass
3. **Divergence**：
   - D-001：TSV not SQLite — Phase 2 PROJ-11
   - D-002：Self-host PROJ-N ≠ legacy `popsicle.db` PROJ-N — separate stores
   - D-003：Parent/system binary blocked via doctor flags

## Compliance

| 门禁 | 证据 | 结果 |
|---|---|---|
| Intent Z3 | cli-ux intents verified（observe）| pass |
| Golden ≥5 | `run-all.sh` 8 scripts | pass |
| cargo test | `cargo test -p cli-ux` | pass |
| Smoke | `self_host_workflow_smoke_passes` | pass |

## Cutover Gate Checklist

- [x] intent gate 已核对
- [x] equivalence gate：8/8 golden pass
- [x] cargo test exit 0
- [x] doctor `current_workspace_binary_match=true`

## Waiver Checklist

- [x] 无豁免

## Approval

- **Status**: Accepted
- **Approved by**: PROJ-10 slice-delivery cutover stage
- **Approval date**: 2026-06-11
