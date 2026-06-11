---
task_id: T-BILL-0002
slug: open-or-change-subscription
title: "我给客户开通或变更订阅"
journey_stage: daily-ops
audience: ["billing-admin", "support"]
task_type: 操作指南
decision_ref: PDR-001
last_updated: 2026-06-10
last_verified: 2026-06-10
intent_kind: operate
involved_features: ["subscriptions", "audit-trail"]
prerequisites:
  - "at least one active plan exists"
limits:
  - "proration rules are pending ADR"
related_intents:
  - "acceptance.intent#SubscriptionStatusChangeAudited"
related_next_tasks:
  - T-BILL-0003
  - T-BILL-0006
fact_cite:
  - "Product Brief"
---

# 我给客户开通或变更订阅

## 本 task 可解答

- "怎么给客户开通订阅？"
- "客户升级或降级 plan 时状态怎么变化？"
- "取消订阅后还会不会继续出账？"

## 前提与限制

至少一个 active plan 存在。Proration 和合同期策略等待架构/产品后续决策。

## 完成路径

1. Billing admin 选择 customer 和 plan。
2. 系统创建或变更 subscription，并记录 actor、reason、timestamp、source object。
3. Subscription 进入 active、scheduled 或 canceled 状态。

## 可观察的成功标志

每次 subscription 状态变化都有 audit event。

## Related Next Tasks

- T-BILL-0003
- T-BILL-0006

Decision-Ref: PDR-001
