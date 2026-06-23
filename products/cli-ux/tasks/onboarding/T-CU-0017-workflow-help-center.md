---
task_id: T-CU-0017
slug: workflow-help-center
title: "在桌面 UI 浏览 intent-coder 工作流帮助（Pipeline / Skill）"
journey_stage: onboarding
audience: ["end-user", "ai-coding-agent", "maintainer"]
task_type: 操作指南
decision_ref: ADR-027
last_updated: 2026-06-23
last_verified: 2026-06-23
intent_kind: produce
involved_features: ["ui", "workflow_catalog", "WorkflowsView"]
prerequisites:
  - "已 popsicle init 且 intent-coder 模块可用"
related_intents:
  - "acceptance.intent#WorkflowHelpCenterBrowsable"
related_next_tasks:
  - T-CU-0002
  - T-CU-0004
fact_cite:
  - "products/cli-ux/decisions/pdr/PDR-007-workflow-help-center-ui.md"
  - "products/cli-ux/decisions/adr/ADR-027-workflow-help-center-ui.md"
---

# 在桌面 UI 浏览 intent-coder 工作流帮助（Pipeline / Skill）

## 本 task 可解答

- intent-coder 有哪些 Pipeline 模板？各 stage 做什么？
- 某个 Skill 产出什么 artifact？状态机有哪些状态？
- 我当前的 Issue 跑到 pipeline 哪一阶段了？对应哪个 skill？

## 完成路径

1. 打开 Popsicle UI，侧栏进入 **工作流帮助**（Workflows）。
2. **Pipeline 模板** Tab：浏览已安装 pipeline 列表，选中后查看描述、阶段 DAG、各 stage 关联 skill。
3. **Skills 技能** Tab：浏览 intent-coder skill 目录，查看 description、artifacts、workflow states、出现于哪些 pipeline。
4. 在 Issue 详情或 Pipeline 运行页点击「查看工作流帮助」，帮助页高亮当前 pipeline 与 stage。
5. 可选：从帮助页用推荐 pipeline 创建新 Issue。

## 可观察的成功标志

- `get_workflow_catalog` 返回 pipelines + skills + workflow_profile 映射。
- 帮助页 master-detail 可切换 pipeline/skill 且无占位符。
- 从 Issue 带 `contextRunId` 打开帮助页时，显示当前 stage 高亮。
