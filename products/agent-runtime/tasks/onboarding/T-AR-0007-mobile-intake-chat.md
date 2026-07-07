---
task_id: T-AR-0007
slug: mobile-intake-chat
title: "我在手机上用 Chat 澄清需求并看到 Agent 实时回复"
journey_stage: onboarding
audience: ["end-user", "new-user"]
task_type: 操作指南
decision_ref: PDR-002
last_updated: 2026-07-07
intent_kind: operate
involved_features: ["chat-api", "chat-turn-queue", "websocket", "cursor-agent"]
prerequisites:
  - "runtime.online(T-AR-0001)"
  - "user.has(mobile_app_configured)"
related_intents:
  - "acceptance.intent#ChatTurnQueuedWhenRuntimeOnline"
  - "acceptance.intent#ChatTurnRejectedWhenRuntimeOffline"
related_next_tasks:
  - T-AR-0008
  - T-AR-0003
---

# 我在手机上用 Chat 澄清需求并看到 Agent 实时回复

## 本 task 可解答

- 怎么在手机上用一句话开始需求澄清（还没有 PROJ-xx）？
- Chat 和「派活页填 Issue Key」有什么区别？
- Agent 回复为什么必须等 Daemon online？

## 前提与限制

Runtime 已 online（T-AR-0001）。本阶段 **尚未创建 popsicle Issue**；会话数据存 agent-server（`chat_sessions` / `chat_messages`）。Agent 执行仍在本机 Daemon（CADR-001 / `SecretsStayOnRuntimeMachine`）。Chat 不替代 IDE 内直接改代码——implement 仍进入 pipeline。

## 完成路径

1. **Mobile「需求」Tab** 新建 `chat_session`（绑定 `workspace_id` + `runtime_id` + 可选 `product_id`）。

2. **用户发送消息** → `POST /v1/chat/sessions/{id}/messages` → Server 入队 `chat_turn_task`。

3. **Daemon** claim → 读取会话 history + `issue-author/guide.md` 上下文 → `cursor-agent -p`（stream-json 写 run/chat logs）→ 助手回复存 Server。

4. **Mobile** 经 WebSocket 收到 `chat_message` / `chat_draft_updated`（draft title、pipeline、description 摘要）。

5. Agent 标记 draft ready 后，用户进入 **T-AR-0008** 确认立项。

## 可观察的成功标志

用户发消息后 60s 内（Daemon online、Agent 认证正常）Mobile 出现 assistant 消息；Server `chat_messages` 含 user + assistant 各至少一条。形式化：`acceptance.intent#ChatTurnQueuedWhenRuntimeOnline`；离线时：`ChatTurnRejectedWhenRuntimeOffline`。

## Related Next Tasks

- **T-AR-0008** — 确认草案并创建 Issue 开始 pipeline
- **T-AR-0003** — 立项后在进度 Tab 看 run

`Decision-Ref: PDR-002`
