---
task_id: T-CU-0012
slug: macos-dmg-install-cli
title: "我用 macOS DMG 安装 CLI 并能在终端运行 popsicle"
journey_stage: onboarding
audience: ["end-user", "maintainer"]
task_type: 操作指南
decision_ref: PDR-003
last_updated: 2026-06-11
last_verified: ~
intent_kind: configure
involved_features: ["dmg", "Install CLI.command", "PATH"]
prerequisites:
  - "macOS arm64 或 x86_64 与对应 DMG 构建产物"
limits:
  - "不含代码签名与 notarization（ADR-016 D-603）"
related_intents:
  - "acceptance.intent#MacosDmgInstallExposesCli"
related_next_tasks:
  - T-CU-0011
  - T-CU-0010
fact_cite:
  - "packaging/macos/README.md"
  - "docs/baseline/2026-06-11/cli-ux-global/golden-002-macos-install-script.sh"
---

# 我用 macOS DMG 安装 CLI 并能在终端运行 popsicle

## 本 task 可解答

- "Popsicle DMG 里有什么？intent-coder 在不在 DMG 文件夹里？"
- "Install CLI.command 做了什么？"
- "装完后终端找不到 popsicle 怎么办？"

## 前提与限制

DMG 内含 `Popsicle.app`、裸 `popsicle` 二进制与 `Install CLI.command`；intent-coder **在 CLI 二进制内**（ADR-017），不在 DMG 独立目录。

## 完成路径

1. 打开 `Popsicle_<version>_arm64.dmg`，将 `Popsicle.app` 拖入 Applications（可选）。
2. 双击 `Install CLI.command`，按提示把 CLI 安装到 `~/.local/bin` 并配置 PATH。
3. 新开终端窗口，运行 `popsicle doctor --format json` 确认可执行。
4. 在空目录执行 `popsicle init`（见 T-CU-0011）验证嵌入模块解压。
5. 需要 UI 时从 Applications 启动 `Popsicle.app`（见 T-CU-0010）。

## 可观察的成功标志

- `which popsicle` 指向 `~/.local/bin/popsicle`（或 Install 脚本声明的路径）。
- `doctor` 返回 `status: ok` 且 `executable_path` 为已安装二进制。

## Related Next Tasks

- T-CU-0011
- T-CU-0010

## Charter Compliance

- [x] frontmatter 必填字段齐全
- [x] 文件长度 ≤ 150 行
- [x] title 是完整第一人称句子
- [x] 「本 task 可解答」3 个用户原话问句
- [x] 完成路径无 if-else 分支
- [x] Related Next Tasks ≥ 1
