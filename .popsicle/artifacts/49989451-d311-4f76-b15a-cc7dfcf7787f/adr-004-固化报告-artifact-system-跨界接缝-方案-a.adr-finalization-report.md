---
id: cdab83c3-e44c-4572-82e7-bfabcc138d0e
doc_type: adr-finalization-report
title: ADR-004 固化报告：artifact-system 跨界接缝（方案 A）
status: final
skill_name: adr-writer
pipeline_run_id: 49989451-d311-4f76-b15a-cc7dfcf7787f
spec_id: cf6e7062-b75f-4925-b633-82a04e775a76
version: 1
parent_doc_id: null
tags: []
metadata: null
created_at: 2026-06-09T06:18:59.470282Z
updated_at: 2026-06-09T06:24:57.878296Z
---

# ADR-004 固化报告 — artifact-system 跨界接缝（方案 A）

> **ADR**: [`ADR-004-artifact-system-seams.md`](../../products/artifact-system/decisions/adr/ADR-004-artifact-system-seams.md)
> **Source-RFC**: RFC-004 `fecff724`
> **Source-Debate**: arch-debate `befaaaae`（方案 A）

## Summary

| 指标 | 值 |
|---|---|
| ADR | ADR-004 |
| 固化结果 | Accepted |
| 解锁 goal 数 | 3（guard 端口 / total / ContextLayer 注册）|
| 移交 intent-spec-writer 的逻辑保证 | 3（GuardResultIsTotal / ContextOrderIndependentOfRegistration / TaskChunkRenamePreservesFields）|
| CADR 风险 | 无（仅两 product 间内部接缝，不触 charter 四铁律 / 全局 Layer Map）|

一句话结论：ADR-004 已固化为 Accepted——artifact-system 采 hexagonal 方案 A，guard upstream 端口仅含 artifact-owned 类型且回传 GuardResult、ContextLayer 运行时注册 + 确定性装配序、DocumentRow 下沉 storage、extractor total 化；contracts.intent 3 goal 已解锁 [ADR-004 Accepted 2026-06-09]。

## 固化检查

> 固化门五项。任一不过 → 退回 rfc-writer，不强行固化。

- [x] **决策无歧义**：§ Decision 现在时、明确（端口签名/排序键/下沉位置/total 化均确定），无「将会/计划/视情况」
- [x] **Consequences 落地**：contracts.intent 3 goal 解锁标记 + ADR-004 文件本次真实落地（见 § 解锁动作）；invariants/ARCHITECTURE/crate 标为下游 stage 落地物
- [x] **Intent Impact 一致**：RFC § Intent & Decision Mapping 的 contracts 行与本次解锁的 3 goal 逐行一致（见 § Intent Impact 核对）
- [x] **CADR 合规**：仅 artifact-system/skill-runtime 内部接缝，未触 charter 四铁律 / 全局 Layer Map
- [x] **Decision Context 充分**：触发（product-debate C + arch-debate A）+ 5 角色辩论 + B/C 否决理由 + rubber-duck 2 blocking 修正齐全

不过项说明：无（五项全过）。

## 解锁动作

> 本次实际改动，一行一处。

| 文件 | 改动 |
|---|---|
| `products/artifact-system/decisions/adr/ADR-004-artifact-system-seams.md` | 新建，Status: Accepted，填 Approval（@curtiseng，2026-06-09）|
| `products/artifact-system/intents/contracts.intent` | 3 个 goal 的 `[ADR-004 候选]` → `[ADR-004 Accepted 2026-06-09]`；Status 头注改「已解锁」|

### 移交 intent-spec-writer 的收紧工单

> 现在可以收紧的逻辑保证（contracts 契约的前后置 → acceptance/invariants）。

| 契约 goal | 可收紧为 | 目标文件 | 形态 |
|---|---|---|---|
| guard 端口回传 GuardResult + 缺省 InvalidSkillDef 不 panic | invariant `GuardResultIsTotal` | invariants.intent | 任意 guard 字符串 → Ok(GuardResult) 或 InvalidSkillDef，∀ 输入不 panic |
| ContextLayer 注册 + 确定性装配序 | invariant `ContextOrderIndependentOfRegistration` | invariants.intent | 同组 layer 任意注册序 → 同一装配序（排序键唯一决定）|
| （承接 PDR-001）task_chunk 重命名保字段 | invariant `TaskChunkRenamePreservesFields` | invariants.intent | 重命名前后 kind/fields 逐键相等 |

## Intent Impact 核对

> 与 ADR § Consequences / RFC § Intent & Decision Mapping 逐行对照，确认无遗漏、无多余。

| Intent 层 | block / goal | ADR 声明 | 实际解锁 | 一致？ |
|---|---|---|---|---|
| contracts.intent | guard upstream 端口 | 定义且解锁 | [ADR-004 Accepted] | ✅ |
| contracts.intent | guard/extractor 全函数 | 定义且解锁 | [ADR-004 Accepted] | ✅ |
| contracts.intent | ContextLayer 注册 + 确定性序 | 定义且解锁 | [ADR-004 Accepted] | ✅ |
| invariants.intent | GuardResultIsTotal | 移交 intent-spec | 工单已列 | ✅（下游）|
| invariants.intent | ContextOrderIndependentOfRegistration | 移交 intent-spec | 工单已列 | ✅（下游）|
| invariants.intent | TaskChunkRenamePreservesFields | 移交 intent-spec | 工单已列 | ✅（下游）|
| acceptance.intent | 4 PRD 种子 block | 不在本 ADR 范围（PDR-001 产）| 无改动 | ✅ |

---

## 检查清单

- [x] 固化门五项已逐项核对
- [x] ADR Status 已改 Accepted、审批信息已填（@curtiseng，2026-06-09，经 stage complete --confirm 兑现）
- [x] contracts 种子 [ADR-004 候选] 已解锁为 [ADR-004 Accepted 2026-06-09]（3 goal）
- [x] 已列出移交 intent-spec-writer 的收紧工单（3 个 invariant）
- [x] 未自行收紧 contracts（职责单一，收紧由 intent-spec-writer 做）
- [x] Intent Impact 核对无遗漏 / 无多余

## Finalization Gate

- [x] Decision 无歧义、现在时
- [x] Consequences 每个文件路径真实可落地（ADR-004 文件 + contracts 解锁本次落地；invariants/ARCHITECTURE/crate 标下游 stage）
- [x] Intent Impact 与 RFC / contracts 种子一致（3 goal 逐行核对）
- [x] 未触及 charter 锁定内容（两 product 内部接缝，非 CADR）
- [x] Decision Context 充分（product-debate C + arch-debate A + 5 角色 + rubber-duck 2 blocking）
