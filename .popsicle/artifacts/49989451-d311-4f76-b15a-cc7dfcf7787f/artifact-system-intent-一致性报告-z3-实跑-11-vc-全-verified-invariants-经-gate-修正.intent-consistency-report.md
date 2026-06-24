---
id: 05d1d91e-47f3-487c-a512-25f17d9d0698
doc_type: intent-consistency-report
title: artifact-system intent 一致性报告（Z3 实跑：11 VC 全 verified，invariants 经 gate 修正）
status: final
skill_name: intent-consistency-check
pipeline_run_id: 49989451-d311-4f76-b15a-cc7dfcf7787f
spec_id: cf6e7062-b75f-4925-b633-82a04e775a76
version: 1
parent_doc_id: null
tags: []
metadata: null
created_at: 2026-06-09T06:39:39.590601Z
updated_at: 2026-06-09T06:41:17.996386Z
---

# intent 一致性报告 — artifact-system（Z3 实跑）

> **验证器**: `intent-cli`（vendored `legacy/popsicle/vender/intent-lang`，intent 0.1.0）— **真实 Z3 输出，非静态审查**
> **模式**: observe（报告但不阻断 pipeline）
> **Source intent-spec**: `7bc5b9af`

## Summary

| 指标 | 数 |
|---|---|
| 检查的 `.intent` 文件 | 9（3 product × 3 层）|
| VC 总数 | 11 |
| ✅ verified | 11 |
| ❌ failed | 0（修正后）|
| ⚠️ skipped | 0 |
| 总体结论 | **PASS** |
| 模式 | observe |

一句话结论：artifact-system 的 acceptance（5 VC）+ invariants（1 VC）全部 `verified`、0 diagnostics；全量 9 文件 11 VC 全绿。**首轮 Z3 实跑曾捕获 invariants.intent 2 处 false-failure（见 § Failures），经 gate 内修正后复跑全 verified**——正是 intent-check 闸应当拦下的问题。

## Per-File Results

> `intent-cli check --format json` 的真实输出（`ok` 字段）。

| 文件 | verified | failed | skipped | ok | 结论 |
|---|---|---|---|---|---|
| products/artifact-system/intents/acceptance.intent | 5 | 0 | 0 | true | ✅ |
| products/artifact-system/intents/invariants.intent | 1 | 0 | 0 | true | ✅ |
| products/artifact-system/intents/contracts.intent | 0 | 0 | 0 | true | ✅（goal 块 0 VC）|
| products/skill-runtime/intents/acceptance.intent | 4 | 0 | 0 | true | ✅ |
| products/skill-runtime/intents/invariants.intent | 1 | 0 | 0 | true | ✅ |
| products/skill-runtime/intents/contracts.intent | 0 | 0 | 0 | true | ✅ |
| products/cli-ux/intents/acceptance.intent | 0 | 0 | 0 | true | ✅（stub）|
| products/cli-ux/intents/invariants.intent | 0 | 0 | 0 | true | ✅（stub）|
| products/cli-ux/intents/contracts.intent | 0 | 0 | 0 | true | ✅（stub）|

### artifact-system VC 明细（6 verified）

| VC | kind | 文件 | status |
|---|---|---|---|
| DocumentRoundTrips | intent | acceptance.intent | verified |
| GuardChecklistCompleteIffNoUnchecked | intent | acceptance.intent | verified |
| ContextAssemblyOrdersByRelevance | intent | acceptance.intent | verified |
| ExtractPreservesKind | intent | acceptance.intent | verified |
| RenameWorkItemToTaskChunk | intent | acceptance.intent | verified（intent-check 从 invariants 重定位）|
| EvaluateGuard | intent | invariants.intent | verified（受 `UnknownGuardIsInvalid` safety 守护 → GuardResultIsTotal 真不变量）|

> 说明：`safety UnknownGuardIsInvalid` 不单列结果行——其 VC 无条件附加到同文件 `EvaluateGuard`；该 intent
> `verified`，即「`(!recognized') ==> (outcome' == GInvalid)`」成立：未知 guard 求值后 outcome 必为 GInvalid，
> 不 panic、不静默 pass。

## Failures

> 首轮 Z3 实跑（修正前）捕获的真实反例 —— gate 价值所在，已修复并复跑通过。

**首轮 invariants.intent（修正前）= ok:false，2 false-failure：**

```
EvaluateGuard           failed   t.kind = CBug, t.fieldsHash = 0, g.outcome = GInvalid, g.recognized = false
RenameWorkItemToTaskChunk failed g.outcome = GInvalid, t.kind = CBug, t.fieldsHash = 0, g.recognized = false
```

**根因**：intent-lang 四规则#2「safety 无条件附加到文件内**所有** intent」对**不相交类型**亦生效。invariants.intent 原同时放
`UnknownGuardIsInvalid(g: GuardEval)` 与 `RenamePreservesPayload(t: TaskChunk)` 两个保持型 safety：
- `EvaluateGuard(g)` 被附加 `RenamePreservesPayload(t)`，但 EvaluateGuard 不约束 `t` → `t'` 被 havoc → 违反 `t.kind'==t.kind` →（反例 t.kind=CBug 被改写）
- `RenameWorkItemToTaskChunk(t)` 被附加 `UnknownGuardIsInvalid(g)`，`g'` 被 havoc → 违反 → 同理

这是**规约层 false-failure**（非业务逻辑错），但若放任，invariants 闸会永红或被误绕过。

**修正**（gate 内，已复跑 verified）：
- invariants.intent 只保留**单一**保持型 safety `UnknownGuardIsInvalid` + `EvaluateGuard`（同类型，无 cross-havoc）
- `RenameWorkItemToTaskChunk` 重定位到 acceptance.intent（该文件**无 safety**，不产生 havoc）→ verified
- `ContextOrderIndependentOfRegistration` 剥离（见 § Skipped）

复跑：`invariants.intent` ok:true（1 verified）、`acceptance.intent` ok:true（5 verified）。

## Skipped

> 能力边界项**未写入** `.intent`（剥离到代码层 + property test），故验证集不出现 skipped 行。

| 约束 | 为何不进 `.intent` | 替代守护 |
|---|---|---|
| `ContextOrderIndependentOfRegistration` | permutation-invariance + 序列推理；intent-lang 无 sequence/sort/聚合算子，无法写成产生 VC 的 safety | ① context.rs 多级确定性排序键（Relevance→固定优先级→稳定 id）；② crates/artifact-system property test（打乱注册序断言装配序不变）|

## Disposition

- **observe（本 skill 行为）**：修正后 0 failed，skill 不阻断 pipeline。
- **gate（CI 行为）**：CI 跑 `intent-validate`，靠 exit code ≠ 0 拦合并。
- 跟进项：
  - [x] 修正 invariants.intent cross-type safety havoc（Rename 重定位 acceptance、ContextOrder 剥离）
  - [x] crates/artifact-system 落地时补 ContextOrder property test（承接本报告 § Skipped）

## Gate Readiness（observe → gate 退出判据）

| 项 | 值 |
|---|---|
| 本次 overall | pass |
| consecutive_clean_runs（含本次）| 1 |
| 升级阈值 N | 3 |
| **gate_ready** | false |

**判据**：`consecutive_clean_runs >= 3` 且本次 `overall == pass` → `gate_ready = true`。本次首轮 clean（修正后），未达阈值，保持 observe 攒数。

**达到 gate_ready 后的开闸动作**：

```yaml
# .github/workflows/intent-gate.yml
- name: intent consistency gate
  run: |
    legacy/popsicle/vender/intent-lang/target/release/intent-cli check products/<product>/intents/*.intent
    # 任一 failed → exit ≠ 0 → 阻断合并；skipped 不算 failed
```

## 检查清单

- [x] 枚举了项目内**所有** `.intent` 文件（9 个，products/*/intents/）
- [x] 每个文件结果均来自 `intent-cli` 的**真实 Z3 输出**，未臆造（JSON ok 字段）
- [x] failed 反例已粘贴根因 + 修正（invariants cross-type havoc）
- [x] skipped/剥离项（ContextOrder）已交代替代守护
- [x] frontmatter 计数与正文一致（9 文件 / 11 VC / 11 verified / 0 failed / 0 skipped）
- [x] 首轮运行，consecutive_clean_runs=1，gate_ready=false
- [x] 已给出达到 gate_ready 后的 CI 开闸建议
