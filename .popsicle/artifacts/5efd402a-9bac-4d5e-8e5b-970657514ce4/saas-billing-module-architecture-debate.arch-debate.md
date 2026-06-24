---
id: ac780ae7-1b56-4e55-b949-a2f110e8b173
doc_type: arch-debate-record
title: SaaS billing module architecture debate
status: final
skill_name: arch-debate
pipeline_run_id: 5efd402a-9bac-4d5e-8e5b-970657514ce4
spec_id: 03848550-ac4b-474a-9d9a-3a05ad68c2e7
version: 1
parent_doc_id: null
tags: []
metadata: null
created_at: 2026-06-10T09:55:22.080590Z
updated_at: 2026-06-10T09:58:15.058030Z
---

---
artifact: arch-debate-record
slug: saas-billing-module-architecture-debate
topic: "Define SaaS billing module boundaries, event ledger, PSP adapter, and tax-ready audit contracts"
participants: [ARCH, SEC, PERF, OPS, DATA, DEV]
confidence: 4
input_mode: greenfield-architecture-brief
date: 2026-06-10
query_anchors:
  - "SaaS billing module 的模块边界为什么这样定？"
  - "为什么选择 billing event ledger 而不是直接改 invoice？"
  - "PSP 和 tax service 的责任边界在哪里？"
---

# 架构辩论纪要 — saas-billing-module-architecture-debate

> 由 `arch-debate` skill 生成。本纪要是技术决策的审计轨迹，供 rfc-writer /
> adr-writer 追溯论据，也供后人理解「当时为什么这么选」。

## Topic

Define SaaS billing module boundaries, event ledger, PSP adapter, and tax-ready audit contracts.

来源：`saas-billing-module-prd-task-graph.prd.md` § Intent Mapping row 9:
Plan, Subscription, Invoice, Payment, Credit, and Audit boundaries stay stable.

## Participants

| 角色 | 立场速写 |
|---|---|
| ARCH | 主张模块边界以 domain aggregate + port contract 固化 |
| SEC | 主张所有金额变化走 append-only event，避免静默篡改 |
| PERF | 主张 ledger 写入路径简单，读模型可异步投影 |
| OPS | 主张 PSP/tax 通过 adapter 隔离，失败可重试可观测 |
| DATA | 主张 invoice/credit/payment 共享 billing event ledger |
| DEV | 主张首版避免微服务拆分，先做模块化单体边界 |

用户置信度：4/5

## Phase 1 — 技术问题 + 质量属性（NFR）

- 要解决的问题：定义 greenfield billing module 的内部模块边界和对外 contracts，支持 PRD 中的 invoice total、credit balance、paid invoice adjustment、payment retry、tax-ready audit trail。
- 硬约束：
  - 不把 PSP 具体 API 写入核心 domain。
  - 不把 tax rate calculation 写入 billing core。
  - Paid invoice 不被直接静默修改；调整必须形成 adjustment + audit event。
  - 所有金额变化可追溯到 actor、reason、source object。
- 质量属性优先级：
  1. Correctness / auditability
  2. Evolvability / clear contracts
  3. Operability / retry observability
  4. Performance / read-model scalability
- 事实基引用：PRD Overview + Product Brief。PSP、tax service、storage、SLA 均为 `[待验证]`。

## Phase 2 — 方案发散

- **方案 A: Modular monolith + append-only billing event ledger**（ARCH）
  - PlanCatalog、Subscription、Invoice、Payment、Credit、AuditTrail 是同一 product 内的 domain modules。
  - 所有 amount-changing command 追加 BillingEvent，再投影出 invoice/payment/credit 读模型。
  - PSP 和 tax 作为 adapter ports，不进入 core invariants。

- **方案 B: Service-per-domain + integration events**（OPS）
  - Plan/Subscription/Invoice/Payment/Credit/Audit 拆成独立服务，通过消息队列异步通信。
  - 适合高规模团队，但 greenfield 首版运维成本高。

- **方案 C: Invoice-centric CRUD core**（DEV）
  - 以 invoice table 为中心，subscription/payment/credit 直接更新 invoice 状态。
  - 实现最快，但 auditability 和 invariants 风险最大。

## Phase 3 — 多角色评审

| 方案 | SEC | PERF | OPS | DATA | DEV |
|---|---|---|---|---|---|
| A | 强，append-only 降低篡改风险 | 中高，写路径简单，读模型可投影 | 中，adapter 可观测 | 强，event 是统一事实源 | 中，模块化单体可控 |
| B | 强，但跨服务权限面更大 | 中，异步一致性复杂 | 弱，首版运维重 | 中，事件 schema governance 重 | 弱，交付成本高 |
| C | 弱，静默更新风险高 | 高，直接读写简单 | 中，组件少 | 弱，历史追溯难 | 强，最快 |

## Phase 4 — 收敛与决策

- ARCH 综合：选择方案 A。它把 billing correctness 放在核心，同时避免首版拆微服务。方案 B 的 service split 推迟到后续规模化 ADR；方案 C 被否，因为它无法自然支撑 audit-first invariant baseline。
- 角色投票：
  - ARCH: 支持 A
  - SEC: 支持 A
  - PERF: 支持 A，要求读模型可投影
  - OPS: 支持 A，要求 adapter retry/observability 进 RFC
  - DATA: 支持 A
  - DEV: 支持 A，要求首版模块化单体而非独立服务
- 用户最终决策：按用户“继续推进”指令，作为 dogfood run 批准进入 rfc-writer。

## Decision

SaaS billing module 使用 modular monolith 边界，核心模块通过 append-only BillingEvent ledger 记录所有金额变化。PSP 和 tax service 作为 adapter ports 接入，核心 domain 只依赖稳定 contracts 和 tax-ready metadata。

## Intent & Decision Mapping

| 核心技术声明 | 目标 intent 层 | 决策载体 | 备注 |
|---|---|---|---|
| Billing core records amount-changing operations as append-only BillingEvent | `contracts.intent` / `invariants.intent` | ADR-001 | 解锁 contracts seed |
| Invoice read model is derived from line items, tax amount, applied credits, and adjustments | `contracts.intent` | ADR-001 | 支撑 InvoiceTotalBalances |
| Credit application is a command that appends event and updates projected remaining balance | `contracts.intent` | ADR-001 | 支撑 CreditApplicationWithinBalance |
| PSP integration is an adapter port and cannot mutate invoices directly | `contracts.intent` | ADR-001 | PSP behavior `[待验证]` |
| Tax service is an adapter port; billing core stores tax-ready fields and supplied tax amount but does not calculate tax rates | `contracts.intent` | ADR-001 | tax engine `[待验证]` |

## 关键分歧

- **模块化单体 vs service-per-domain**：OPS 认为服务拆分更清晰，DEV/DATA 认为 greenfield 首版应先固化 domain contracts；收敛为模块化单体，未来通过 ADR 拆服务。
- **event ledger 是否过重**：DEV 担心实现复杂度，SEC/DATA 强调 auditability；收敛为 append-only event ledger + 简单投影，不做完整 accounting ledger。
- **tax boundary**：RISK/FINOPS 在 PRD 中要求 tax-ready；ARCH 明确 tax-ready 不等于 tax-compliant，税率计算留给 adapter。

## 用户决策点

- [x] 用户决策是否覆盖了多数角色意见？否。多数角色支持方案 A。

## 下游接驳建议

- rfc-writer：把本纪要 + rfc-draft 打磨成正式 RFC + contracts seed + ADR-001 骨架。
- adr-writer：固化 ADR-001 后，解锁 `contracts.intent` 收紧。
- intent-spec-writer：把 acceptance/invariants/contracts seeds 统一收紧。

## Output Checklist

- [x] Phase 1-4 小结齐全
- [x] 关键分歧与各方立场已记录
- [x] 用户决策点已显式记录（含覆盖情况）
- [x] 每个数字/模块名引用可追溯到事实基（greenfield：PRD/Product Brief）
- [x] Topic 与另两份 artifact 一致
