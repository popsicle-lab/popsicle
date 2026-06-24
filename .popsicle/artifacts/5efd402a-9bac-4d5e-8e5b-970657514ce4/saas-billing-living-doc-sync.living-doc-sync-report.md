---
id: a6e0f97e-1a63-4b3f-91d2-b9dca0809f04
doc_type: living-doc-sync-report
title: SaaS billing living doc sync
status: final
skill_name: living-doc-author
pipeline_run_id: 5efd402a-9bac-4d5e-8e5b-970657514ce4
spec_id: 03848550-ac4b-474a-9d9a-3a05ad68c2e7
version: 1
parent_doc_id: null
tags:
- saas-billing-module
- living-docs
- greenfield-product-spec
metadata: null
created_at: 2026-06-10T10:21:52.245467Z
updated_at: 2026-06-10T10:25:22.018593Z
---

---
artifact: living-doc-sync-report
slug: saas-billing-module
generated_by: living-doc-author
target: all
last_updated: 2026-06-10
docs_scanned: 15
drift_signals: 9
docs_refreshed: 10
manual_followups: 2
query_anchors:
  - "SaaS billing 的活文档过期了吗？"
  - "哪些 task 已经被 intent-check 验证？"
  - "tasks 索引和产品状态刷新了吗？"
---

# 活文档保活报告 — saas-billing-module

> 由 `living-doc-author` skill 生成。只对账与刷新活文档元数据，不创作正文、不改业务逻辑。

## Summary

| 指标 | 值 |
|---|---:|
| target | all |
| 扫描文档数 | 15 |
| 发现 drift 信号 | 9 |
| 本次刷新文档数 | 10 |
| 待人工处置项 | 2 |
| 结论 | 已自动对齐可安全刷新的 metadata/header；剩余 2 项进入后续 PDR/intent-spec |

一句话结论：SaaS billing 的 task、intent、PDR、ADR、PRODUCT、ARCHITECTURE 已和 intent-check 结果对齐；本阶段不进入实现，只留下 contract 收紧和 delivery scope 两个后续事项。

## Scan Checklist

- [x] target 已确认
- [x] 所有 task / intent / PDR / ADR / PRODUCT.md / ARCHITECTURE.md 已枚举
- [x] 四类 drift 信号已逐条核对，证据已记录
- [x] 已区分可自动刷新的元数据与需 PDR 的正文改动

## Drift 信号

### 1. 过期 staleness

| 文档 | 证据 | 严重度 |
|---|---|---|
| （无） | 所有 saas-billing-module 文档均为 2026-06-10 创建或更新 | — |

### 2. 断链 broken-ref

| 来源 | 失效引用 | 类型 |
|---|---|---|
| （无） | 7 个 task 的 `related_intents` 均能在 `acceptance.intent` 或 `invariants.intent` 找到对应 intent 名称 | — |

### 3. 孤儿 orphan

| 对象 | 情况 |
|---|---|
| （无） | 5 条 acceptance intent 与 3 条 invariant intent 均被 task 索引或 task frontmatter 引用；7 个 task 均在 `tasks/README.md` 列出 |

### 4. 未验证 unverified

| Task | 当前 last_verified | 报告中的状态 | 可回填？ |
|---|---|---|---|
| T-BILL-0001 | `~` | `PlanCreationCapturesBillingMetadata` verified（2026-06-10）| 是 |
| T-BILL-0002 | `~` | `SubscriptionStatusChangeAudited` verified（2026-06-10）| 是 |
| T-BILL-0003 | `~` | `InvoiceTotalBalances` / `PaidInvoiceAdjustmentOnly` verified（2026-06-10）| 是 |
| T-BILL-0004 | `~` | `PaymentFailureRetryVisible` verified（2026-06-10）| 是 |
| T-BILL-0005 | `~` | `CreditApplicationWithinBalance` / `CreditApplicationAudited` verified（2026-06-10）| 是 |
| T-BILL-0006 | `~` | `BillingAuditTrailExportable` verified（2026-06-10）| 是 |
| T-BILL-0007 | `~` | `PaymentFailureRetryVisible` verified（2026-06-10）| 是 |

### 5. 状态头 drift

| 文档 | 证据 | 可刷新？ |
|---|---|---|
| `PRODUCT.md` | 仍描述 `contracts.intent` 待 arch-debate / rfc-writer 后补，但 ADR-001 已 Accepted 且 contracts seed 已存在 | 是 |
| `ARCHITECTURE.md` | 顶部仍为 `Proposed`，但 ADR-001 已 Accepted | 是 |

## 刷新动作

| 文件 | 改动 |
|---|---|
| `products/saas-billing-module/tasks/onboarding/T-BILL-0001-create-sellable-plan.md` | `last_verified: ~` -> `2026-06-10` |
| `products/saas-billing-module/tasks/daily-ops/T-BILL-0002-open-or-change-subscription.md` | `last_verified: ~` -> `2026-06-10` |
| `products/saas-billing-module/tasks/daily-ops/T-BILL-0003-confirm-invoice-amount-source.md` | `last_verified: ~` -> `2026-06-10` |
| `products/saas-billing-module/tasks/troubleshooting/T-BILL-0004-handle-payment-failure-retry.md` | `last_verified: ~` -> `2026-06-10` |
| `products/saas-billing-module/tasks/daily-ops/T-BILL-0005-issue-and-apply-credit.md` | `last_verified: ~` -> `2026-06-10` |
| `products/saas-billing-module/tasks/lifecycle/T-BILL-0006-export-billing-audit-trail.md` | `last_verified: ~` -> `2026-06-10` |
| `products/saas-billing-module/tasks/admin/T-BILL-0007-configure-payment-retry-policy.md` | `last_verified: ~` -> `2026-06-10` |
| `products/saas-billing-module/tasks/README.md` | 健康度从 draft 刷新为 intent verified，并加入 intent-check report 引用 |
| `products/saas-billing-module/PRODUCT.md` | 状态头与 contracts catalog 对齐 ADR-001 / intent-check 现状 |
| `products/saas-billing-module/ARCHITECTURE.md` | 状态头对齐为 ADR-001 boundary accepted / implementation pending |

## 健康度快照

| 旅程阶段 | Task 数 | 平均行数 | 最久未更新 | 未引用 task |
|---|---:|---:|---|---|
| onboarding | 1 | 53 | 2026-06-10 | 无 |
| daily-ops | 3 | 54 | 2026-06-10 | 无 |
| troubleshooting | 1 | 53 | 2026-06-10 | 无 |
| admin | 1 | 51 | 2026-06-10 | 无 |
| lifecycle | 1 | 53 | 2026-06-10 | 无 |

## 待人工处置

| 项 | 原因 | 跟进 |
|---|---|---|
| `contracts.intent` 仍为 goal-only / 0 VC | 当前只有 ADR-001 module boundary goals，尚未写成可验证 safety/contract VC | 后续由 `intent-spec-writer` 在接口稳定后收紧 |
| 首个 delivery slice 未选择 | greenfield-product-spec 只完成 spec 链，不负责实现落地 | 下一条 workflow 建议跑 delivery/slice 类工作流，先选 T-BILL-0001 + T-BILL-0002 作为 MVP |

---

## 检查清单

- [x] 四类 drift 信号都已扫描并列出
- [x] 刷新动作每条都对应真实文件改动
- [x] last_verified 只回填了 verified 的 task
- [x] 健康度快照数字与刷新后的 README 一致
- [x] 所有越界 drift 已转待人工处置，未擅自改正文
- [x] frontmatter 计数与正文一致
