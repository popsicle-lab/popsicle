---
task_id: T-AR-0003
slug: view-run-progress
title: "我日常用手机查看 pipeline 阶段进度"
journey_stage: daily-ops
audience: ["end-user"]
task_type: 操作指南
decision_ref: PDR-001
last_updated: 2026-07-06
intent_kind: observe
involved_features: ["websocket", "pipeline-mirror"]
prerequisites:
  - "user.has(active_or_recent_dispatch)"
related_intents: []
related_next_tasks:
  - T-AR-0004
fact_cite:
  - "doc-181.product-debate.md § Phase 1"
---

# 我日常用手机查看 pipeline 阶段进度

## 本 task 可解答

- 手机上怎么看当前 run 在哪个 stage？
- doc check 失败会在手机上有提示吗？
- 和桌面 `popsicle ui` 看到的内容一致吗？

## 前提与限制

已有派活或历史 run。手机读 Server 镜像 + WS 推送；真相仍在开发机 `.popsicle/state.db` `[假设：P0 单向同步]`。

## 完成路径

1. 打开 App **Runs** 列表，点选 run_id。

2. 查看 **阶段 DAG**：与 `popsicle pipeline status --format json` 的 `stages` 一致。

3. 展开 **日志/事件**：Daemon 上报的 stage 名、doc_id、exit_code。

4. `doc check` 失败时显示 **failed** 与 `next` 提示（来自 popsicle JSON）。

## 可观察的成功标志

手机当前 stage 与开发机 CLI `pipeline status` 一致（允许 ≤15s 同步延迟 `[假设]`）。

## Related Next Tasks

- **T-AR-0004** — 远程审批

`Decision-Ref: PDR-001`
