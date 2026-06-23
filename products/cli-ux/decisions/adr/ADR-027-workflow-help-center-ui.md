# ADR-027 · Workflow help center UI（PROJ-57）

> **Status**: Accepted
> **Date**: 2026-06-23
> **Product**: cli-ux
> **Source-PDR**: PDR-007

## Context

intent-coder Pipeline/Skill 知识分散在 YAML 与 guide.md；桌面 UI 无统一入口。Issue 执行 pipeline 时开发者难以对照「当前阶段 → skill → artifact」。

## Decision

1. **`workflow_catalog.rs`** 构建只读 catalog（pipelines + skills + workflow_profile 推荐）。
2. **Tauri `get_workflow_catalog`** 供 React 帮助页消费。
3. **`WorkflowsView`** 侧栏「工作流帮助」：Pipeline DAG + 阶段说明 + Skill 目录。
4. **Issue 联动**：IssueDetailView / PipelineView → 帮助页（`contextRunId` + stage 高亮）。
5. **纯新 UI**：equivalence 阶段登记 N/A；切流以测试 + intent 为准（无 legacy golden）。

## File Manifest

| Path | Change |
|---|---|
| `crates/cli-ux/src/workflow_catalog.rs` | 新增 |
| `crates/cli-ux/src/ui/commands.rs` | `get_workflow_catalog` |
| `crates/cli-ux/tests/workflow_catalog.rs` | 集成测试 |
| `ui/src/pages/WorkflowsView.tsx` | 帮助中心页 |
| `ui/src/lib/pipelineLayout.ts` | DAG 布局 |
| `ui/src/App.tsx` / `Sidebar.tsx` / `navigation.ts` | 路由 |
| `ui/src/pages/IssueDetailView.tsx` | 帮助入口 |
| `ui/src/pages/PipelineView.tsx` | 帮助入口 + skill 链接 |
| `ui/src/pages/SettingsView.tsx` | 工作流画像 → 帮助 |
| `ui/src/i18n/messages.ts` / `useTauri.ts` | 文案与 IPC |
| `products/cli-ux/intents/acceptance.intent` | Block 18 |
| `products/cli-ux/tasks/onboarding/T-CU-0017-*.md` | task |

## Compliance

| 门禁 | 结果 |
|---|---|
| `make check` | pass |
| intent-validate | pass |
| legacy golden | N/A（新能力） |
