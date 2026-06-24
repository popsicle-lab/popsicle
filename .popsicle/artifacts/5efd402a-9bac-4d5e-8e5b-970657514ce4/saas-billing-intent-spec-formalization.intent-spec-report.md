# Intent Spec 收紧报告 — saas-billing-module

> intent-spec-writer 把 PRD 的 acceptance/invariants seed 与 ADR-001 解锁后的 contracts handoff 收紧为可合并、可被 intent-lang/Z3 验证的正式 intent-lang。

## Summary

| 指标 | 值 |
|---|---|
| 目标产品 | saas-billing-module |
| 来源种子 | PRD acceptance/invariants seed + ADR-001 contracts-unlocked |
| acceptance 操作 intent | 5 verified |
| invariants intent | 3 verified |
| contracts goal | 5 goal, 0 VC, parse/typecheck ok |
| 剥离约束数 | 3 |
| `intent check` exit | 0 for all 3 files |
| 结论 | 可进入 intent-consistency-check |

一句话结论：SaaS billing 的三层 intent 文件已就地合并到 `products/saas-billing-module/intents/`，并经 `intent-validate` 标准工具实跑通过。

## Ingest Checklist

- [x] 可用种子已读，列出其全部 intent / safety / goal / type
- [x] 目标产品现有三件套已读，已列出现存符号名（去重基线）
- [x] PRD/RFC Intent Mapping 已读，每条声明的目标层已确认
- [x] 每条种子内容已初步标好归属层（含「降级到 task」的项）

## 分层归位

| 来源 | 归属层 | 落点文件 | 形态 | 备注 |
|---|---|---|---|---|
| Plan creation metadata | acceptance | `acceptance.intent` | intent require/ensure | trivial verified |
| Subscription status audit | acceptance | `acceptance.intent` | intent require/ensure | trivial verified |
| Payment failure retry visibility | acceptance | `acceptance.intent` | intent require/ensure | trivial verified |
| Credit application audited | acceptance | `acceptance.intent` | intent require/ensure | trivial verified |
| Billing audit exportable | acceptance | `acceptance.intent` | intent require/ensure | trivial verified |
| InvoiceTotalBalances | invariants | `invariants.intent` | intent require/ensure | verified |
| CreditApplicationWithinBalance | invariants | `invariants.intent` | intent require/ensure | verified |
| PaidInvoiceAdjustmentOnly | invariants | `invariants.intent` | intent require/ensure | verified |
| Billing core contracts | contracts | `contracts.intent` | goal | ADR-001 Accepted |

## 剥离的约束

| 被剥离的约束 | 类型 | 去向 | 守护手段 |
|---|---|---|---|
| PSP retry code taxonomy | runtime/vendor behavior | T-BILL-0004 / future ADR | adapter tests |
| Tax calculation correctness | external tax service behavior | T-BILL-0001 / T-BILL-0006 | tax adapter tests |
| Projection lag / performance | NFR | RFC Quality Attributes | benchmark/SLO |

## 四规则审查

- [x] 规则 1：所有后态约束使用 primed `x'`
- [x] 规则 2：acceptance 文件不含 safety
- [x] 规则 3：无 frame 需求未伪造；当前种子只声明目标字段后态
- [x] 规则 4：acceptance 操作规约为 trivial verified；不把它误称为强证明

## 验证结果

```text
Checking acceptance.intent...
✅ intent PlanCreationCapturesBillingMetadata — verified
✅ intent SubscriptionStatusChangeAudited — verified
✅ intent PaymentFailureRetryVisible — verified
✅ intent CreditApplicationAudited — verified
✅ intent BillingAuditTrailExportable — verified

Checking invariants.intent...
✅ intent InvoiceTotalBalances — verified
✅ intent CreditApplicationWithinBalance — verified
✅ intent PaidInvoiceAdjustmentOnly — verified

Checking contracts.intent...
exit 0 (goal-only contracts file; 0 VC)
```

- 单文件 exit：0 / 0 / 0
- verified / skipped / failed：8 / 0 / 0

## 冲突检查

- [x] 无同名 intent / type 重复声明
- [x] 无类型字段冲突
- [x] 无 safety 污染 acceptance 作用域
- 冲突明细：无

## 合并计划

1. 已就地合并：`products/saas-billing-module/intents/acceptance.intent`
2. 已就地合并：`products/saas-billing-module/intents/invariants.intent`
3. 已就地合并：`products/saas-billing-module/intents/contracts.intent`
4. 下一步：`intent-consistency-check` 重新枚举并汇总报告

## 检查清单

- [x] formal .intent 单独 intent check 通过（exit 0）
- [x] 与现有 acceptance.intent 拼接后仍通过（已就地文件）
- [x] invariants.intent / contracts.intent 分别通过或合理 0 VC
- [x] 无时间/性能/运行时约束残留在 .intent
- [x] 四规则审查全过
- [x] 无重复声明 / 命名冲突
- [x] 合并计划写明追加位置与类型复用方式
