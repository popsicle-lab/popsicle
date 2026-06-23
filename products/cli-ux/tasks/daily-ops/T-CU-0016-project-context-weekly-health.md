---
task_id: T-CU-0016
slug: project-context-weekly-health
title: "工程画像（PROJECT_CONTEXT）与 weekly 健康巡检"
journey_stage: daily-ops
audience: ["ai-coding-agent", "maintainer"]
task_type: 操作指南
decision_ref: ADR-026
last_updated: 2026-06-23
last_verified: 2026-06-23
intent_kind: produce
involved_features: ["ui", "pipeline", "project_context"]
prerequisites:
  - "已 popsicle init"
related_intents:
  - "acceptance.intent#ProjectContextEditableInSettings"
  - "acceptance.intent#WeeklyHealthCheckPipeline"
  - "acceptance.intent#AgentContextIncludesProjectContext"
related_next_tasks: []
fact_cite:
  - "products/cli-ux/decisions/pdr/PDR-006-project-context-and-weekly-health.md"
  - "products/cli-ux/decisions/adr/ADR-026-project-context-weekly-health.md"
---

# 工程画像（PROJECT_CONTEXT）与 weekly 健康巡检

## 本 task 可解答

- 工程画像保存在哪里？如何编辑？
- weekly 活文档巡检怎么跑？
- `issue start` 会不会注入 PROJECT_CONTEXT？

## 工程画像（单一源）

权威文件：**`docs/PROJECT_CONTEXT.md`**（git 追踪）。

- **§工程画像**：Tech Stack、crate 布局、DevOps、约束 —— 在 **Settings → 工程画像** 编辑，或人工改 git。
- **§现在状态**：product 数、迁移进度 —— 由 **weekly-health-check** 刷新；勿在 Settings 手工改。
- **迁移对照**：[`migration/traceability.md`](../../../migration/traceability.md)（不合并进 PROJECT_CONTEXT）。

## Settings UI

1. 侧栏 **Settings** → **工程画像（PROJECT_CONTEXT）**。
2. 编辑 Markdown → **保存工程画像**。
3. 勾选 **工作流注入** 时，`issue start` / `doc create` 的 `agent_context` 含 §工程画像（截断 4KB）。

## Weekly 巡检

```bash
popsicle issue create --type technical --product cli-ux \
  --pipeline weekly-health-check \
  --title "Weekly 活文档健康巡检" --format json
popsicle issue start <KEY> --format json
popsicle pipeline next --run <run_id>
# doc create living-doc-author --target tasks-index,product-context
popsicle doc check <doc_id>
popsicle pipeline stage complete health-sync --run <run_id>
popsicle issue close <KEY>
```

外部 cron 可每周调用上述流程（见 [`docs/MIGRATION.md`](../../../docs/MIGRATION.md)）。

## slice-delivery living-docs

仅 `--target implementation-status,architecture-manifest,product-header`。**不含** tasks-index / product-context。

## 验证

```bash
docs/baseline/2026-06-23/cli-ux-weekly-health/run-all.sh
make check
```
