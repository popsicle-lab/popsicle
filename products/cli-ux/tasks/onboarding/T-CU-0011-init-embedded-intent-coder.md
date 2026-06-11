---
task_id: T-CU-0011
slug: init-embedded-intent-coder
title: "我在全新目录 init 后自动获得 intent-coder 模块"
journey_stage: onboarding
audience: ["ai-coding-agent", "end-user"]
task_type: 操作指南
decision_ref: PDR-003
last_updated: 2026-06-11
last_verified: ~
intent_kind: produce
involved_features: ["init", "intent-coder", "admin sync-intent-coder"]
prerequisites:
  - "已安装 popsicle CLI（含 ADR-017 嵌入 bundle）"
limits:
  - "dogfood 仓库根存在 intent-coder/ 时优先复制工作区版本，非嵌入路径"
related_intents:
  - "acceptance.intent#InitInstallsEmbeddedIntentCoder"
related_next_tasks:
  - T-CU-0001
  - T-CU-0002
fact_cite:
  - "products/cli-ux/decisions/adr/ADR-017-intent-coder-embedded-bundle.md"
  - "crates/cli-ux/tests/intent_coder_install.rs"
---

# 我在全新目录 init 后自动获得 intent-coder 模块

## 本 task 可解答

- "DMG 装完 popsicle，新项目还要单独 clone intent-coder 吗？"
- "`popsicle init` 会把 skill 装到哪？"
- "`popsicle module add` 不能用，怎么更新模块？"

## 前提与限制

适用于无仓库根 `intent-coder/` 的普通项目。monorepo dogfood 仍从工作区根同步 live 模块。

## 完成路径

1. `mkdir my-app && cd my-app`（空目录，无 `intent-coder/` 兄弟目录）。
2. 运行 `popsicle init`，输出应提及 intent-coder 版本与 `.popsicle/modules/intent-coder` 路径。
3. 确认 `.popsicle/modules/intent-coder/skills/` 与 `tools/intent-validate/tool.yaml` 存在。
4. 运行 `popsicle doctor --format json`，`intent_coder_module` 有版本、`intent_coder_bundle` 为 `embedded`。
5. 需要强制刷新时运行 `popsicle admin sync-intent-coder --format json`。

## 可观察的成功标志

- 无需 `popsicle module add`（deferred）即可 `doc create` 使用 intent-coder skill 名。
- `popsicle tool run intent-validate path=products` 能找到模块内 tool 定义。

## Related Next Tasks

- T-CU-0001
- T-CU-0002

## Charter Compliance

- [x] frontmatter 必填字段齐全
- [x] 文件长度 ≤ 150 行
- [x] title 是完整第一人称句子
- [x] 「本 task 可解答」3 个用户原话问句
- [x] 完成路径无 if-else 分支
- [x] Related Next Tasks ≥ 1
