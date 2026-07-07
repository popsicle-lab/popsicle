---
task_id: T-AR-0004
slug: mobile-stage-approval
title: "我在手机上批准 requires_approval 阶段"
journey_stage: daily-ops
audience: ["admin", "end-user"]
task_type: 操作指南
decision_ref: PDR-001
last_updated: 2026-07-06
intent_kind: approve
involved_features: ["approval-push", "stage-complete-confirm"]
prerequisites:
  - "run.stage.requires_approval == true"
  - "project.workflow.approval_mode allows mobile confirm"
related_intents:
  - "acceptance.intent#ApprovalCreatesConfirmTask"
related_next_tasks:
  - T-AR-0003
fact_cite:
  - "AGENTS.md delegate-dangerous 审批模式"
---

# 我在手机上批准 requires_approval 阶段

## 本 task 可解答

- 手机上怎么点批准 pipeline stage？
- `cutover` / `living-docs` 能在手机上批吗？
- 批准后 Daemon 会自动继续吗？

## 前提与限制

`workflow.approval_mode` 为 `delegate-dangerous` 时，非危险 stage 可由 Daemon 代批；**危险 stage**（`cutover`、`living-docs`）仍须显式 `--confirm`，手机操作等价于下发 confirm 任务。

## 完成路径

1. App 推送 **待审批**（stage 名 + artifact 摘要链接）。

2. 用户审阅 artifact（只读预览或跳转桌面）。

3. 点 **批准** → Server 入队 `stage_complete --confirm` 任务。

4. Daemon 执行 `popsicle pipeline stage complete <stage> --run <id> --confirm`。

5. run 进入下一阶段；WS 推送更新。

## 可观察的成功标志

CLI `pipeline status` 显示该 stage `completed` 且 `current_stage` 前进。形式化：`acceptance.intent#ApprovalCreatesConfirmTask`。

## Related Next Tasks

- **T-AR-0003** — 继续查看进度

`Decision-Ref: PDR-001`
