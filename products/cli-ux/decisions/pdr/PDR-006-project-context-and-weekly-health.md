# PDR-006: 工程画像单一源与 weekly 健康巡检

> **Status**: Accepted
> **Date**: 2026-06-23
> **Product**: cli-ux
> **Source**: PROJ-55
> **Related ADRs**: ADR-026

## Decision Context

`docs/PROJECT_CONTEXT.md` 长期停留在 project-init 骨架：living-doc-author 的 `product-context` target 只刷「现在状态」，不填 Tech Stack；slice-delivery 默认也不跑该 target。用户需要单一 Markdown 源（UI 可编辑）+ weekly 巡检 + agent 注入。

## Decision

1. **单一源**：`docs/PROJECT_CONTEXT.md`（git）；废弃 `.popsicle/project-context.md` 为权威源。
2. **结构**：§工程画像（UI/人工）、§现在状态（weekly 机械刷新）、§相关链接、§未来 collab 触发条件（PDR-001 A01）。
3. **weekly-health-check pipeline**：1 stage `living-doc-author`，target=`tasks-index,product-context`，无 `requires_approval`。
4. **Settings UI**：读写 PROJECT_CONTEXT Markdown。
5. **注入**：`inject_on_run` 时追加 §工程画像（截断 4KB，不含 §现在状态）。
6. **迁移对照**：仍在 `migration/traceability.md`；PROJECT_CONTEXT 只链接。

## Consequences

- 新增 `products/cli-ux/tasks/daily-ops/T-CU-0016-project-context-weekly-health.md`
- `acceptance.intent` Block 15–17
- `crates/cli-ux/src/project_context.rs`、Tauri IPC、Settings UI
- living-doc / project-init / pipeline yaml 口径修正

## Intent Impact

| Intent | Task | File |
|---|---|---|
| `ProjectContextEditableInSettings` | T-CU-0016 | acceptance.intent |
| `WeeklyHealthCheckPipeline` | T-CU-0016 | acceptance.intent |
| `AgentContextIncludesProjectContext` | T-CU-0016 | acceptance.intent |

## Validation Plan

- `intent-validate path=products/cli-ux/intents` exit 0
- golden: issue start 含 Project context；weekly pipeline install
- `make check`
