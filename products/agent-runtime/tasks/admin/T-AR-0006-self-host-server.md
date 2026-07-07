---
task_id: T-AR-0006
slug: self-host-server
title: "我自托管 agent-runtime Server 并连接手机 App"
journey_stage: admin
audience: ["admin"]
task_type: 配置任务
decision_ref: PDR-001
last_updated: 2026-07-06
intent_kind: configure
involved_features: ["self-host-server", "device-auth"]
prerequisites:
  - "admin.has(podman_or_equivalent)"
related_intents: []
related_next_tasks:
  - T-AR-0001
fact_cite:
  - "doc-181 § 方案 A P0 自托管 Docker"
---

# 我自托管 agent-runtime Server 并连接手机 App

## 本 task 可解答

- Server 必须自托管吗能用手机派活吗？
- Podman Compose 最小集包含什么？
- 手机和 Daemon 怎么认证同一 workspace？

## 前提与限制

P0 提供 Podman 自托管路径；Multica Cloud 桥接不在本 task。PostgreSQL + API + WS（PROJ-88）。

## 完成路径

1. 部署 Server（示例，具体 manifest 见 arch-debate ADR）：

   ```bash
   # macOS 首次：podman machine init && podman machine start
   ./deploy/agent-runtime/up.sh
   ```

2. 配置 `AGENT_RUNTIME_SERVER_URL` 于 Daemon 与 Mobile。

3. 设备登录 / token 绑定 workspace。

4. 跑 T-AR-0001 注册 Runtime，再 T-AR-0002 验证派活。

## 可观察的成功标志

Mobile 与 Daemon 均显示同一 workspace online；派活端到端成功一次。

## Related Next Tasks

- **T-AR-0001** — 注册 Runtime

`Decision-Ref: PDR-001`
