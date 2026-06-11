---
task_id: T-CU-0010
slug: ui-open-project-recents
title: "我用桌面 UI 打开项目并在欢迎页看到最近项目"
journey_stage: onboarding
audience: ["end-user", "maintainer"]
task_type: 操作指南
decision_ref: PDR-003
last_updated: 2026-06-11
last_verified: ~
intent_kind: produce
involved_features: ["ui", "ProjectPicker", "ProjectSwitcher", "global.json"]
prerequisites:
  - "已构建带 ui feature 的 Popsicle.app 或 popsicle ui"
limits:
  - "不含 Apple 签名 / Gatekeeper 绕过（ADR-016 D-603）"
related_intents:
  - "acceptance.intent#UiProjectOpenPersistsRecents"
related_next_tasks:
  - T-CU-0009
  - T-CU-0002
fact_cite:
  - "products/cli-ux/decisions/adr/ADR-016-ui-project-switcher.md"
  - "docs/baseline/2026-06-11/cli-ux-project-ui/"
---

# 我用桌面 UI 打开项目并在欢迎页看到最近项目

## 本 task 可解答

- "双击 Popsicle.app 为什么以前只打印帮助？现在该怎么进界面？"
- "桌面版怎么选项目、看最近打开过的仓库？"
- "UI 里换项目后，CLI 的 global.json 会不会一起更新？"

## 前提与限制

需要 Tauri UI 构建（`cargo build -p cli-ux --features ui` 或 DMG 内的 `.app`）。未签名 DMG 的安装限制不变。

## 完成路径

1. 从 Finder 双击 `Popsicle.app`（无参数）应直接进入 UI 而非 CLI help。
2. 在欢迎页 `ProjectPicker` 点击「浏览」选含 `.popsicle/` 的目录，或点最近项目卡片。
3. 进入主界面后，侧栏 `ProjectSwitcher` 显示当前项目并可切换已注册项。
4. 关闭并重开应用，最近列表仍包含刚打开的项目（`last_opened_at` 已写入 `global.json`）。
5. 可选：在终端 `popsicle doctor --format json` 确认 `workspace_root` 与 UI 一致。

## 可观察的成功标志

- `.app` 零参数启动打开窗口。
- `~/.popsicle/global.json` 中对应 `ProjectEntry.last_opened_at` 更新。
- 侧栏与欢迎页列出已注册项目（最多 12 条 MRU）。

## Related Next Tasks

- T-CU-0009
- T-CU-0002

## Charter Compliance

- [x] frontmatter 必填字段齐全
- [x] 文件长度 ≤ 150 行
- [x] title 是完整第一人称句子
- [x] 「本 task 可解答」3 个用户原话问句
- [x] 完成路径无 if-else 分支
- [x] Related Next Tasks ≥ 1
