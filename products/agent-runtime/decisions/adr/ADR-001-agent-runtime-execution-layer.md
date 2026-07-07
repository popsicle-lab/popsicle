# ADR-001 · agent-runtime 执行面分层（Server + Daemon + cli-ux 接缝）

> **Status**: Accepted
> **Date**: 2026-07-06
> **Product**: agent-runtime
> **Generated-by**: rfc-writer（PROJ-81）
> **Relates**: PDR-001 · ADR-007 cli-ux IO shell · ADR-001 telemetry fail-open

## Context

PDR-001 要求手机远程派活与本机 Daemon 执行 popsicle + Agent CLI。arch-debate（doc-185–187）否决「全塞进 cli-ux」与「Multica 桥接」，选定独立 `agent-server` + `agent-daemon`，cli-ux 仅增 `daemon` 子命令。

## Decision

1. **新建 crate** `agent-server`（协调）与 `agent-daemon`（执行），加入 workspace `crates/*`。
2. **cli-ux** 增加 `popsicle daemon start|stop|status|logs`，实现为 thin wrapper 调用 agent-daemon 库（ADR-007 不变：不吞 pipeline FSM）。
3. **状态真相** 在开发机 `.popsicle/state.db`；Server 存 task/run **镜像 + 事件**（P0 单向同步）。
4. **Prompt 组装** 在 daemon 内读 `intent-coder/skills/*/guide.md`；不等待 `popsicle prompt` deferred 恢复。
5. **首期 Agent adapter**：`cursor-agent`；并发默认 daemon 20 / agent profile 6（可配置）。
6. **CADR-001 候选**（并行）：charter 增补「agent-runtime 为 IDD 专用派活，非通用工作流平台」。

## Alternatives

| 方案 | 否决理由 |
|------|---------|
| 全进 cli-ux | 违反 IO shell 边界，二进制与测试耦合 |
| legacy popsicle-sync | 已砍；语义是 Yjs 非派活 |
| Multica 执行层 | 双 Issue 模型，IDD 门禁不对齐 |

## Consequences

- `products/agent-runtime/ARCHITECTURE.md` — File Manifest 落地
- `products/agent-runtime/intents/contracts.intent` — ADR Accepted 后解锁 goal 日期标注
- `crates/cli-ux/Cargo.toml` — 可选依赖 agent-daemon
- `Cargo.toml` workspace members — 增 agent-server、agent-daemon
- `deploy/agent-runtime/` — Podman Compose（compose.yaml）P0

## Compliance

| 门禁 | 证据 | 结果 |
|---|---|---|
| arch-debate | doc-185–187 | pass |
| contracts 种子 | contracts.intent | 待 Accepted 后 intent-spec |

## File Manifest

| Path | Change |
|------|--------|
| `crates/agent-server/` | 新增 |
| `crates/agent-daemon/` | 新增 |
| `crates/cli-ux/src/daemon.rs` | 新增 |
| `products/agent-runtime/ARCHITECTURE.md` | 已起草 |
| `products/agent-runtime/intents/contracts.intent` | 种子 |

## Approval

- **Status**: Accepted
- **Approved by**: PROJ-81 pipeline（delegate-dangerous）
- **Approval date**: 2026-07-06
