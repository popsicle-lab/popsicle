---
task_id: T-CU-0004
slug: pipeline-stage-advance
title: "我查询 pipeline 状态并推进 stage"
journey_stage: daily-ops
audience: ["ai-coding-agent"]
task_type: 操作指南
decision_ref: PDR-001
last_updated: 2026-06-09
last_verified: ~
intent_kind: validate
involved_features: ["pipeline", "stage", "skill-runtime"]
prerequisites:
  - "已有 pipeline run"
limits:
  - "业务状态机归 skill-runtime，不在 CLI 重写"
related_intents:
  - "acceptance.intent#StageAdvanceReflectsState"
related_next_tasks:
  - T-CU-0005
fact_cite:
  - "docs/baseline/2026-06-08/api-contracts.md § pipeline"
---

# 我查询 pipeline 状态并推进 stage

## 本 task 可解答

- "pipeline status 怎么告诉我当前 stage？"
- "stage complete 后下一个 stage 怎么解锁？"
- "需要人工批准的 stage 怎么确认？"

## 前提与限制

你需要已有 run。CLI 只调用 skill-runtime 的状态机，不复制状态机逻辑。

## 完成路径

1. 运行 `popsicle pipeline status --run <run> --format json`。
2. 读取每个 stage 的 `state` 与 documents。
3. 运行 `popsicle pipeline stage complete <stage> --run <run> [--confirm]`。
4. 再次查询 status，确认当前 stage completed、下游 ready。

## 可观察的成功标志

stage 状态推进与 skill-runtime 状态机一致，requires_approval stage 必须显式 `--confirm`。

## Related Next Tasks

- T-CU-0005

## Charter Compliance

- [x] frontmatter 必填字段齐全
- [x] 文件长度 ≤ 150 行
- [x] title 是完整第一人称句子
- [x] 「本 task 可解答」3 个用户原话问句
- [x] 完成路径无 if-else 分支
- [x] Related Next Tasks ≥ 1
