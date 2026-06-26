---
task_id: T-TEL-0003
slug: orchestration-auto-span
title: "编排命令自动 emit popsicle.run 与 stage span"
journey_stage: daily-ops
audience: ["ai-coding-agent", "human-maintainer"]
task_type: 操作指南
decision_ref: PDR-001
last_updated: 2026-06-26
last_verified: ~
intent_kind: observe
involved_features: ["orchestration-span", "cli-ux-inject"]
prerequisites:
  - "telemetry crate 已链接 cli-ux（feature-delivery 后）"
limits:
  - "编排 span 不含 LLM thinking 全文"
related_intents:
  - "acceptance.intent#OrchestrationSpanOnIssueStart"
related_next_tasks: []
fact_cite:
  - "PDR-001 §Decision"
---

# 编排命令自动 emit popsicle.run 与 stage span

## 本 task 可解答

- 哪些 popsicle 命令会自动写 span？
- 没有 Agent 配合也能观测什么？
- 编排 span 与 gen_ai span 怎么关联？

## 前提与限制

编排层 span 由 **skill-runtime `PipelineSession`** 在 `start` / `complete_current` 经 `SessionSpanSink` emit（含 `popsicle.skill`）；**`doc check`** 在 cli-ux 层 emit `popsicle.doc.check`；`doc create` 仍在 cli-ux inject。相邻 span 含 `popsicle.duration_ms`。

## 完成路径

1. 运行 `popsicle issue start <key> --format json`（`PipelineSession::start` 经 `SessionSpanSink` 写 span）。
2. 检查 `.popsicle/telemetry/{run_id}/spans.wal.jsonl` 是否含 `popsicle.run.start` / `popsicle.stage.complete` / `popsicle.doc.check` 及 `popsicle.skill`、`popsicle.trace_id`。
3. 完成一个 stage 后检查 span 树 `popsicle.trace_id`（= run_id）一致。
4. 在云端按 `popsicle.issue_key` 聚合耗时。

## 可观察的成功标志

WAL 中存在编排 span；即使 WAL 目录不可写，`issue start` 仍成功返回 run_id。

## Charter Compliance

- [x] frontmatter 必填字段齐全
- [x] title 第一人称
- [x] 「本 task 可解答」3 问句
