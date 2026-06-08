---
artifact: adr-finalization-report
slug: {slug}
generated_by: adr-writer
adr_id: ADR-{id}
target_product: {target_product}
finalized: false             # true 表示本次已把 ADR 固化为 Accepted
last_updated: {YYYY-MM-DD}
goals_unlocked: 0
handoff_to_spec_writer: 0
query_anchors:
  - "这个 ADR 固化了吗？审批人是谁？"
  - "哪些契约被解锁、可以收紧成 intent 了？"
  - "这个决策有没有偷改 charter？"
---

# ADR 固化报告 — {slug}

> 由 `adr-writer` 生成。adr-writer 是 contracts.intent 闭环里「解锁」这一棒——
> **不发明 ADR 内容**（那是 rfc-writer），只做固化门 + 解锁 + 一致性核对。

## Summary

| 指标 | 值 |
|---|---|
| ADR | ADR-{id} |
| 固化结果 | Accepted / 退回 rfc-writer |
| 解锁 goal 数 | 0 |
| 移交 intent-spec-writer 的逻辑保证 | 0 |
| CADR 风险 | 无 / 已标记退回 |

一句话结论：……

## 固化检查

> 固化门五项。任一不过 → 退回 rfc-writer，不强行固化。

- [ ] **决策无歧义**：§ Decision 现在时、明确，无「将会/计划/视情况」
- [ ] **Consequences 落地**：每个文件路径真实可落地（已存在 / 本次创建）
- [ ] **Intent Impact 一致**：与 RFC § Intent & Decision Mapping + contracts 种子 goal 对应
- [ ] **CADR 合规**：未触及 charter 四铁律 / Layer Map（触及则标记并退回）
- [ ] **Decision Context 充分**：触发因素 + 辩论摘要 + 备选否决理由齐全

不过项说明：……

## 解锁动作

> 本次实际改动，一行一处。

| 文件 | 改动 |
|---|---|
| ADR-{id}-{slug}.md | Status Proposed → Accepted；填 Approval |
| {slug}.contracts-unlocked.intent | goal "{名}" 注释 [Awaiting ADR-{id}] → [ADR-{id} Accepted] |

### 移交 intent-spec-writer 的收紧工单

> 现在可以收紧的逻辑保证（contracts 契约的前后置 → acceptance/invariants）。

| 契约 goal | 可收紧为 | 目标文件 | 形态 |
|---|---|---|---|
| "{goal 名}" | {intent 名} | acceptance.intent | require/ensure |
| "{goal 名}" | {safety 名} | invariants.intent | safety + primed |

## Intent Impact 核对

> 与 ADR § Intent Impact 表逐行对照，确认无遗漏、无多余。

| Intent 层 | block | ADR 声明 | 实际解锁 | 一致？ |
|---|---|---|---|---|
| contracts.intent | {goal} | 解锁+收紧 | 已解锁 | ✅ |
| invariants.intent | {safety} | 新增 | 移交 spec-writer | ✅ |

---

## 检查清单

- [ ] 固化门五项已逐项核对
- [ ] ADR Status 已改 Accepted、审批信息已填（或已退回并说明）
- [ ] contracts 种子 [Awaiting] 已解锁为 [Accepted]
- [ ] 已列出移交 intent-spec-writer 的收紧工单
- [ ] 未自行收紧 contracts（职责单一）
- [ ] Intent Impact 核对无遗漏 / 无多余
