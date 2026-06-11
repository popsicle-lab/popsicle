---
task_id: T-BILL-0007
slug: configure-payment-retry-policy
title: "我配置默认支付重试策略"
journey_stage: admin
audience: ["billing-admin"]
task_type: 配置任务
decision_ref: PDR-001
last_updated: 2026-06-10
last_verified: 2026-06-10
intent_kind: configure
involved_features: ["retry-policy", "payments"]
prerequisites:
  - "payment retry is enabled"
limits:
  - "specific PSP behavior is out of scope"
related_intents:
  - "acceptance.intent#PaymentFailureRetryVisible"
related_next_tasks:
  - T-BILL-0004
fact_cite:
  - "Product Brief"
---

# 我配置默认支付重试策略

## 本 task 可解答

- "怎么配置默认重试次数？"
- "不同失败原因能不能不同策略？"
- "修改策略会不会影响已有 failed invoice？"

## 前提与限制

Billing module 已启用 payment retry。不指定具体 PSP。

## 完成路径

1. Billing admin 设置 retry count、interval 和 stop condition。
2. 系统校验策略不违反最大重试边界。
3. 新 payment failure 使用该策略生成 retry schedule。

## 可观察的成功标志

新 payment failure 生成可见且可审计的 retry schedule。

## Related Next Tasks

- T-BILL-0004

Decision-Ref: PDR-001
