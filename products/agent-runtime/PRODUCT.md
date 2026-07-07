# Product: agent-runtime

> **Layer**: L2（用户可见行为）
> **Audience**: PM、IDD 开发者、平台维护者、AI agent
> **Status**: P9 交付（PROJ-95 Intake Chat）+ P8 交付（PROJ-92 Expo + PROJ-90 桌面 UI）
> **Last-Updated**: 2026-07-07
> **Last-Decision-Ref**: PDR-002（Proposed）· CADR-001

## 一行用途

手机（或远程 Web）派活 + 本机 Daemon 自动跑 `popsicle` pipeline 并调起 Agent CLI；**P9** 起支持 Issue 之前的 **需求 Chat Intake**（实时澄清 → 确认草案 → 本机 `issue create` + pipeline）。Server 只协调队列与会话，不执行 AI、不持有 API Key。

## 用户视角的入口

- **Daemon**：`popsicle daemon start|status|logs`（cli-ux 接缝，ADR 候选）
- **桌面 UI**：Settings → Agent Runtime（Server URL、cursor-agent login、daemon 状态）；Issue 详情 → **远程派活**
- **手机 App（Expo）**：`apps/mobile/` — 设置 / **需求 Chat（P9）** / 派活 / 进度 / Run 详情批准（T-AR-0002–0004、T-AR-0007–0008）
- **派活**：Mobile App / 桌面 UI / Server API `POST /v1/dispatch`
- **进度**：WebSocket 订阅 run/stage 事件（`GET /v1/ws`）
- **审批**：Mobile 对 `requires_approval` stage 下发 `stage complete --confirm`

**Deferred（spec 阶段）**：App Store / TestFlight 商店包、Multica 桥接。

## Problem Statement

**Current Situation**：IDD pipeline 依赖用户在 Cursor 手动执行 `popsicle pipeline next`；离开桌面后 run 停滞。

**Proposed Solution**：agent-runtime 提供 Task Queue + 本机 Daemon；Daemon subprocess 调用 workspace 内 `popsicle` 并按 stage skill 调 Agent CLI（首期 Cursor Agent）。

**Business Impact**：单开发者 + 本机 Agent 可并行推进 spec/实现，手机承担派活与审批控制面。

`Decision-Ref: PDR-001`

## Success Metrics

| Metric | Baseline | Target | Measurement |
|---|---|---|---|
| 派活进入 running | 无 | 首次 dogfood 成功 1 次 | Server task 状态机 |
| P0 pipeline 无人值守完成 | 0 | ≥1 条 fix-regression 或短链 | PROJ dogfood run |
| Daemon 离线可诊断 | 无 | T-AR-0005 路径可走通 | 故障 task 演练 |

`Decision-Ref: PDR-001`

## User Intents Catalog

| User Query | → Task | Journey |
|---|---|---|
| 「怎么装 Daemon？」 | T-AR-0001 | onboarding |
| 「怎么第一次在手机上派活？」 | T-AR-0002 | onboarding |
| 「手机上怎么一句话澄清需求（Chat）？」 | T-AR-0007 | onboarding |
| 「Chat 聊完怎么自动建 Issue 开 pipeline？」 | T-AR-0008 | daily-ops |
| 「进度在哪看？」 | T-AR-0003 | daily-ops |
| 「怎么手机上点批准？」 | T-AR-0004 | daily-ops |
| 「Agent 没动怎么办？」 | T-AR-0005 | troubleshooting |
| 「怎么自托管 Server？」 | T-AR-0006 | admin |

## Tasks Catalog

- [Onboarding](tasks/onboarding/) — Daemon 安装、首次派活、**需求 Chat**（3 task）
- [Daily-Ops](tasks/daily-ops/) — 进度查看、远程审批、**Chat bootstrap 立项**（3 task）
- [Troubleshooting](tasks/troubleshooting/) — 派活失败诊断（1 task）
- [Admin](tasks/admin/) — 自托管 Server（1 task）
- [Lifecycle](tasks/lifecycle/) — [TBD]

详见 [`tasks/README.md`](tasks/README.md)。

## Intents Catalog

- [`intents/acceptance.intent`](intents/acceptance.intent) — 派活/Runtime 注册验收种子（PDR-001）
- [`intents/invariants.intent`](intents/invariants.intent) — 密钥本机、IDD-only 派活不变量种子

## Committed Roadmap

- PDR-001：agent-runtime MVP spec [Accepted 2026-07-06]
- CADR-001：charter IDD 专用派活边界 [Accepted 2026-07-06]
- P7：Tauri 桌面 UI 远程派活 + Runtime 设置（PROJ-90）
- P8：Expo 手机 App 派活/进度/审批（PROJ-92）
- P9：Mobile 需求 Chat Intake + bootstrap 立项（PROJ-94 spec → feature-delivery）

## Open Questions

- Squad 多 Agent 委派（P2；见 arch-debate）

---

> 修订本文件遵循 [`docs/CHARTER.md`](../../docs/CHARTER.md) 第 3 条铁律：必须引用 Decision ID。
