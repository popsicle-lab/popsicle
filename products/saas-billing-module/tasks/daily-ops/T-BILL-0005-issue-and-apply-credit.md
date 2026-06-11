---
task_id: T-BILL-0005
slug: issue-and-apply-credit
title: "我给客户发放并抵扣 credit"
journey_stage: daily-ops
audience: ["billing-admin", "finance"]
task_type: 操作指南
decision_ref: PDR-001
last_updated: 2026-06-10
last_verified: 2026-06-10
intent_kind: operate
involved_features: ["credits", "invoices", "audit-trail"]
prerequisites:
  - "customer exists"
limits:
  - "approval workflow is pending product decision"
related_intents:
  - "invariants.intent#CreditApplicationWithinBalance"
  - "acceptance.intent#CreditApplicationAudited"
related_next_tasks:
  - T-BILL-0003
  - T-BILL-0006
fact_cite:
  - "Product Brief"
---

# 我给客户发放并抵扣 credit

## 本 task 可解答

- "怎么给客户发 credit？"
- "credit 能不能超过 invoice 金额？"
- "credit 用完后怎么追踪剩余额度？"

## 前提与限制

Customer 存在，并且 billing admin 提供 credit reason。审批流程暂不在首版固定。

## 完成路径

1. Billing admin 创建 credit，填写 reason 和 amount。
2. 系统在 invoice 上应用不超过 remaining balance 的 credit。
3. Credit remaining balance 和 audit event 更新。

## 可观察的成功标志

Credit applied amount 不超过 remaining balance。

## Related Next Tasks

- T-BILL-0003
- T-BILL-0006

Decision-Ref: PDR-001
