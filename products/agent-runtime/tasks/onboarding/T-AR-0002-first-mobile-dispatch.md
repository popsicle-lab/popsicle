---
task_id: T-AR-0002
slug: first-mobile-dispatch
title: "我第一次在手机上派活并看到开发机开始跑"
journey_stage: onboarding
audience: ["end-user", "new-user"]
task_type: 操作指南
decision_ref: PDR-001
last_updated: 2026-07-06
intent_kind: operate
involved_features: ["dispatch-api", "task-queue", "websocket"]
prerequisites:
  - "runtime.online(T-AR-0001)"
  - "user.has(agent_profile_bound_to_runtime)"
related_intents:
  - "acceptance.intent#DispatchQueuedWhenRuntimeOnline"
related_next_tasks:
  - T-AR-0003
  - T-AR-0005
fact_cite:
  - "doc-182.product-debate.md § 候选 Tasks TBD-1"
---

# 我第一次在手机上派活并看到开发机开始跑

## 本 task 可解答

- 怎么第一次在手机上把 Issue 派给 Agent？
- 派活后多久能在手机看到 running？
- 派活和 `popsicle issue start` 是什么关系？

## 前提与限制

Runtime 已 online（T-AR-0001）。派活绑定已有 `product` + `pipeline` + 可选 `issue_key`。同一 Issue 同时仅一个 active run（与 skill-runtime 一致）。

## 完成路径

1. **在手机 App** 选择 workspace → 创建或选择 Issue → 选择 Agent Profile → 点 **派活**。

2. **Server** 创建 `DispatchTask`，状态 `queued`。

3. **Daemon** 下次轮询认领 → `dispatched` → 执行 `popsicle issue start`（若未启动）或继续 `pipeline next` → `running`。

4. **手机** WebSocket 收到 stage/log 更新；任务时间线显示 **running**。

> 端到端延迟目标见 task「可观察的成功标志」（不进 acceptance.intent，遵守 D2）。

## 可观察的成功标志

手机时间线从 `queued` 变为 `running`，且开发机 `popsicle pipeline status` 显示 `run_status: in_progress`。形式化：`acceptance.intent#DispatchQueuedWhenRuntimeOnline`。

## Related Next Tasks

- **T-AR-0003** — 日常看进度
- **T-AR-0005** — 派活失败排查

`Decision-Ref: PDR-001`
