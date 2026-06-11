---
task_id: T-CU-0001
slug: first-init-next-step
title: "我第一次初始化 popsicle-new 并看到下一步"
journey_stage: onboarding
audience: ["ai-coding-agent", "maintainer"]
task_type: 操作指南
decision_ref: PDR-001
last_updated: 2026-06-09
last_verified: ~
intent_kind: produce
involved_features: ["init", "intent-coder", "pipelines"]
prerequisites:
  - "空仓库或未初始化 workspace"
limits:
  - "不覆盖 cloud sync / Tauri UI"
related_intents:
  - "acceptance.intent#InitShowsNextStep"
related_next_tasks:
  - T-CU-0002
fact_cite:
  - "docs/baseline/2026-06-08/api-contracts.md § popsicle-cli 子命令清单"
---

# 我第一次初始化 popsicle-new 并看到下一步

## 本 task 可解答

- "我第一次跑 popsicle init 后下一步是什么？"
- "intent-coder 模块是怎么进入 workspace 的？"
- "AI agent 怎么知道接下来跑哪个 pipeline？"

## 前提与限制

你需要先有一个本地 repo。此 task 不覆盖 sync/cloud/Tauri。

## 完成路径

1. 运行 `popsicle init` 初始化 `.popsicle/`。
2. 确认 `.popsicle/modules/intent-coder/` 已安装（嵌入 bundle 或 dogfood 工作区覆盖，ADR-017）。
3. 运行 `popsicle doctor --format json` 查看 `intent_coder_module` 与 bundled pipelines。
4. CLI 输出下一步建议，指向 `intent-coder/guides/pipeline-selection.md` 与 `issue create`。

## 可观察的成功标志

初始化后 CLI 返回非空 next step，且 `pipeline` / `tool run intent-validate` 能读取 workspace 状态。

## Related Next Tasks

- T-CU-0002

## Charter Compliance

- [x] frontmatter 必填字段齐全
- [x] 文件长度 ≤ 150 行
- [x] title 是完整第一人称句子
- [x] 「本 task 可解答」3 个用户原话问句
- [x] 完成路径无 if-else 分支
- [x] Related Next Tasks ≥ 1
