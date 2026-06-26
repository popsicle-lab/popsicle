# Product: telemetry

> **Layer**: L2（用户可见行为）
> **Audience**: PM、工程师、AI agent、平台维护者
> **Status**: MVP 已交付（PROJ-68）
> **Last-Updated**: 2026-06-26
> **Last-Decision-Ref**: ADR-002

## 一行用途

Agent 观测旁路：OpenTelemetry WAL 本地缓冲 + OTLP 异步云端导出；与编排主平面 fail-open 解耦。

## 用户视角的入口

- **配置**：`.popsicle/otel.yaml` 或 `~/.popsicle/otel.yaml` 设置 OTLP 端点（T-TEL-0001）。
- **Agent 上报**：`popsicle tool run telemetry action=guide`（init 后权威指南）→ `action=record ...`（T-TEL-0002）；monorepo spec 见 [`AGENT_TELEMETRY.md`](AGENT_TELEMETRY.md)、[`SPAN_SCHEMA.md`](SPAN_SCHEMA.md)
- **Run 报告**：`popsicle tool run telemetry action=report run=<run_id>`（T-TEL-0004）；schema 见 [`REPORT_SCHEMA.md`](REPORT_SCHEMA.md)
- **编排观测**：`issue start` / `doc create` / `stage complete` / `doc check` 自动 best-effort span（T-TEL-0003）。
- **数据面**：`.popsicle/telemetry/{run_id}/` WAL；不进 `state.db`、不进 `doc check`。

## Tasks Catalog

- [Admin](tasks/admin/) — OTLP 配置与上云验证（T-TEL-0001）
- [Daily-Ops](tasks/daily-ops/) — Agent 上报、编排 span、run 报告（T-TEL-0002–0004）
- [Onboarding](tasks/onboarding/) — [TBD]
- [Troubleshooting](tasks/troubleshooting/) — [TBD]
- [Lifecycle](tasks/lifecycle/) — [TBD]

详见 [`tasks/README.md`](tasks/README.md)。

## Intents Catalog

- [`intents/acceptance.intent`](intents/acceptance.intent) — 4 block（PDR-001/002）
- [`intents/invariants.intent`](intents/invariants.intent) — FailOpen 不变量种子
- [`intents/contracts.intent`](intents/contracts.intent) — TelemetrySinkFailOpen（ADR-001）

## Committed Roadmap

- PDR-001：Agent 观测旁路产品定案（Accepted；Phase A–C，PROJ-67–72；guide 链 PROJ-75）
- PDR-002：run 报告与 weekly 反馈闭环（Accepted；spec PROJ-73，MVP report PROJ-74）
- ADR-001：OTel WAL / OTLP / TelemetrySink 端口（Accepted）
- ADR-002：Run 聚合报告 / `action=report`（Accepted）

## Open Questions

- MCP `telemetry.record` 是否与 `tool run` 并存（P2 产品决策；见 [`ARCHITECTURE.md`](ARCHITECTURE.md) P2）

---

> 修订本文件遵循 [`docs/CHARTER.md`](../../docs/CHARTER.md) 第 3 条铁律：必须引用 Decision ID。
