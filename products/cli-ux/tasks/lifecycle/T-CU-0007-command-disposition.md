---
task_id: T-CU-0007
slug: command-disposition
title: "我确认旧命令 checklist item sync 被移除或延后"
journey_stage: lifecycle
audience: ["maintainer", "ai-coding-agent"]
task_type: 迁移指南
decision_ref: PDR-001
last_updated: 2026-06-09
last_verified: ~
intent_kind: migrate
involved_features: ["command-tree", "drop-list"]
prerequisites:
  - "理解 legacy 22 命令清单"
limits:
  - "不删除 legacy/popsicle 源码，只定义 popsicle-new CLI 暴露面"
related_intents:
  - "invariants.intent#RemovedCommandsStayRemoved"
related_next_tasks:
  - T-CU-0001
fact_cite:
  - "docs/baseline/2026-06-08/api-contracts.md § 子命令清单（22 个）"
---

# 我确认旧命令 checklist item sync 被移除或延后

## 本 task 可解答

- "checklist 命令为什么不保留？"
- "item 命令和 task_chunk 是什么关系？"
- "sync 是删掉还是以后再做？"

## 前提与限制

你在审查命令树迁移。此 task 不删除 legacy 源码，只定义新 CLI 暴露面。

## 完成路径

1. 查看 PDR-001 disposition 表。
2. 确认 `checklist` → drop，并入 `doc check`。
3. 确认 `item` → drop，由 task_chunk/doc 派生路径替代。
4. 确认 `sync` → defer/drop，不进 IDD MVP。

## 可观察的成功标志

新 `popsicle --help` 不暴露 `checklist` / `item` / `sync` 顶层命令。

## Related Next Tasks

- T-CU-0001

## Charter Compliance

- [x] frontmatter 必填字段齐全
- [x] 文件长度 ≤ 150 行
- [x] title 是完整第一人称句子
- [x] 「本 task 可解答」3 个用户原话问句
- [x] 完成路径无 if-else 分支
- [x] Related Next Tasks ≥ 1
