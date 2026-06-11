---
task_id: T-BILL-0006
slug: export-billing-audit-trail
title: "我为审计导出某个客户的计费事件链"
journey_stage: lifecycle
audience: ["finance", "auditor"]
task_type: 操作指南
decision_ref: PDR-001
last_updated: 2026-06-10
last_verified: 2026-06-10
intent_kind: inspect
involved_features: ["audit-trail", "tax-ready-fields"]
prerequisites:
  - "customer has billing events"
limits:
  - "tax filing format is out of scope"
related_intents:
  - "acceptance.intent#BillingAuditTrailExportable"
related_next_tasks:
  - T-BILL-0003
  - T-BILL-0005
fact_cite:
  - "Product Brief"
---

# 我为审计导出某个客户的计费事件链

## 本 task 可解答

- "怎么查看客户的完整计费事件？"
- "某次 invoice total 的来源能追溯到哪些对象？"
- "audit trail 里有哪些 tax-ready 字段？"

## 前提与限制

Customer 有 billing events。税务申报格式和税额计算不属于本 task。

## 完成路径

1. Finance 按 customer 和时间范围查询 audit trail。
2. 系统返回 plan、subscription、invoice、payment、credit events。
3. Finance 导出带 actor、reason、source object、tax-ready fields 的事件链。

## 可观察的成功标志

任一金额变化都能追溯到 source object 和 actor。

## Related Next Tasks

- T-BILL-0003
- T-BILL-0005

Decision-Ref: PDR-001
