---
id: 22fd56b2-b2f8-4191-8112-afd3a3472823
doc_type: living-doc-sync-report
title: skill-runtime 活文档保活报告（intent 锚点对账 + last_verified 回填）
status: final
skill_name: living-doc-author
pipeline_run_id: f89529af-d8ce-40f7-ad05-985e35b9cfec
spec_id: df11b6f9-e504-42b7-8d27-700d529c2346
version: 1
parent_doc_id: null
tags: []
metadata: null
created_at: 2026-06-08T08:24:54.150179Z
updated_at: 2026-06-08T08:29:45.607020Z
---

---
artifact: living-doc-sync-report
slug: skill-runtime-living-doc-sync
generated_by: living-doc-author
target: all
last_updated: 2026-06-08
docs_scanned: 12
drift_signals: 9
docs_refreshed: 8
manual_followups: 7
query_anchors:
  - "skill-runtime 的 task→intent 锚点还对得上吗？"
  - "哪些 task 已被 Z3 verified、回填了 last_verified？"
  - "哪些 intent 引用是 orphan、要走 prd-writer/PDR？"
---

# 活文档保活报告 — skill-runtime-living-doc-sync

> 由 `living-doc-author` skill 生成。**只对账与刷新活文档元数据**（索引 / 反向引用 / last_verified /
> 健康度 / 骨架 catalog 注解），不创作正文、不改业务逻辑——后者走 prd-writer + PDR（charter 铁律 #3）。

## Summary

| 指标 | 值 |
|---|---|
| target | all |
| 扫描文档数 | 12（6 task + 3 intent + PRODUCT.md + tasks/README.md + intent-consistency-report）|
| 发现 drift 信号 | 9 |
| 本次刷新文档数 | 8 |
| 待人工处置项 | 7 |
| 结论 | **有 drift：6 处已自动刷新，3 类 orphan + PRODUCT.md 考古项转人工** |

一句话结论：task 层 `related_intents` 锚点是 PDR-002 时期 prd-writer 写的 kebab-case 猜测命名，与 intent-spec 形式化出的 PascalCase block 名全面 drift；4 条 1:1 清晰映射的断链已修复并对 3 个 verified task 回填 last_verified，3 类指向「被 intent-spec 有意降级/跳过」的 orphan 引用与 PRODUCT.md 考古 [TBD] 转人工。

## Scan Checklist

- [x] target 已确认（all）
- [x] 所有 task / intent / PDR / PRODUCT.md 已枚举（products/skill-runtime/ 全量 + intent-consistency-report）
- [x] 四类 drift 信号已逐条核对，证据已记录（行号 + 字段）
- [x] 已区分「可自动刷新的元数据」与「需 PDR 的正文改动」（经 rubber-duck 复核 scope 边界）

## Drift 信号

### 1. 过期 staleness

| 文档 | 证据 | 严重度 |
|---|---|---|
| （无）| 全部 6 task last_updated=2026-06-08（0 天），无 > 90 天项 | — |

### 2. 断链 broken-ref

> task `related_intents` 锚点用旧 kebab-case 命名，与 intent-spec 形式化的 block 名不符。

| 来源 | 失效引用 | 真实 block | 处置 |
|---|---|---|---|
| T-0001 | `acceptance.intent#T-0001-pipeline-bootstrap-to-first-pause` | `PipelineBootstrapsToFirstPause` | ✅ 已修 |
| T-0002 | `acceptance.intent#T-0002-stage-advance-idempotent` | `StageAdvanceWithApproval` | ✅ 已修（见下注）|
| T-0004 | `acceptance.intent#T-0004-recovered-pipeline-can-advance` | `RecoveredPipelineCanAdvance` | ✅ 已修 |
| T-0006 | `acceptance.intent#T-0006-upgrade-does-not-affect-completed-runs` | `UpgradeDoesNotAffectCompletedRuns` | ✅ 已修 |

> ⚠ T-0002 注：旧 slug 写的是 "idempotent"，但**没有**形式化的幂等 intent；该 task 标题是
> 「把 pipeline 推到下一个 stage（含审批暂停点）」，唯一对应的 verified 验收意图是
> `StageAdvanceWithApproval`（审批闸），故按双射伙伴重指向。幂等性仍停留在 task 级语义，未进 `.intent`。

### 3. 孤儿 orphan

> 指向「intent-spec 有意降级/跳过、`.intent` 中根本不存在」的 block —— 需人决定 re-point/drop（可能需 PDR）。

| 对象 | 情况 | 处置 |
|---|---|---|
| T-0002 → `invariants.intent#stage-transitions-forward-only` | forward-only 从未形式化为具名 invariant | ⚠ 已就地标注，转人工 |
| T-0003 → `invariants.intent#read-only-commands-do-not-mutate-state` | intent-spec 把全局 read-only safety 降级为 task 级断言（否则证伪所有 mutating 操作）| ⚠ 已就地标注，转人工 |
| T-0005 → `invariants.intent#audit-trail-immutable` / `#audit-trail-reconstructible` | intent-spec D2 跳过（intent-lang 无法表达可重建聚合）| ⚠ 已就地标注，转人工 |

> 双射体检：4 条 acceptance verified intent ↔ T-0001/T-0002/T-0004/T-0006 完整 1:1，无未配对 acceptance block。

### 4. 未验证 unverified

| Task | 原 last_verified | intent-check 状态 | 可回填？ |
|---|---|---|---|
| T-0001 | （无）| PipelineBootstrapsToFirstPause = verified | ✅ 已回填 2026-06-08 |
| T-0004 | （无）| RecoveredPipelineCanAdvance = verified | ✅ 已回填 2026-06-08 |
| T-0006 | （无）| UpgradeDoesNotAffectCompletedRuns = verified | ✅ 已回填 2026-06-08 |
| T-0002 | （无）| StageAdvanceWithApproval = verified，**但**仍带 forward-only orphan | ❌ 暂不回填（task 级 last_verified 不能在有未解析 orphan 时认证）|
| T-0003 | （无）| 无 verified intent（约束降级 task 级）| ❌ 不回填 |
| T-0005 | （无）| 无 verified intent（D2 跳过）| ❌ 不回填 |

## 刷新动作

| 文件 | 改动 |
|---|---|
| T-0001.md frontmatter | 锚点→`#PipelineBootstrapsToFirstPause`；新增 `last_verified: 2026-06-08` |
| T-0002.md frontmatter | 锚点→`#StageAdvanceWithApproval`；forward-only 行加 orphan 标注（不回填 last_verified）|
| T-0003.md frontmatter | read-only 锚点加 orphan 标注（降级说明）|
| T-0004.md frontmatter | 锚点→`#RecoveredPipelineCanAdvance`；新增 `last_verified: 2026-06-08` |
| T-0005.md frontmatter | audit-trail 两锚点加 orphan 标注（D2 跳过说明）|
| T-0006.md frontmatter | 锚点→`#UpgradeDoesNotAffectCompletedRuns`；新增 `last_verified: 2026-06-08` |
| tasks/README.md | 健康度快照换真实行数（124/117/112/116/109）+ last_verified 回填说明；刷新状态注 |
| PRODUCT.md | Intents Catalog「待填」→「已填 + Z3 verified」（仅索引/状态，不加业务正文）|

## 健康度快照

| 旅程阶段 | Task 数 | 平均行数 | 最久未更新 | 未引用 task |
|---|---|---|---|---|
| onboarding | 1 | 124 | T-0001（0 天）| 无 |
| daily-ops | 2 | 117 | T-0002 / T-0003（0 天）| 无 |
| troubleshooting | 1 | 112 | T-0004（0 天）| 无 |
| admin | 1 | 116 | T-0005（0 天）| 无 |
| lifecycle | 1 | 109 | T-0006（0 天）| 无 |

> 6 task 全部 0 天，无 > 90 天归档候选；全部有反向引用（related_next_tasks 互链），无孤儿 task。

## 待人工处置

- [ ] **T-0002 `#stage-transitions-forward-only`**：决定是把 forward-only 形式化为具名 invariant（走 prd-writer + 新 PDR + 重跑 intent-check），还是 drop 该锚点。解决后方可给 T-0002 回填 last_verified。
- [ ] **T-0003 `#read-only-commands-do-not-mutate-state`**：read-only 已按 intent-spec 降级为 task 级断言；决定保留指向「task 级断言」的说明锚点还是删除。可能需 PDR。
- [ ] **T-0005 `#audit-trail-immutable` / `#audit-trail-reconstructible`**：D2 能力边界跳过；决定 re-point 到 task 级表述还是删除。可能需 PDR。
- [ ] **PRODUCT.md「用户视角的入口」[TBD: needs archaeology]**：prd-writer 据 PRD/考古填写。
- [ ] **PRODUCT.md「Committed Roadmap」[TBD]**：prd-writer + product-debate 产出。
- [ ] **PRODUCT.md「Open Questions」[TBD]**：fact-extractor Risk Hotspots 填初稿，禁止 AI 编造。
- [ ] **PRODUCT.md「Last-Decision-Ref」[TBD]**：考古确定首决策引用（候选 PDR-001 / PDR-002 / ADR-002）。

> 上述 7 项均超出 living-doc-author 自动刷新范围（需 authorship/decision/PDR）。无「验证失败需修 spec 的 task」——intent-check 报告 0 failed。

---

## 检查清单

- [x] 四类 drift 信号都已扫描并列出（含「（无）」项）
- [x] 刷新动作每条都对应真实文件改动（8 文件，已逐一核对）
- [x] last_verified 只回填了 verified 且无未解析 orphan 的 task（T-0001/T-0004/T-0006；T-0002 因 orphan 暂缓）
- [x] 健康度快照数字与刷新后的 README 一致（124/117/112/116/109）
- [x] 所有越界 drift 已转「待人工处置」，未擅自改正文（PRODUCT.md 考古 [TBD] 保留原样）
- [x] frontmatter 计数与正文一致（扫 12 / drift 9 / 刷新 8 / 待人工 7）
