---
task_id: T-TEL-0002
slug: agent-telemetry-record
title: "Agent 通过 tool run telemetry 上报 gen_ai span"
journey_stage: daily-ops
audience: ["ai-coding-agent"]
task_type: 操作指南
decision_ref: PDR-001
last_updated: 2026-06-26
last_verified: 2026-06-26
intent_kind: produce
involved_features: ["tool-run-telemetry", "gen-ai-span"]
prerequisites:
  - "已有 active pipeline run"
limits:
  - "不替代 Cursor hook；Agent 显式调用"
related_intents:
  - "acceptance.intent#TelemetryRecordFailOpen"
related_next_tasks:
  - T-TEL-0004
fact_cite:
  - "PDR-001 §Decision"
---

# Agent 通过 tool run telemetry 上报 gen_ai span

## 本 task 可解答

- Agent 怎么上报 temperature 和 model？
- `tool run telemetry` 失败会打断 stage 吗？
- span 存在哪里？

## 前提与限制

Active run 可选关联 `run_id` / `doc_id`。写入 `.popsicle/telemetry/{run_id}/spans.wal.jsonl`；失败 fail-open。

## 完成路径

0. 不确定用法时先读 module guide：

   ```bash
   popsicle tool run telemetry action=guide
   ```

1. 在 stage 工作中调用：

   ```bash
   popsicle tool run telemetry action=record span=gen_ai.chat \
     run=<run_id> doc=<doc_id> \
     model=composer-2.5-fast temperature=0.2 \
     input_tokens=1200 output_tokens=400 format=json
   ```

2. 可选上报决策摘要：

   ```bash
   popsicle tool run telemetry action=record span=popsicle.decision \
     run=<run_id> summary="选用 feature-delivery" format=json
   ```

3. 可选 completion score（1–5）：

   ```bash
   popsicle tool run telemetry action=record span=popsicle.run.score \
     run=<run_id> score=4 rubric=spec-clarity format=json
   ```

## 可观察的成功标志

WAL 文件 append 新行；CLI 返回 `status: ok` 或 `status: degraded` 但 **exit 0**；`doc check` 不读取 telemetry。`doc check` 通过时 JSON 含 `telemetry_hint`（score 命令模板）。

字段表见 [`SPAN_SCHEMA.md`](../../SPAN_SCHEMA.md)（spec 维护）；monorepo 约定摘要见 [`AGENT_TELEMETRY.md`](../../AGENT_TELEMETRY.md)。**Agent 运行时**仅须 `popsicle tool run telemetry action=guide`。

## Related Next Tasks

- T-TEL-0003

## Charter Compliance

- [x] frontmatter 必填字段齐全
- [x] title 第一人称
- [x] 「本 task 可解答」3 问句
