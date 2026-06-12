# ADR-023 · Issue ↔ Task many-to-many linking (PROJ-43)

> **Status**: Accepted
> **Date**: 2026-06-11
> **Product**: cli-ux
> **Generated-by**: cutover-author（PROJ-43）
> **Source-Equivalence**: doc-99.equivalence-baseline.md
> **Source-Coverage**: doc-98.shadow-implementer.md

## Context

PROJ-42 引入的 `epic_task_id` 仅能 0/1 绑定 task，Guidance 使用固定 5 条旅程启发式，
与「Issue 语义关联多个 task」产品方向不符。PROJ-43 改为 `issue_tasks` 多对多表（`linked` /
`proposed`），并增加独立 `issue-author` skill（不进 pipeline yaml）。

Golden `docs/baseline/2026-06-11/cli-ux-issue-tasks/` 5/5 pass。

## Decision

1. **存储**：`issue_tasks`（SQLite + TSV `issue_task` 行）；加载时将旧 `epic_task_id` 回填为 `linked`。
2. **CLI**：`--tasks T1,T2`；`--proposed-task "title|journey"`（可重复）；`--epic-task` 废弃但兼容。
3. **Guidance**：优先 `linked` / `proposed`；无关联时启发式 `take(3)`。
4. **Agent**：`issue-author` 在 `issue create` 前独立执行（guide 含 pipeline 决策树）。
5. **UI**：创建 Issue 多选 task；详情 Guidance 分区展示。

## Alternatives

| 方案 | 否决理由 |
|---|---|
| 保留 epic 父 task | 用户确认价值低；无法表达多 task |
| issue-author 进 pipeline stage | 需在无 run_id 时创建 Issue，与模型冲突 |

## Consequences

- `migration/traceability.md` — PROJ-43 行 → `cutover-done`
- `products/cli-ux/tasks/README.md` — Roadmap 行更新为 issue_tasks
- `epic_task_id` 列只读兼容，新写入以 `issue_tasks` 为准

## Compliance

| 门禁 | 证据 | 结果 |
|---|---|---|
| Intent Z3 | `intent-validate path=products/cli-ux` exit 0 | pass |
| Equivalence ≥5 golden | baseline.yaml 5/5 | pass |
| cargo test | `make check` | pass |

## Cutover Gate Checklist

- [x] intent gate 已核对
- [x] equivalence gate 已核对（golden_pass=5）
- [x] cargo test 已核对
- [x] 未通过项 blocker：无

## File Manifest

| Path | Change |
|---|---|
| `crates/storage/src/workspace.rs` | `IssueTaskLink` |
| `crates/storage/src/sqlite.rs` | `issue_tasks` table |
| `crates/cli-ux/src/self_host.rs` | create/list/migrate |
| `crates/cli-ux/src/lib.rs` | CLI flags |
| `crates/cli-ux/src/workspace_readers.rs` | Guidance |
| `crates/cli-ux/src/ui/dto.rs` | `task_links` |
| `intent-coder/skills/issue-author/` | new skill |
| `ui/src/pages/IssuesView.tsx` | multi-select |
| `docs/baseline/2026-06-11/cli-ux-issue-tasks/` | golden chain |

## Approval

- **Status**: Accepted
- **Approved by**: PROJ-43 slice-delivery cutover
- **Approval date**: 2026-06-11
