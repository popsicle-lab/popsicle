# intent-spec 报告 — skill-runtime（formal acceptance + invariants）

> 由 intent-spec-writer 生成。承上启下：把 prd-writer 的 acceptance 种子 + adr-writer 的
> contracts-unlocked 工单收紧成可合并、可被 Z3 验证的正式 intent-lang。
> 关联：PDR-002 / ADR-002（Accepted 2026-06-08）。

## Summary

| 指标 | 值 |
|---|---|
| acceptance 操作 intent（trivial verified）| 5 → 落地 5（4 收紧 + Skill 类型扩展）|
| invariants safety（真实 VC）| 1（`ApprovedBeforeCompleted`，HC-2 审批闸）|
| 降级/剥离到 task（D2）| 2（schema 向后兼容、read-only 不变性）|
| skipped（intent-lang 能力边界）| 1（audit-trail 可重建，需聚合）|
| 类型冲突 | 1 已修复（enum 变体重名 → 加 Stage*/Run* 前缀，见 § 去重/冲突检查）|
| intent check 实跑 | ⚠️ 未跑——本环境无 intent 二进制，**静态审查**；实跑留 intent-consistency-check Z3 闸 |

## 分层归位

| 来源声明 | 目标层 | 形态 | 说明 |
|---|---|---|---|
| T-0001 PipelineBootstrapsToFirstPause | acceptance.intent | intent | 操作后置规约（trivial verified）|
| T-0002 StageAdvanceWithApproval | acceptance.intent + invariants.intent | intent + safety 绑定 | acceptance 留 trivial 副本；invariants 同名副本受 `ApprovedBeforeCompleted` 守护 → HC-2 真验证 |
| T-0004 RecoveredPipelineCanAdvance | acceptance.intent | intent | 操作后置规约 |
| T-0006 UpgradeDoesNotAffectCompletedRuns | acceptance.intent | intent | 受 ADR-002 影响，Skill 类型扩展 pkgVersion/schemaVersion |
| HC-2 审批闸（ADR-002 contracts-unlocked 工单①）| invariants.intent | safety + primed | `ApprovedBeforeCompleted`：`(status'==Completed && requiresApproval') ==> approvedAt'>0` |
| schema 版本独立（contracts-unlocked 工单②）| —（降级）| task 成功标志 + contracts goal | 见 § 剥离清单 |
| T-0003 ReadOnlyCommandsDoNotMutate | —（降级）| task 成功标志 | 见 § 剥离清单 |
| T-0005 AuditTrailIsReconstructible | —（skipped）| theorem（未来）| 见 § 剥离清单 |

## 剥离 / 降级清单（D2 与能力边界）

1. **schema 向后兼容 ⟹ schema_version 不变**（工单②）：判定「向后兼容」是**运行时语义事实**，
   intent-lang 不验运行时事实（D2）。→ **降级**：保留 `contracts.intent` 的 goal `state_machine schema 版本独立于包版本` 作意图登记，
   并写进 T-0006「可观察成功标志」由契约回归测试守护。**不**做成假 intent（避免 require==ensure 的同义反复）。
2. **read-only 命令不改状态**（T-0003）：若做成全局 safety（`ensure 所有字段不变`），按四规则#2 会被附加到
   文件内所有 intent，从而**证伪所有 mutating 操作**。→ **降级**到 T-0003 task「可观察成功标志」
   （`pipeline status` / `skill list` 前后 ledger 哈希不变），不进 `.intent`。
3. **audit-trail 可重建/不可篡改**（T-0005）：需对 artifact **序列**做聚合/历史推理；intent-lang
   无聚合（`count`/`where`）→ 只能写 struct-forall `theorem`，当前会被 **skipped**（仅声明意图，
   等 intent-lang 支持）。本批不落 `.intent`，留 T-0005 task 守护。

## 四规则审查

| 规则 | 审查结果 |
|---|---|
| 1) 后态用 primed `x'` | ✅ 所有 ensure / invariant 后态均 primed |
| 2) 一文件一作用域 | ✅ acceptance.intent 仅操作 intent、无 safety；invariants.intent 的 safety 只与其绑定操作同文件，无 frame 冲突项混入 |
| 3) 无 frame 需显式 `x'==x` | ✅ 4 个 acceptance 块均补 status-机相关 frame（currentStageIndex/totalStages）；`StageAdvanceWithApproval` 额外 frame requiresApproval/approvedAt（HC-2 safety 所需）。其余块未 frame approval 字段——因 acceptance 无 safety、不产生 VC，不影响验证 |
| 4) 纯 require+ensure = trivial verified | ✅ 已区分：acceptance 5 条 trivial verified（操作规约）；invariants `ApprovedBeforeCompleted` 是唯一**真实 VC**（非 trivial）|

> ⚠️ 别被一片 ✅ 误导：acceptance 的「verified」是 trivial（合法可跑、不被证伪）；
> HC-2 真正被证伪/被证明只在 invariants.intent 的 `ApprovedBeforeCompleted` 对 `StageAdvanceWithApproval` 的 VC 上。

## 去重 / 冲突检查

- 复用目标 `acceptance.intent` 既有领域类型（StageStatus / PipelineRunStatus / SkillStatus / Stage / PipelineRun），未重复声明。
- **enum 变体重名（已修复）**：`StageStatus` 与 `PipelineRunStatus` 原共享 `InProgress/Completed/Blocked` 三个裸变体名。
  intent-lang 的 SMT 编码（`smt.rs`）对每个 enum 生成**无前缀**的全局 datatype constructor，且 typeck（`typeck.rs`）按裸名跨 enum 解析 → 同名变体会使 Z3 声明冲突 / 类型解析歧义、`intent check` 失败。
  **修复**：`StageStatus → {StageBlocked,StageReady,StageInProgress,StageCompleted,StageError}`、
  `PipelineRunStatus → {RunPending,RunInProgress,RunCompleted,RunBlocked}`；已同步 acceptance/invariants/增量三文件全部引用。`SkillStatus`（Loaded/Active/Deprecated）无重名，不动。
- **类型变更**：`Skill.version` → `Skill.pkgVersion + schemaVersion`（ADR-002）。已同步唯一引用点
  `UpgradeDoesNotAffectCompletedRuns`（`skOld.version` → `skOld.pkgVersion`）。无残留 `.version` 引用。
- 无 intent 重名冲突（invariants 的 `StageAdvanceWithApproval` 是同名绑定副本，刻意为之，见 § 分层归位）。

## 合并计划

> 已就地合并到 product intents（保持 intent-check stage 可直接运行）：

- [x] `products/skill-runtime/intents/acceptance.intent`：Skill 类型扩展双版本 + 4 块补 frame + T-0006 改 pkgVersion。
- [x] `products/skill-runtime/intents/invariants.intent`：落地 `ApprovedBeforeCompleted` safety + 绑定操作 `StageAdvanceWithApproval`。
- [x] `products/skill-runtime/intents/contracts.intent`：已由 adr-writer 解锁为 `[ADR-002 Accepted]`（本阶段不再改）。
- 增量原件留存：`<run>/...acceptance-formal.intent`（doc 0d21ade7）。

## 验证结果

- **静态审查**：5 acceptance intent + 1 invariants safety + 1 绑定操作，语法符合 intent-lang SPEC（type/enum/intent/safety、primed、`==>`、`require/ensure/invariant`）。
- **实跑**：⚠️ 本环境无 `intent` 二进制（intent-lang 仅在 `legacy/popsicle/vender/`），无法跑 `intent check`/Z3。
  预期：acceptance 全 trivial verified、`ApprovedBeforeCompleted` 对 StageAdvance 的 VC verified、audit-trail theorem skipped、exit 0。
  实跑交 **intent-consistency-check（Z3 闸）** 阶段确认。沿用项目 `[未经 intent-cli 实测]` 惯例标注。

## 检查清单

- [x] 分层归位完成（acceptance / invariants / 降级 / skipped 各有去向）
- [x] D2 约束剥离（schema 兼容、read-only、时延均不进 `.intent`）
- [x] 四规则逐条审查
- [x] 去重查冲突（类型复用 + Skill 变更已同步）
- [x] 合并计划已执行（products/ 三文件就地合并）
- [x] trivial verified vs 真实不变量已在报告显式区分
- [x] 无法实跑 intent check 已如实标注（无二进制）
