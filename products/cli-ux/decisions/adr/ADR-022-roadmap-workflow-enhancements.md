# ADR-022 · Roadmap workflow enhancements (PROJ-42 P1–P6)

> **Status**: Accepted
> **Date**: 2026-06-11
> **Product**: cli-ux
> **Generated-by**: cutover-author（PROJ-42）
> **Source-Equivalence**: doc-95.equivalence-baseline.md
> **Source-Coverage**: doc-94.shadow-implementer.md

## Context

README Roadmap P1–P6 adds workflow personas, product health signals, issue
grouping, retro doc checklist, mermaid in task markdown, and optional
`epic_task_id` on issues. Implementation landed in `crates/cli-ux/`, `storage/`,
and `ui/` without new `.intent` acceptance blocks (retro path per README §两条用户旅程).

Golden baseline `docs/baseline/2026-06-11/cli-ux-roadmap-workflow/` passes 5/5.
`migration/traceability.md` row is **in-shadow** pending this ADR.

## Decision

1. **切流范围**：greenfield UI/CLI enhancements on already-cutover `cli-ux` slice;
   no legacy submodule path to sunset.
2. **主路径**：`popsicle` CLI + Tauri UI consume new fields immediately;
   `workflow.profile` in `.popsicle/project.yaml` is authoritative for default
   pipelines and approval hints.
3. **已知 divergence**：无（全新能力，非 legacy parity 对账）。

## Alternatives

| 方案 | 否决理由 |
|---|---|
| 先跑 slice-spec 再 delivery | retro 增量；README 已定义轻路径 |
| 跳过 golden | 违反 CONTRIBUTING §4 |

## Consequences

- `migration/traceability.md` — PROJ-42 行 → `cutover-done`
- `migration/progress.md` — cli-ux 备注追加 PROJ-42
- `README.md` Roadmap — 保持 P1–P6 ☑（living-doc-author 补 task 行）

## Compliance

| 门禁 | 证据 | 结果 |
|---|---|---|
| Intent Z3 | `intent-validate path=products/cli-ux` exit 0（无新 block）| pass |
| Equivalence ≥5 golden | doc-95 / baseline.yaml 5/5 | pass |
| cargo test | `make check` + roadmap run-all | pass |

## Cutover Gate Checklist

- [x] intent gate 已核对（cli-ux 无回归）
- [x] equivalence gate 已核对（golden_pass=5）
- [x] cargo test 已核对（make check exit 0）
- [x] 未通过项已列明 blocker（无）

## Waiver Checklist

- [x] 用户书面确认豁免哪一门禁 — N/A（三门禁均 pass）
- [x] 豁免理由写入 ADR § Compliance — N/A
- [x] 补偿措施已列出 — N/A

## File Manifest

| Path | Change |
|---|---|
| `crates/cli-ux/src/project_config.rs` | `WorkflowProfile` |
| `crates/cli-ux/src/workspace_readers.rs` | `scan_product_health` |
| `crates/cli-ux/src/self_host.rs` | `--epic-task`, TSV col 10 |
| `crates/storage/src/sqlite.rs` | `epic_task_id` migration |
| `ui/src/components/*` | ProductHealth, Mermaid, RetroDocBanner |
| `ui/src/lib/issueGroup.ts` | issue grouping |
| `intent-coder/guides/retro-doc-checklist.md` | P4 guide |

## Approval

- **Status**: Accepted
- **Approved by**: PROJ-42 slice-delivery cutover (`--confirm`)
- **Approval date**: 2026-06-11
