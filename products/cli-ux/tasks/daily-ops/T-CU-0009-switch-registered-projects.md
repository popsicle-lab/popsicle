---
task_id: T-CU-0009
slug: switch-registered-projects
title: "我在多个已注册项目之间切换并让命令指向正确 workspace"
journey_stage: daily-ops
audience: ["ai-coding-agent", "maintainer"]
task_type: 操作指南
decision_ref: PDR-003
last_updated: 2026-06-11
last_verified: ~
intent_kind: configure
involved_features: ["project", "global.json", "--project"]
prerequisites:
  - "至少两个已初始化的 popsicle workspace（各自含 .popsicle/）"
limits:
  - "不覆盖 Tauri UI 内的项目选择器（见 T-CU-0010）"
related_intents:
  - "acceptance.intent#ProjectRegistryOverridesWorkspace"
related_next_tasks:
  - T-CU-0010
  - T-CU-0002
fact_cite:
  - "docs/baseline/2026-06-11/cli-ux-global/golden-001-project-registry.sh"
  - "products/cli-ux/decisions/adr/ADR-016-ui-project-switcher.md § Context"
---

# 我在多个已注册项目之间切换并让命令指向正确 workspace

## 本 task 可解答

- "我有好几个 popsicle 项目，怎么登记并切换？"
- "`popsicle issue list` 读的是哪个目录？"
- "能不能只对某次命令指定另一个项目路径？"

## 前提与限制

各项目须已 `popsicle init`。本 task 只讲 CLI 侧 `global.json` 注册表，UI 切换见 T-CU-0010。

## 完成路径

1. 对每个 workspace 执行 `popsicle project add <path> --name <alias> --format json`。
2. 执行 `popsicle project use <alias> --format json` 设置默认项目。
3. 运行 `popsicle issue list --format json` 确认针对默认 workspace 成功。
4. 对另一项目执行 `popsicle issue list --project <other-path> --format json` 验证 `--project` 覆盖。
5. 用 `popsicle doctor --format json` 查看 `workspace_root` 与 `global_config_path`。

## 可观察的成功标志

- `~/.popsicle/global.json`（或 `POPSICLE_HOME`）含 `projects` 与 `default_project`。
- 默认命令与 `--project` 覆盖均返回 `status: ok` 且指向预期根目录。

## Related Next Tasks

- T-CU-0010
- T-CU-0002

## Charter Compliance

- [x] frontmatter 必填字段齐全
- [x] 文件长度 ≤ 150 行
- [x] title 是完整第一人称句子
- [x] 「本 task 可解答」3 个用户原话问句
- [x] 完成路径无 if-else 分支
- [x] Related Next Tasks ≥ 1
