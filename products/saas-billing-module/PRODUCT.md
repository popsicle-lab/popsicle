# Product: saas-billing-module

> **Layer**: L2（用户可见行为）
> **Audience**: PM、平台工程、计费运营、财务、AI Copilot
> **Status**: spec verified（greenfield dogfood；ADR-001 accepted）
> **Last-Updated**: 2026-06-10
> **Last-Decision-Ref**: PDR-001 / ADR-001

## 一行用途

提供一个可嵌入 SaaS 产品的 billing module，覆盖 plan、subscription、invoice、payment retry、credit 和 tax-ready audit trail。

## 用户视角的入口

- Billing admin 创建可销售 plan。
- Billing admin 给 customer 开通、变更、取消 subscription。
- Finance 查看 invoice total 的金额来源。
- Support 处理 payment failure 并解释 retry 状态。
- Finance 发放/抵扣 credit。
- Auditor 导出 customer billing event chain。

## Tasks Catalog

- [Onboarding](tasks/onboarding/) — 创建第一个可销售 plan（1 task）
- [Daily-Ops](tasks/daily-ops/) — 订阅、发票、credit 日常操作（3 tasks）
- [Troubleshooting](tasks/troubleshooting/) — 支付失败与重试诊断（1 task）
- [Admin](tasks/admin/) — 默认重试策略配置（1 task）
- [Lifecycle](tasks/lifecycle/) — 审计导出与生命周期追溯（1 task）

详见 [`tasks/README.md`](tasks/README.md)。

## Intents Catalog

- [`intents/acceptance.intent`](intents/acceptance.intent) — billing task acceptance seed。
- [`intents/invariants.intent`](intents/invariants.intent) — money movement invariants seed。
- [`intents/contracts.intent`](intents/contracts.intent) — ADR-001 解锁的 module boundary contracts（当前 goal-only / 0 VC）。

## Committed Roadmap

- PDR-001：greenfield billing task graph + first invariant baseline。
- ADR-001：Plan / Subscription / Invoice / Payment / Credit / Audit 模块边界已 Accepted。

## Open Questions

- PSP 错误码和 retry strategy 的映射方式。
- Tax service 的职责边界。
- Proration、multi-currency、usage rating 是否进入后续 PDR。

---

> 修订本文件遵循 [`docs/CHARTER.md`](../../docs/CHARTER.md) 第 3 条铁律：必须引用 Decision ID。
