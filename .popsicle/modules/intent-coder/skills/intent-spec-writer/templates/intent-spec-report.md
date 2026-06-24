---
artifact: intent-spec-report
slug: {slug}
generated_by: intent-spec-writer
target_product: {target_product}
source_seed: {slug}.acceptance.intent / {slug}.contracts-unlocked.intent
pdr: PDR-{id}
last_updated: {YYYY-MM-DD}
intents_formalized: 0
constraints_stripped: 0
verify_exit: 0
query_anchors:
  - "种子收紧成正式 intent 了吗？"
  - "哪些约束被剥离到测试了？"
  - "收紧后的 intent 能合并进现有 acceptance 吗？"
---

# Intent Spec 收紧报告 — {slug}

> intent-spec-writer 把 prd-writer 的种子 + adr-writer 的 contracts-unlocked handoff
> 收紧为可合并、可被 Z3 验证的正式 intent-lang。

## Summary

| 指标 | 值 |
|---|---|
| 目标产品 | {target_product} |
| 来源种子 | {slug}.acceptance.intent / {slug}.contracts-unlocked.intent |
| 正式化 intent 数 | 0 |
| 剥离约束数（降级到 task）| 0 |
| `intent check` exit | 0 |
| 结论 | 可合并 / 需修订 |

一句话结论：……

## Ingest Checklist

- [ ] 种子已读，列出其全部 intent / safety / goal / type
- [ ] 目标产品现有三件套已读，已列出现存符号名（去重基线）
- [ ] PRD § Intent Mapping 已读，每条声明目标层已确认
- [ ] 每条种子内容已初步标好归属层

## 分层归位

> 种子里每条内容归到哪一层。invariants / contracts 层给出落点与片段，由后续处理。

| 来源（种子 block / PRD 行）| 归属层 | 落点文件 | 形态 | 备注 |
|---|---|---|---|---|
| ExampleOperation | acceptance | {target_product}/intents/acceptance.intent | intent require/ensure | trivial verified |
| （保持型不变量示例）| invariants | {target_product}/intents/invariants.intent | safety + primed | 需完整 ensure |
| （契约示例）| contracts | {target_product}/intents/contracts.intent | goal / contract intent | ADR Accepted 后可收紧 |

## 剥离的约束

> 违反 D2 的时间/性能/运行时约束，从 .intent 移除并登记去向。没有则写「（无）」。

| 被剥离的约束 | 类型 | 去向（task 成功标志）| 守护手段 |
|---|---|---|---|
| 例：activation ≤ 30s | 时间 | task T-XXXX | e2e 计时断言 |

## 四规则审查

- [ ] 规则 1：所有 safety/invariant 后态用 primed `x'`
- [ ] 规则 2：acceptance 文件不含 safety（不污染作用域）
- [ ] 规则 3：需「不变」的字段已显式 `ensure x' == x`
- [ ] 规则 4：已标注哪些是 trivial verified（操作规约）vs 真验证不变量

## 验证结果

> `intent check` 真实输出。failed 必须逐字抄反例。

```
<intent check 输出粘贴处>
```

- 单文件 exit：0
- 与现有 acceptance.intent 拼接后 exit：0
- verified / skipped / failed：0 / 0 / 0

## 冲突检查

- [ ] 无同名 intent / type 重复声明
- [ ] 无类型字段冲突
- [ ] 无安全规则互相否定
- 冲突明细（如有）：……

## 合并计划

1. 复用现有类型：……（哪些 type 已在目标文件，勿重复声明）
2. 追加位置：products/{target_product}/intents/{acceptance,invariants,contracts}.intent
3. invariants / contracts 增量分别落到对应文件（见分层归位）
4. 合并后跑 intent-consistency-check 做闸

## Goal 追溯（realized_by）

> 合并 acceptance / invariants **之后**回填 `contracts.intent` 每个 goal 的 `realized_by`。

| goal 名 | realized_by（safety / intent 名）| 依据（ADR / PRD 行）|
|---|---|---|
| "{goal 名}" | {SafetyOrIntent}, … | ADR-XXX § … / PRD row N |

- [ ] 每个 goal 的 `realized_by` 非空
- [ ] 引用的符号均在合并后的 `products/{target_product}/intents/` 可解析
- [ ] `popsicle tool run intent-validate path=products/{target_product}/intents` exit 0（含 goal 追溯闸）

## 检查清单

- [ ] formal .intent 单独 intent check 通过（exit 0）
- [ ] 与现有 acceptance.intent 拼接后仍通过
- [ ] invariants.intent / contracts.intent 分别通过或合理 skipped
- [ ] 无时间/性能/运行时约束残留在 .intent
- [ ] 四规则审查全过
- [ ] 无重复声明 / 命名冲突
- [ ] 合并计划写明追加位置与类型复用方式
- [ ] 每个 contracts goal 已回填 `realized_by`（见 Goal 追溯段）
