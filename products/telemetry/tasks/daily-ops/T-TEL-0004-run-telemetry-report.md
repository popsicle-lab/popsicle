---
task_id: T-TEL-0004
slug: run-telemetry-report
title: "我对单个 pipeline run 生成 telemetry 聚合报告"
journey_stage: daily-ops
audience: ["human-maintainer", "ai-coding-agent"]
task_type: 操作指南
decision_ref: PDR-002
last_updated: 2026-06-26
last_verified: 2026-06-26
intent_kind: observe
involved_features: ["tool-run-telemetry", "run-report"]
prerequisites:
  - "目标 run 已有 `.popsicle/telemetry/{run_id}/spans.wal.jsonl`（Phase A–C）"
limits:
  - "报告不进 doc check / pipeline gate"
  - "默认 stdout；不自动 git 提交"
related_intents:
  - "acceptance.intent#TelemetryReportFailOpen"
related_next_tasks: []
fact_cite:
  - "PDR-002 §Decision"
---

# 我对单个 pipeline run 生成 telemetry 聚合报告

## 本 task 可解答

- 怎么从 WAL 看某次 run 的 stage 耗时？
- doc check 失败几次、哪个 skill？
- Agent 有没有上报 gen_ai / score？

## 前提与限制

需要 Phase A–C 产生的 WAL。无 WAL 时 report 返回空结构 + exit 0（fail-open）。

## 完成路径

1. 生成单 run 报告：

   ```bash
   popsicle tool run telemetry action=report run=<run_id> format=json --format json
   ```

2. 可选扫描 workspace 下全部 run：

   ```bash
   popsicle tool run telemetry action=report path=.popsicle/telemetry limit=10 format=json --format json
   ```

3. 根据报告字段（见 [`REPORT_SCHEMA.md`](../../REPORT_SCHEMA.md)）判断：
   - `stages[]` — stage 时间线与 `duration_ms`
   - `doc_checks` — passed/failed 计数与 skill 分布
   - `agent_coverage` — 是否有 `gen_ai.chat` / `popsicle.run.score`

4. Weekly：维护者跑 `doc-sync-weekly`，`PROJECT_CONTEXT` §现在状态 会有一行 telemetry 健康摘要。

## 可观察的成功标志

CLI 返回 JSON/text，`status: ok`，exit 0；即使 WAL 缺失也不影响其他 popsicle 命令。字段表见 [`REPORT_SCHEMA.md`](../../REPORT_SCHEMA.md)。

## Related Next Tasks

（无 — Phase D MVP 止于 report + weekly 摘要）

## Charter Compliance

- [x] frontmatter 必填字段齐全
- [x] title 第一人称
- [x] 「本 task 可解答」3 问句
