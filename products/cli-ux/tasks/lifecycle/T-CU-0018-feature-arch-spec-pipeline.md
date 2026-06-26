---
task_id: T-CU-0018
slug: feature-arch-spec-pipeline
title: "我为大增量能力选用 feature-arch-spec pipeline 并完成 bundled 安装"
journey_stage: lifecycle
audience: ["ai-coding-agent", "human-maintainer"]
task_type: 操作指南
decision_ref: ADR-030
last_updated: 2026-06-26
last_verified: 2026-06-26
intent_kind: produce
involved_features: ["workflow-catalog", "pipeline-taxonomy"]
prerequisites:
  - "ADR-030 Accepted"
limits:
  - "本 task 仅覆盖 pipeline yaml 与文档同步，不含 telemetry 实现"
related_intents: []
related_next_tasks: []
fact_cite:
  - "products/cli-ux/decisions/adr/ADR-030-feature-arch-spec-pipeline.md"
---

# 我为大增量能力选用 feature-arch-spec pipeline 并完成 bundled 安装

## 本 task 可解答

- "已有 product 大增量 spec 该用哪个 pipeline？"
- "feature-arch-spec 与 feature-spec 差在哪？"
- "bundled pipeline 如何自 heal 安装？"

## 完成路径

1. 确认 Issue 描述含 PDR+ADR 需求且非新 product → `--pipeline feature-arch-spec`。
2. 确认 `intent-coder/pipelines/feature-arch-spec.pipeline.yaml` 存在。
3. 运行 `popsicle admin sync-intent-coder` 或 `popsicle init` 触发自 heal。
4. `popsicle help` 或 UI Workflows 页可见 `feature-arch-spec`。

## 可观察的成功标志

`pipeline_taxonomy` 测试断言 `feature-arch-spec` 的 domain 为 `feature`；issue-author 决策树含三分支。

## Charter Compliance

- [x] frontmatter 必填字段齐全
- [x] title 第一人称
- [x] 「本 task 可解答」3 问句
