---
id: c4aeb8c7-3c9c-4e88-8a6e-f2e0e15d0b2a
doc_type: prd-overview
title: SaaS billing module PRD task graph
status: final
skill_name: prd-writer
pipeline_run_id: 5efd402a-9bac-4d5e-8e5b-970657514ce4
spec_id: 03848550-ac4b-474a-9d9a-3a05ad68c2e7
version: 1
parent_doc_id: null
tags: []
metadata: null
created_at: 2026-06-10T09:43:26.303341Z
updated_at: 2026-06-10T09:47:53.727503Z
---

# PRD Overview — SaaS billing module task graph

> **Status**: Approved for dogfood PRD stage
> **Target Product**: `saas-billing-module`
> **Source Debate**: `saas-billing-module-product-debate.product-debate.md`
> **Input Mode**: `greenfield-product-brief`
> **Fact Basis**: `Product Brief`
> **PDR**: `PDR-001-saas-billing-module-task-graph.md`
> **Quality Score**: 92/100
> **Last-Updated**: 2026-06-10

## Core Intent

SaaS teams manage plan catalog, subscriptions, invoices, payment retries, credits, and tax-ready audit trails through task-sized docs, with billing invariants prepared for intent verification.

## Problem Statement

A greenfield billing module can easily become a feature list or a payment-provider integration note. This PRD instead defines the user task graph and the money-movement rules that must remain verifiable: invoice totals balance, credits cannot over-apply, paid invoices are adjusted rather than silently edited, and billing events remain auditable.

`Decision-Ref: PDR-001` | `Fact: Product Brief`

## Success Metrics

| Metric | Baseline | Target | Measurement | Cite |
|---|---|---|---|---|
| Task graph coverage | 0 | 7 tasks across all 5 journey stages | `products/saas-billing-module/tasks/README.md` | Product Brief |
| Intent seed coverage | 0 | 7 acceptance blocks + 3 invariants | intent seed files | Product Brief |
| Audit traceability | Undefined | Every amount-changing task references audit event expectations | task checklist + intent mapping | Product Brief |

`Decision-Ref: PDR-001`

## File Manifest

| File | Action |
|---|---|
| `products/saas-billing-module/PRODUCT.md` | Create |
| `products/saas-billing-module/tasks/README.md` | Create |
| `products/saas-billing-module/tasks/onboarding/T-BILL-0001-create-sellable-plan.md` | Create |
| `products/saas-billing-module/tasks/daily-ops/T-BILL-0002-open-or-change-subscription.md` | Create |
| `products/saas-billing-module/tasks/daily-ops/T-BILL-0003-confirm-invoice-amount-source.md` | Create |
| `products/saas-billing-module/tasks/troubleshooting/T-BILL-0004-handle-payment-failure-retry.md` | Create |
| `products/saas-billing-module/tasks/daily-ops/T-BILL-0005-issue-and-apply-credit.md` | Create |
| `products/saas-billing-module/tasks/lifecycle/T-BILL-0006-export-billing-audit-trail.md` | Create |
| `products/saas-billing-module/tasks/admin/T-BILL-0007-configure-payment-retry-policy.md` | Create |
| `products/saas-billing-module/intents/acceptance.intent` | Create draft formal seed |
| `products/saas-billing-module/intents/invariants.intent` | Create draft invariant seed |
| `products/saas-billing-module/decisions/pdr/PDR-001-saas-billing-module-task-graph.md` | Create skeleton |

## User Intents Catalog

| User Query | Task | Journey Stage | Audience |
|---|---|---|---|
| "怎么创建一个新的订阅 plan？" | T-BILL-0001 | onboarding | billing-admin |
| "plan 需要哪些价格和税务字段？" | T-BILL-0001 | onboarding | billing-admin |
| "怎么给客户开通订阅？" | T-BILL-0002 | daily-ops | billing-admin |
| "invoice total 是怎么算出来的？" | T-BILL-0003 | daily-ops | finance |
| "支付失败后系统会重试几次？" | T-BILL-0004 | troubleshooting | support |
| "怎么给客户发 credit？" | T-BILL-0005 | daily-ops | finance |
| "怎么查看客户的完整计费事件？" | T-BILL-0006 | lifecycle | auditor |
| "怎么配置默认重试次数？" | T-BILL-0007 | admin | billing-admin |

## Intent Mapping

| # | Core Statement | Target Intent Layer | Task | Block |
|---|---|---|---|---|
| 1 | Invoice total equals subtotal + tax - applied credits. | `invariants.intent` | T-BILL-0003 | `InvoiceTotalBalances` |
| 2 | A credit cannot be applied above its remaining balance. | `invariants.intent` | T-BILL-0005 | `CreditApplicationWithinBalance` |
| 3 | Paid invoice changes require adjustment and audit event. | `invariants.intent` | T-BILL-0003 / T-BILL-0006 | `PaidInvoiceAdjustmentOnly` |
| 4 | Plan creation exposes sellable and tax-ready metadata. | `acceptance.intent` | T-BILL-0001 | `PlanCreationCapturesBillingMetadata` |
| 5 | Subscription status changes record actor, reason, timestamp, and source object. | `acceptance.intent` | T-BILL-0002 | `SubscriptionStatusChangeAudited` |
| 6 | Payment failure creates visible retry schedule and status. | `acceptance.intent` | T-BILL-0004 / T-BILL-0007 | `PaymentFailureRetryVisible` |
| 7 | Credit issue/application updates remaining balance and audit trail. | `acceptance.intent` | T-BILL-0005 | `CreditApplicationAudited` |
| 8 | Audit export returns customer billing event chain with tax-ready fields. | `acceptance.intent` | T-BILL-0006 | `BillingAuditTrailExportable` |
| 9 | Plan, Subscription, Invoice, Payment, Credit, and Audit boundaries stay stable. | `contracts.intent` | all | `BillingModuleContracts` (ADR candidate) |

## Out of Tasks

- PSP-specific integration is out of scope.
- Tax-rate calculation engine is out of scope.
- Revenue recognition and accounting journal entries are out of scope.
- Metered usage rating and multi-currency settlement are out of scope.

`Decision-Ref: PDR-001`

## Risk Assessment

| Risk | Probability | Impact | Mitigation | Affected Tasks | Fact Cite |
|---|---|---|---|---|---|
| Tax-ready is mistaken for tax-compliant | Medium | High | Explicitly keep tax calculation out of scope | T-BILL-0001, T-BILL-0006 | Product Brief |
| Credit rules allow over-application | Medium | High | Seed invariant before implementation | T-BILL-0005 | Product Brief |
| Retry policy couples to a PSP too early | Medium | Medium | Keep PSP mapping in ADR branch | T-BILL-0004, T-BILL-0007 | [待验证] |
| Contracts freeze too early | Medium | Medium | Send boundary decisions to arch-debate | all | [待验证] |

`Decision-Ref: PDR-001`

## Dependencies & Blockers

- `arch-debate` / `rfc-writer` must define module contracts before `contracts.intent` is tightened.
- PSP behavior, tax service behavior, SLA, and persistence model are `[待验证]` until architecture branch.

## Quality Score

| Dimension | Score | Notes |
|---|---:|---|
| Completeness | 19/20 | Covers all requested product areas |
| Clarity | 18/20 | Greenfield assumptions marked |
| Testability | 14/15 | Acceptance and invariant seeds present |
| AI Digestibility | 19/20 | Task files have frontmatter and query anchors |
| IDD Fit | 22/25 | Contracts intentionally deferred to ADR branch |
| **Total** | **92/100** | Pass |

## Ingest Checklist

- [x] prd-draft 已读取，已通过 task-centric 形态校验
- [x] debate-record 已读取
- [x] Product Brief 已记录为 greenfield fact basis
- [x] target_product 已由 product brief 显式声明
- [x] target_product 的 5 个 journey stage 目录已创建
- [x] PDR ID 已分配（PDR-001）
- [x] Task ID 范围已分配（T-BILL-0001..T-BILL-0007）

## Quality Checklist

- [x] File Manifest 与 PDR Consequences 完全一致
- [x] 每个新增 task 文件单独存在且符合 task 模板
- [x] User Intents Catalog 覆盖每个新增 task
- [x] Intent Mapping 与 acceptance/invariant seed block 一一对应
- [x] 无历史/未来叙事短语
- [x] greenfield 判断引用 Product Brief 或标 `[待验证]`
- [x] Quality Score ≥ 90

## Review Checklist

- [x] prd-overview 的 File Manifest 与 PDR Consequences 完全一致
- [x] 每个 task 文件路径符合 `products/saas-billing-module/tasks/<journey-stage>/...`
- [x] 每个 task 文件单独检查：frontmatter / 长度 / Related Next Tasks 都过关
- [x] acceptance.intent / invariants.intent 种子的 block 名与 task_id / invariant 映射一致
- [x] tasks/README.md 列出所有新增 task
- [x] PRD 质量评分 ≥ 90
- [x] 5 类 artifact 的 `target_product` 一致
- [x] 已向用户展示五类完整产出（本 dogfood run 以 artifact + product files 形式展示）
