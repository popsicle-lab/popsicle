---
task_id: T-BILL-0001
slug: create-sellable-plan
title: "我创建一个可销售的订阅 plan"
journey_stage: onboarding
audience: ["billing-admin", "platform-engineer"]
task_type: 配置任务
decision_ref: PDR-001
last_updated: 2026-06-10
last_verified: 2026-06-10
intent_kind: configure
involved_features: ["plan-catalog", "tax-ready-metadata"]
prerequisites:
  - "SaaS product has at least one sellable offer"
limits:
  - "tax calculation is out of scope"
related_intents:
  - "acceptance.intent#PlanCreationCapturesBillingMetadata"
related_next_tasks:
  - T-BILL-0002
  - T-BILL-0003
fact_cite:
  - "Product Brief"
---

# 我创建一个可销售的订阅 plan

## 本 task 可解答

- "怎么创建一个新的订阅 plan？"
- "plan 需要哪些价格和税务字段？"
- "下架 plan 后已有订阅会怎样？"

## 前提与限制

产品团队已经定义可售卖套餐。税率计算不属于本 task；这里只保留 tax-ready metadata。

## 完成路径

1. Billing admin 填写 plan name、billing interval、base price 和 tax-ready metadata。
2. 系统校验可销售字段完整。
3. Plan 进入 active 状态并可被 subscription 引用。

## 可观察的成功标志

Active plan 出现在 plan catalog，并能被 T-BILL-0002 选中。

## Related Next Tasks

- T-BILL-0002
- T-BILL-0003

Decision-Ref: PDR-001
