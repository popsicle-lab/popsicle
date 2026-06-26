---
task_id: T-TEL-0001
slug: configure-otlp-endpoint
title: "我配置 OTLP 端点并确认 trace 已上云"
journey_stage: admin
audience: ["human-maintainer", "ai-coding-agent"]
task_type: 配置任务
decision_ref: PDR-001
last_updated: 2026-06-26
last_verified: ~
intent_kind: configure
involved_features: ["otlp-export", "otel-yaml"]
prerequisites:
  - "已有 .popsicle/ workspace"
limits:
  - "配置错误不得阻塞 issue/pipeline/doc 主命令"
related_intents:
  - "acceptance.intent#OtlpConfigRoundTrips"
related_next_tasks:
  - T-TEL-0002
fact_cite:
  - "PDR-001 §Decision"
  - "doc-140 product-debate"
---

# 我配置 OTLP 端点并确认 trace 已上云

## 本 task 可解答

- OTLP 端点配置写在哪里？
- 怎么确认 trace 已经导出到云端？
- 导出失败会影响 `popsicle issue start` 吗？

## 前提与限制

你需要可访问的 OTLP HTTP/gRPC 端点（或本地 Collector）。telemetry 为 fail-open：导出失败仅诊断，主流程 exit 0。

## 完成路径

1. 在 workspace `.popsicle/otel.yaml` 或 `~/.popsicle/otel.yaml` 写入 `exporter.endpoint` 与可选 `headers`。
2. 运行一次编排命令（如 `issue start`）或 `popsicle tool run telemetry action=record ...` 产生 span。
3. 运行 `popsicle tool run telemetry action=flush`（delivery 阶段实现）或等待后台 batch。
4. 在云端 Collector / Grafana / Honeycomb 中按 `popsicle.run_id` 检索 trace。

## 可观察的成功标志

云端可见带 `popsicle.run_id` resource 的 span；主命令 exit code 不受 exporter 可达性影响。

## Related Next Tasks

- T-TEL-0002

## Charter Compliance

- [x] frontmatter 必填字段齐全
- [x] title 第一人称
- [x] 「本 task 可解答」3 问句
- [x] 完成路径无大量 if-else
