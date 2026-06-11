---
task_id: T-CU-0003
slug: doc-artifact-command
title: "我创建查看和校验 stage 文档"
journey_stage: daily-ops
audience: ["ai-coding-agent"]
task_type: 操作指南
decision_ref: PDR-001
last_updated: 2026-06-09
last_verified: ~
intent_kind: produce
involved_features: ["doc", "guard", "artifact-system"]
prerequisites:
  - "已有 active pipeline run"
limits:
  - "checklist 独立命令不保留"
related_intents:
  - "acceptance.intent#DocCommandWritesArtifact"
related_next_tasks:
  - T-CU-0004
  - T-CU-0005
fact_cite:
  - "docs/baseline/2026-06-08/api-contracts.md § doc"
---

# 我创建查看和校验 stage 文档

## 本 task 可解答

- "doc create 会把 artifact 写到哪里？"
- "doc show 应该返回哪些文档字段？"
- "checklist 还要单独命令吗？"

## 前提与限制

你需要已有 active run。`checklist` 单独命令被裁剪，校验走 `doc check`。

## 完成路径

1. 运行 `popsicle doc create <skill> --title <title> --run <run> --format json`。
2. CLI 返回 `id`、`doc_type`、`file_path`、`status`。
3. 运行 `popsicle doc show <id>` 查看 frontmatter + body。
4. 运行 `popsicle doc check <id>` 或 stage complete 触发 guard。

## 可观察的成功标志

artifact 文件存在，document row 存在，guard 结果可诊断。

## Related Next Tasks

- T-CU-0004
- T-CU-0005

## Charter Compliance

- [x] frontmatter 必填字段齐全
- [x] 文件长度 ≤ 150 行
- [x] title 是完整第一人称句子
- [x] 「本 task 可解答」3 个用户原话问句
- [x] 完成路径无 if-else 分支
- [x] Related Next Tasks ≥ 1
