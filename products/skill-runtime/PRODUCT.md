# Product: skill-runtime

> **Layer**: L2（用户可见行为）
> **Audience**: PM、销售、客户成功、AI Copilot
> **Status**: lib 切流完成（ADR-005）；CLI 入口待 cli-ux slice
> **Last-Updated**: 2026-06-09
> **Last-Decision-Ref**: ADR-005（Accepted 2026-06-09）

## 一行用途

Skill 状态机 + Pipeline DAG + Run/Spec ledger + Hook 总线 + Tool/Memory 注册表 + Advisor —— intent-coder 私有引擎的灵魂。

## 用户视角的入口

[TBD: needs archaeology]

## Tasks Catalog

> 5 个旅程阶段的入口（v0.2 任务图）。具体 task 文件由 prd-writer 写到 `tasks/<stage>/` 下。

- [Onboarding](tasks/onboarding/) — 首次接触 → 首次成功（[TBD]）
- [Daily-Ops](tasks/daily-ops/) — 日常使用（[TBD]）
- [Troubleshooting](tasks/troubleshooting/) — 故障排查（[TBD]）
- [Admin](tasks/admin/) — 管理类（[TBD]）
- [Lifecycle](tasks/lifecycle/) — 终止 / 迁出 / 续费（[TBD]）

详见 [`tasks/README.md`](tasks/README.md)。

## Intents Catalog

- [`intents/invariants.intent`](intents/invariants.intent) — product 自然律（已填：HC-2 审批闸 `ApprovedBeforeCompleted`；intent-check Z3 verified）
- [`intents/contracts.intent`](intents/contracts.intent) — 模块 API 契约（已填：2 goal 块，ADR-002 Accepted）
- [`intents/acceptance.intent`](intents/acceptance.intent) — 验收契约（已填：4 intent，intent-check Z3 全 verified）

## Committed Roadmap

[TBD: needs archaeology + product-debate]

## Open Questions

- [TBD: needs archaeology] —— 由 fact-extractor 的 Risk Hotspots 填初稿；不允许 AI 编造

---

> 本文件是首切片骨架；下游 skill 执行顺序：
> `fact-extractor` → `product-debate` → `prd-writer` 三步后会填充本表所有 `[TBD]`。
> 修订本文件遵循 [`docs/CHARTER.md`](../../docs/CHARTER.md) 第 3 条铁律：必须引用 Decision ID。
