---
task_id: T-AR-0008
slug: bootstrap-issue-from-chat
title: "我确认 Chat 需求摘要后自动创建 Issue 并开始 pipeline"
journey_stage: daily-ops
audience: ["end-user"]
task_type: 操作指南
decision_ref: PDR-002
last_updated: 2026-07-07
intent_kind: operate
involved_features: ["bootstrap-api", "issue-create", "orchestrator", "run-mirror"]
prerequisites:
  - "runtime.online(T-AR-0001)"
  - "chat.session.ready(T-AR-0007)"
related_intents:
  - "acceptance.intent#BootstrapCreatesIssueAndRun"
related_next_tasks:
  - T-AR-0003
  - T-AR-0004
---

# 我确认 Chat 需求摘要后自动创建 Issue 并开始 pipeline

## 本 task 可解答

- Chat 聊完后怎么变成 PROJ-xx 并自动跑 pipeline？
- Issue description 里会不会带上 Chat 上下文？
- bootstrap 和旧「派活页填已有 Key」怎么分工？

## 前提与限制

会话状态为 `ready`（含可编辑的 `draft_title`、`draft_pipeline`、`draft_description`）。Daemon online。bootstrap **在本机**执行 `popsicle issue create` + `issue start` + 现有 `run_unattended` orchestrator；Server 仍只持 mirror。用户可在 Mobile 上修改 pipeline 再确认。

## 完成路径

1. **Mobile** 审阅 draft 卡片（标题 / pipeline / 需求摘要），点 **「创建 Issue 并开始」**。

2. **Server** `POST /v1/chat/sessions/{id}/bootstrap` → 入队 `bootstrap_task`。

3. **Daemon** claim →  
   `popsicle issue create --format json`（description 含「需求摘要 + Chat 引用」）→  
   `popsicle issue start <key>` →  
   `sync_run_mirror` →  
   `run_unattended` orchestrator。

4. **Server** 更新 session：`linked_issue_key`、`linked_run_id`、`status=bootstrapped`；WS 事件 `session_bootstrapped`。

5. **Mobile** 跳转 **进度 Tab**（T-AR-0003）；后续批准走 T-AR-0004。

## 可观察的成功标志

本机 `popsicle issue show <key>` 存在新 Issue；`pipeline status` 为 `in_progress`；Mobile 进度列表出现 run 且 `issue_key` 正确。形式化：`acceptance.intent#BootstrapCreatesIssueAndRun`。

## Related Next Tasks

- **T-AR-0003** — 查看 run 进度
- **T-AR-0004** — 远程批准危险 stage

`Decision-Ref: PDR-002`
