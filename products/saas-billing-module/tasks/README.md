# saas-billing-module — Tasks 索引

> **Status**: spec verified（PDR-001；intent-check 25/25 verified）
> **Last-Updated**: 2026-06-10

5 个固定旅程阶段。缺一不可，也不允许第 6 个。

| 旅程阶段 | 任务数 | 状态 | 健康度 |
|---|---:|---|---|
| `onboarding/` | 1 | proposed | ✅ intent verified（last_verified 2026-06-10） |
| `daily-ops/` | 3 | proposed | ✅ intent verified（last_verified 2026-06-10） |
| `troubleshooting/` | 1 | proposed | ✅ intent verified（last_verified 2026-06-10） |
| `admin/` | 1 | proposed | ✅ intent verified（last_verified 2026-06-10） |
| `lifecycle/` | 1 | proposed | ✅ intent verified（last_verified 2026-06-10） |

## Task 清单（PDR-001）

| Task | 旅程 | 标题 | acceptance/invariant |
|---|---|---|---|
| T-BILL-0001 | onboarding | 我创建一个可销售的订阅 plan | PlanCreationCapturesBillingMetadata |
| T-BILL-0002 | daily-ops | 我给客户开通或变更订阅 | SubscriptionStatusChangeAudited |
| T-BILL-0003 | daily-ops | 我查看并确认一张 invoice 的金额来源 | InvoiceTotalBalances |
| T-BILL-0004 | troubleshooting | 我处理一次支付失败并看到重试状态 | PaymentFailureRetryVisible |
| T-BILL-0005 | daily-ops | 我给客户发放并抵扣 credit | CreditApplicationWithinBalance |
| T-BILL-0006 | lifecycle | 我为审计导出某个客户的计费事件链 | BillingAuditTrailExportable |
| T-BILL-0007 | admin | 我配置默认支付重试策略 | PaymentFailureRetryVisible |

## 命名约定

task 文件命名：`<journey-stage>/T-BILL-XXXX-<slug>.md`。
每个 task 文件必须带 YAML frontmatter、query anchors、Related Next Tasks 和 Decision-Ref。

Decision-Ref: PDR-001

Intent-Check: `.popsicle/artifacts/5efd402a-9bac-4d5e-8e5b-970657514ce4/saas-billing-intent-consistency-report.intent-consistency-report.md`
