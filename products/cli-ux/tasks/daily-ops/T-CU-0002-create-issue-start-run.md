---
task_id: T-CU-0002
slug: create-issue-start-run
title: "我创建 issue 并启动 pipeline run"
journey_stage: daily-ops
audience: ["ai-coding-agent", "maintainer"]
task_type: 操作指南
decision_ref: PDR-001
last_updated: 2026-06-09
last_verified: ~
intent_kind: produce
involved_features: ["issue", "spec", "pipeline"]
prerequisites:
  - "已有 spec"
limits:
  - "不要求 legacy stdout byte parity"
related_intents:
  - "acceptance.intent#IssueStartCreatesRun"
related_next_tasks:
  - T-CU-0003
  - T-CU-0004
fact_cite:
  - "docs/baseline/2026-06-08/api-contracts.md § issue / pipeline"
---

# 我创建 issue 并启动 pipeline run

## 本 task 可解答

- "issue create 后我怎么启动工作流？"
- "spec lock 是什么时候被拿住的？"
- "pipeline run id 从哪里拿？"

## 前提与限制

你需要先有一个 spec。此 task 锁定 JSON 语义字段，不锁 legacy 文案字节。

## 完成路径

1. 运行 `popsicle issue create --spec <spec> --pipeline <pipeline> --format json`。
2. 读取返回的 `key` / `id` / `spec_id` / `pipeline`。
3. 运行 `popsicle issue start <key> --format json`。
4. CLI 返回 `run_id` 与 `next_command`，并持有 spec lock。

## 可观察的成功标志

`issue start` 创建一个 pipeline run，issue 进入 in_progress，spec lock 指向该 run。

## Related Next Tasks

- T-CU-0003
- T-CU-0004

## Charter Compliance

- [x] frontmatter 必填字段齐全
- [x] 文件长度 ≤ 150 行
- [x] title 是完整第一人称句子
- [x] 「本 task 可解答」3 个用户原话问句
- [x] 完成路径无 if-else 分支
- [x] Related Next Tasks ≥ 1
