---
id: fc189d57-c273-4225-8f69-ee68a8741425
doc_type: living-doc-sync-report
title: artifact-system 活文档保活报告（intent 锚点对账 + last_verified 全回填）
status: final
skill_name: living-doc-author
pipeline_run_id: 49989451-d311-4f76-b15a-cc7dfcf7787f
spec_id: cf6e7062-b75f-4925-b633-82a04e775a76
version: 1
parent_doc_id: null
tags: []
metadata: null
created_at: 2026-06-09T06:47:53.164400Z
updated_at: 2026-06-09T06:54:12.120736Z
---

---
artifact: living-doc-sync-report
slug: artifact-system-living-doc-sync
generated_by: living-doc-author
target: all
last_updated: 2026-06-09
docs_scanned: 12
drift_signals: 2
docs_refreshed: 9
manual_followups: 1
query_anchors:
  - "artifact-system 的 task→intent 锚点还对得上吗？"
  - "intent-check 改了 invariants 后哪些 task 锚点需重指向？"
  - "哪些 task 已 Z3 verified、回填了 last_verified？"
---

# 活文档保活报告 — artifact-system-living-doc-sync

> 由 `living-doc-author` skill 生成。**只对账与刷新活文档元数据**（索引 / 锚点 / last_verified /
> 健康度 / catalog 状态），不创作正文、不改业务逻辑——后者走 prd-writer + PDR（charter 铁律 #3）。

## Summary

| 指标 | 值 |
|---|---|
| target | all |
| 扫描文档数 | 12（6 task + 3 intent + PRODUCT.md + tasks/README.md + intent-consistency-report）|
| 发现 drift 信号 | 2（均为 broken-ref，源自 intent-check 期 invariants 重构）|
| 本次刷新文档数 | 9（6 task + README + PRODUCT.md，其中 2 task 改锚点、6 task 回填 last_verified）|
| 待人工处置项 | 1（ContextOrder property test，承接 intent-check § Skipped，落代码时补）|
| 结论 | **2 处断链已修复并对 6/6 verified task 回填 last_verified；无 orphan，双射干净** |

一句话结论：intent-check 闸为修 cross-type havoc 把 invariants 的两个保持型约束做了重构（`GuardResultIsTotal`→`EvaluateGuard`、`TaskChunkRenamePreservesFields` 重定位到 `acceptance.intent#RenameWorkItemToTaskChunk`），导致 T-AS-0005/0006 两条 `related_intents` 锚点 drift；已按真实 block 名修复并对全部 6 个 task 回填 `last_verified: 2026-06-09`，无 orphan 引用，6 acceptance/invariant block ↔ 6 task 双射完整。

## Scan Checklist

- [x] target 已确认（all）
- [x] 所有 task / intent / PDR / PRODUCT.md 已枚举（products/artifact-system/ 全量 + intent-consistency-report 05d1d91e）
- [x] 四类 drift 信号已逐条核对，证据已记录（行号 + 锚点名）
- [x] 已区分「可自动刷新的元数据」与「需 PDR 的正文改动」（仅改锚点/索引/状态，未动业务正文）

## Drift 信号

### 1. 过期 staleness

| 文档 | 证据 | 严重度 |
|---|---|---|
| （无）| 全部 6 task last_updated=2026-06-09（0 天），无 > 60 天项 | — |

### 2. 断链 broken-ref

> 源头：intent-check 期为消除 intent-lang「safety 文件级附加到不相交类型 → cross-havoc」的 false-failure，对 invariants.intent 做了重构（详见 intent-consistency-report 05d1d91e § Failures）。

| 来源 | 失效引用 | 真实 block | 处置 |
|---|---|---|---|
| T-AS-0005 | `invariants.intent#GuardResultIsTotal` | `invariants.intent#EvaluateGuard`（受 `safety UnknownGuardIsInvalid` 守护的全函数性不变量）| ✅ 已修（frontmatter + 正文链接 + 限制段）|
| T-AS-0006 | `invariants.intent#TaskChunkRenamePreservesFields` | `acceptance.intent#RenameWorkItemToTaskChunk`（重定位到无-safety 文件以避免 havoc）| ✅ 已修（frontmatter + 正文链接 + 完成路径段）|

> tasks/README.md 末两行映射列、PRODUCT.md Intents Catalog 同步指向旧名，亦一并刷新。

### 3. 孤儿 orphan

> acceptance/invariant 具名 block ↔ task 双射体检。

| 检查 | 结果 |
|---|---|
| 6 个 verified block（5 acceptance + 1 invariant `EvaluateGuard`）是否都有 task | ✅ 全配对（DocumentRoundTrips→T-AS-0001/0002、GuardChecklistCompleteIffNoUnchecked→T-AS-0003、ContextAssemblyOrdersByRelevance→T-AS-0004、ExtractPreservesKind+EvaluateGuard→T-AS-0005、RenameWorkItemToTaskChunk→T-AS-0006）|
| 是否有 task 指向不存在的 block | ✅ 无（修复后 7/7 锚点全 resolve）|
| 是否有未配对 acceptance block | ✅ 无 |

> `ContextOrderIndependentOfRegistration` 在 intent-check 期剥离到代码层（intent-lang 无序列算子），**无 task 以它为 related_intents**，故不构成断链 orphan，仅作代码期 followup（见 § 待人工处置）。

### 4. 未验证 unverified

| Task | 原 last_verified | intent-check 状态 | 可回填？ |
|---|---|---|---|
| T-AS-0001 | （无）| DocumentRoundTrips = verified | ✅ 已回填 2026-06-09 |
| T-AS-0002 | （无）| DocumentRoundTrips = verified | ✅ 已回填 2026-06-09 |
| T-AS-0003 | （无）| GuardChecklistCompleteIffNoUnchecked = verified | ✅ 已回填 2026-06-09 |
| T-AS-0004 | （无）| ContextAssemblyOrdersByRelevance = verified | ✅ 已回填 2026-06-09 |
| T-AS-0005 | （无）| ExtractPreservesKind + EvaluateGuard = verified | ✅ 已回填 2026-06-09 |
| T-AS-0006 | （无）| RenameWorkItemToTaskChunk = verified | ✅ 已回填 2026-06-09 |

> 与 slice-1 不同：本 slice 无「带未解析 orphan 的 task」，6/6 锚点修复后均干净 verified，故 6 个全部回填。

## 刷新动作

| 文件 | 改动 |
|---|---|
| T-AS-0001.md frontmatter | 新增 `last_verified: 2026-06-09`（锚点已正确）|
| T-AS-0002.md frontmatter | 新增 `last_verified: 2026-06-09` |
| T-AS-0003.md frontmatter | 新增 `last_verified: 2026-06-09` |
| T-AS-0004.md frontmatter | 新增 `last_verified: 2026-06-09` |
| T-AS-0005.md | 锚点 `#GuardResultIsTotal`→`#EvaluateGuard`（frontmatter + 限制段 + 形式化链接 + 验证说明）；新增 `last_verified` |
| T-AS-0006.md | 锚点 `invariants#TaskChunkRenamePreservesFields`→`acceptance#RenameWorkItemToTaskChunk`（frontmatter + 完成路径段 + 形式化链接）；新增 `last_verified` |
| tasks/README.md | Status/健康度列换真实状态（6/6 锚点 verified + last_verified）；末两行映射 `GuardResultIsTotal`/`TaskChunkRenamePreservesFields`→`EvaluateGuard`/`RenameWorkItemToTaskChunk` |
| PRODUCT.md | Intents Catalog「待填」→标注各层 block 数 + Z3 verified 2026-06-09（仅索引/状态，不加业务正文）|

## 健康度快照

| 旅程阶段 | Task 数 | 平均行数 | 最久未更新 | 锚点状态 | 未引用 task |
|---|---|---|---|---|---|
| onboarding | 1 | 80 | T-AS-0001（0 天）| ✅ verified | 无 |
| daily-ops | 3 | 80 | T-AS-0002/0003/0004（0 天）| ✅ verified | 无 |
| troubleshooting | 1 | 80 | T-AS-0005（0 天）| ✅ verified | 无 |
| admin | 0 | — | — | n/a | n/a |
| lifecycle | 1 | 80 | T-AS-0006（0 天）| ✅ verified | 无 |

> 6 task 全部 0 天，无 > 60 天归档候选；全部经 related_next_tasks 互链，无孤儿 task。

## 待人工处置

- [x] **ContextOrder property test（代码期）**：`ContextOrderIndependentOfRegistration` 在 intent-check 期剥离（intent-lang 无 sequence/sort/聚合算子），需在 crates/artifact-system 落地时补：① context 多级确定性排序键（Relevance→固定优先级→稳定 id）；② 打乱注册序断言装配序不变的 property test。承接 intent-consistency-report 05d1d91e § Skipped。

> 仅此 1 项，且属代码实现期 followup（非活文档 drift），living-doc-author 不自动处理。无「验证失败需修 spec 的 task」——intent-check 报告修复后 0 failed。

---

## 检查清单

- [x] 四类 drift 信号都已扫描并列出（含「（无）」项）
- [x] 刷新动作每条都对应真实文件改动（9 文件，已逐一核对，anchors 全 resolve）
- [x] last_verified 只回填了 verified 且无未解析 orphan 的 task（6/6 干净，全回填）
- [x] 健康度快照数字与刷新后的 README 一致
- [x] 所有越界项已转「待人工处置」，未擅自改业务正文（PRODUCT.md 仅改 catalog 状态行）
- [x] frontmatter 计数与正文一致（扫 12 / drift 2 / 刷新 9 / 待人工 1）
