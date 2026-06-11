---
task_id: T-CU-0006
slug: admin-maintenance
title: "我执行低频 admin migrate 和 reinit"
journey_stage: admin
audience: ["maintainer"]
task_type: 管理指南
decision_ref: PDR-001
last_updated: 2026-06-09
last_verified: ~
intent_kind: operate
involved_features: ["admin", "migrate", "reinit"]
prerequisites:
  - "维护者明确进入 admin 子树"
limits:
  - "不把 migrate 保留为顶层命令"
related_intents:
  - "acceptance.intent#AdminCommandsAreExplicit"
related_next_tasks:
  - T-CU-0007
fact_cite:
  - "docs/baseline/2026-06-08/api-contracts.md § admin / migrate / reinit"
---

# 我执行低频 admin migrate 和 reinit

## 本 task 可解答

- "migrate 还在顶层命令吗？"
- "reinit 属于主路径还是 admin？"
- "低频维护命令如何避免误触？"

## 前提与限制

你是维护者。`migrate` / `reinit` 不属于日常 IDD 主路径。

## 完成路径

1. 运行 `popsicle admin --help`。
2. 看到 `migrate` / `reinit` 作为 admin 子命令。
3. 运行前 CLI 展示目标 workspace 与影响范围。
4. 执行后 CLI 返回结构化结果。

## 可观察的成功标志

低频维护动作必须显式进入 `admin` 子树，顶层命令树不暴露 `migrate`。

## Related Next Tasks

- T-CU-0007

## Charter Compliance

- [x] frontmatter 必填字段齐全
- [x] 文件长度 ≤ 150 行
- [x] title 是完整第一人称句子
- [x] 「本 task 可解答」3 个用户原话问句
- [x] 完成路径无 if-else 分支
- [x] Related Next Tasks ≥ 1
