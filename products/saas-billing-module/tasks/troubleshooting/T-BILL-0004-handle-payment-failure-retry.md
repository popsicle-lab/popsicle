---
task_id: T-BILL-0004
slug: handle-payment-failure-retry
title: "我处理一次支付失败并看到重试状态"
journey_stage: troubleshooting
audience: ["billing-admin", "support"]
task_type: 故障排查
decision_ref: PDR-001
last_updated: 2026-06-10
last_verified: 2026-06-10
intent_kind: diagnose
involved_features: ["payments", "retry-policy", "audit-trail"]
prerequisites:
  - "invoice has a failed payment attempt"
limits:
  - "PSP error mapping is pending ADR"
related_intents:
  - "acceptance.intent#PaymentFailureRetryVisible"
related_next_tasks:
  - T-BILL-0003
  - T-BILL-0007
fact_cite:
  - "Product Brief"
---

# 我处理一次支付失败并看到重试状态

## 本 task 可解答

- "支付失败后系统会重试几次？"
- "客户和管理员在哪里看到失败原因？"
- "重试成功后 invoice 状态怎么变？"

## 前提与限制

Invoice 出现 payment failed 状态。PSP 错误码映射等待 architecture branch。

## 完成路径

1. Payment failure 被记录到 invoice。
2. 系统生成 retry schedule。
3. Support 查看下一次 retry 时间、失败原因和当前状态。

## 可观察的成功标志

Failure、retry schedule、最终 payment outcome 都有 audit event。

## Related Next Tasks

- T-BILL-0003
- T-BILL-0007

Decision-Ref: PDR-001
