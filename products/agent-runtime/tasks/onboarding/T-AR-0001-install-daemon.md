---
task_id: T-AR-0001
slug: install-daemon
title: "我安装并启动本机 Daemon 让 Runtime 上线"
journey_stage: onboarding
audience: ["end-user", "admin"]
task_type: 操作指南
decision_ref: PDR-001
last_updated: 2026-07-06
intent_kind: configure
involved_features: ["popsicle-daemon", "runtime-registration"]
prerequisites:
  - "user.installed(popsicle, >= self-host MVP)"
  - "user.has(workspace_with_.popsicle)"
related_intents:
  - "acceptance.intent#RuntimeRegistersWhenDaemonStarts"
related_next_tasks:
  - T-AR-0002
fact_cite:
  - "doc-181.product-debate.md § Decision"
---

# 我安装并启动本机 Daemon 让 Runtime 上线

## 本 task 可解答

- 怎么让开发机在 agent-runtime 里显示为 online Runtime？
- `popsicle daemon start` 和 Cursor 里的 Agent 是什么关系？
- Daemon 离线时派活会怎样？

## 前提与限制

本机已 `popsicle init` 且 `popsicle doctor` 通过。Daemon 探测 PATH 中的 Agent CLI（首期 `cursor-agent`）。API Key 仅在本机 Agent CLI 使用，不上传 Server。

## 完成路径

1. **启动 Daemon**（任选其一）：

   **桌面 UI（推荐）**：Settings → Agent Runtime → **启动 Daemon**（自动保存 Server URL 并后台运行）。

   **终端**：

   ```bash
   popsicle daemon start --background
   # 或前台调试：popsicle daemon start --foreground
   ```

2. **确认状态**：

   ```bash
   popsicle daemon status --format json
   ```

   可观察：`status: online`，且列出已注册 Runtime（daemon × cursor-agent × workspace）。

3. **在 Server / Mobile 设置页** 确认该 Runtime 行状态为 **online**（心跳 ≤45s `[假设]`）。

## 可观察的成功标志

`daemon status` 返回 online；Server Runtimes 列表出现对应行且可接受派活。形式化：`acceptance.intent#RuntimeRegistersWhenDaemonStarts`。

## Related Next Tasks

- **T-AR-0002** — 首次手机派活

`Decision-Ref: PDR-001`
