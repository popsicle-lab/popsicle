---
task_id: T-CU-0013
slug: project-agent-settings
title: "配置本项目 Agent 偏好（语言、产品目录）"
journey_stage: onboarding
audience: ["ai-coding-agent", "maintainer"]
task_type: 操作指南
decision_ref: ADR-019
last_updated: 2026-06-11
last_verified: 2026-06-11
intent_kind: produce
involved_features: ["init", "admin", "ui", "doctor"]
prerequisites:
  - "已 `popsicle init` 或打开已有 workspace"
limits:
  - "不含 legacy `context scan` / `prompt` 全量装配"
related_intents: []
related_next_tasks:
  - T-CU-0002
fact_cite:
  - "products/cli-ux/decisions/adr/ADR-019-per-project-agent-config.md"
---

# 配置本项目 Agent 偏好（语言、产品目录）

## 本 task 可解答

- "Agent 应该用中文还是英文回复？"
- "产品文档目录不是默认 `products/` 时怎么告诉 pipeline？"
- "偏好如何进入 Cursor / AGENTS.md？"

## 完成路径

### CLI / 初始化

1. `popsicle init` 会写入 `.popsicle/project.yaml` 并（默认）同步
   `AGENTS.md` 中的 `<!-- popsicle:project-config -->` 区块。
2. 修改 yaml 后运行 `popsicle admin sync-project-config` 刷新 `AGENTS.md`。
3. `popsicle doctor --format json` 可查看 `agent_language`、`products_dir`。

### 桌面 UI

1. 侧栏打开 **Settings**。
2. 设置语言、产品目录、默认 spec；勾选「同步 AGENTS.md」与「工作流注入」。
3. 点 **保存配置**。

### 工作流注入

- `popsicle issue start` 的 JSON 含 `agent_context`（`inject_on_run: true`）。
- `popsicle doc create` 在 frontmatter 写入 `agent_context`。

## 验证

```bash
docs/baseline/2026-06-11/cli-ux-project-config/run-all.sh
```
