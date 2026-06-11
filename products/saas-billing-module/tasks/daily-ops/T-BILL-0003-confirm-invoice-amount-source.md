---
task_id: T-BILL-0003
slug: confirm-invoice-amount-source
title: "我查看并确认一张 invoice 的金额来源"
journey_stage: daily-ops
audience: ["billing-admin", "finance"]
task_type: 操作指南
decision_ref: PDR-001
last_updated: 2026-06-10
last_verified: 2026-06-10
intent_kind: inspect
involved_features: ["invoices", "credits", "tax-ready-audit"]
prerequisites:
  - "subscription has generated an invoice"
limits:
  - "tax rate calculation is delegated"
related_intents:
  - "invariants.intent#InvoiceTotalBalances"
  - "invariants.intent#PaidInvoiceAdjustmentOnly"
related_next_tasks:
  - T-BILL-0004
  - T-BILL-0005
fact_cite:
  - "Product Brief"
---

# 我查看并确认一张 invoice 的金额来源

## 本 task 可解答

- "invoice total 是怎么算出来的？"
- "credit 和 tax 在 invoice 上怎么体现？"
- "paid invoice 能不能直接改？"

## 前提与限制

Subscription 已触发 invoice。税率计算由外部 tax service 或后续 ADR 决定。

## 完成路径

1. 系统生成 invoice line items。
2. 系统展示 subtotal、tax、applied credits 和 total due。
3. Finance 查看 breakdown 和相关 audit references。

## 可观察的成功标志

Invoice total 等于 line item subtotal + tax - applied credits。

## Related Next Tasks

- T-BILL-0004
- T-BILL-0005

Decision-Ref: PDR-001
