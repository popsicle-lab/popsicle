---
id: c1dd4a8f-9c79-4b59-9810-4b45318e725a
doc_type: intent-consistency-report
title: skill-runtime intent 一致性报告（Z3 全 verified）
status: final
skill_name: intent-consistency-check
pipeline_run_id: f89529af-d8ce-40f7-ad05-985e35b9cfec
spec_id: df11b6f9-e504-42b7-8d27-700d529c2346
version: 1
parent_doc_id: null
tags: []
metadata: null
created_at: 2026-06-08T08:17:56.793917Z
updated_at: 2026-06-08T08:19:47.872533Z
---

---
artifact: intent-consistency-report
slug: skill-runtime-intent-consistency
generated_by: intent-consistency-check
mode: observe
last_updated: 2026-06-08
intent_files_checked: 9
vcs_total: 5
vcs_verified: 5
vcs_failed: 0
vcs_skipped: 0
overall: pass
consecutive_clean_runs: 1
gate_ready: false
query_anchors:
  - "skill-runtime 的 .intent 自洽吗？Z3 过了吗？"
  - "HC-2 审批闸不变量被真正验证了吗？"
  - "intent gate 什么时候能从 observe 升级成 CI 硬闸？"
---

# Intent 一致性报告 — skill-runtime-intent-consistency

> 由 `intent-consistency-check` skill 调用 intent-lang + Z3 生成。
> **只验证逻辑一致性**（require / ensure / invariant / safety 之间不打架），
> 不覆盖时间 / 性能 / 运行时约束（那些由 task「可观察的成功标志」守护）。
>
> ⚙️ 验证器：`popsicle tool run intent-validate` 因 PATH 无 `intent` 二进制而失败，
> 已回退——从 `legacy/popsicle/vender/intent-lang` 用 `cargo build --release -p intent-cli`
> 构建 `intent-cli`，并以 `intent-cli --format json check <file>` 实跑（Z3 在 `~/.local/bin/z3`）。
> **本报告是真实 Z3 验证结果，非静态审查。**

## Summary

| 指标 | 数 |
|---|---|
| 检查的 `.intent` 文件 | 9（3 product × 3 层）|
| VC 总数 | 5 |
| ✅ verified | 5 |
| ❌ failed | 0 |
| ⚠️ skipped | 0 |
| 总体结论 | **PASS** |
| 模式 | observe（报告但不阻断 pipeline）|

一句话结论：skill-runtime 的 acceptance/invariants/contracts 全部 `ok:true`、0 diagnostics、0 failed/skipped；HC-2 审批闸不变量 `ApprovedBeforeCompleted` 经 Z3 在绑定操作 `StageAdvanceWithApproval` 上真实验证通过。

## Per-File Results

> `exit` 是 `intent check` 的退出码（0 = 全通过/合理跳过）。

| 文件 | verified | failed | skipped | exit | 结论 |
|---|---|---|---|---|---|
| products/skill-runtime/intents/acceptance.intent | 4 | 0 | 0 | 0 | ✅ |
| products/skill-runtime/intents/invariants.intent | 1 | 0 | 0 | 0 | ✅ |
| products/skill-runtime/intents/contracts.intent | 0 | 0 | 0 | 0 | ✅（goal 块 0 VC）|
| products/cli-ux/intents/acceptance.intent | 0 | 0 | 0 | 0 | ✅（stub）|
| products/cli-ux/intents/invariants.intent | 0 | 0 | 0 | 0 | ✅（stub）|
| products/cli-ux/intents/contracts.intent | 0 | 0 | 0 | 0 | ✅（stub）|
| products/artifact-system/intents/acceptance.intent | 0 | 0 | 0 | 0 | ✅（stub）|
| products/artifact-system/intents/invariants.intent | 0 | 0 | 0 | 0 | ✅（stub）|
| products/artifact-system/intents/contracts.intent | 0 | 0 | 0 | 0 | ✅（stub）|

### skill-runtime VC 明细（5 verified）

| VC | kind | 文件 | status |
|---|---|---|---|
| PipelineBootstrapsToFirstPause | intent | acceptance.intent | verified |
| StageAdvanceWithApproval | intent | acceptance.intent | verified（trivial 规约副本）|
| RecoveredPipelineCanAdvance | intent | acceptance.intent | verified |
| UpgradeDoesNotAffectCompletedRuns | intent | acceptance.intent | verified |
| StageAdvanceWithApproval | intent | invariants.intent | verified（受 `ApprovedBeforeCompleted` safety 守护 → HC-2 真不变量）|

> 说明：`safety ApprovedBeforeCompleted` 本身不单列结果行——它的 VC 被无条件附加到同文件
> `StageAdvanceWithApproval` 上；该 intent 在 invariants.intent 中 `verified`，即 HC-2 审批闸
> 「`(status'==StageCompleted && requiresApproval') ==> approvedAt'>0`」成立。

## Failures

（无）

## Skipped

（无）

> 注：intent-spec 阶段判为「能力边界 skipped」的 audit-trail 可重建意图**未写入** `.intent`
> （留在 T-0005 task），故本次验证集里不出现 skipped 行。read-only 不变性与 schema 向后兼容
> 同理降级到 task，不在 `.intent`。

## Disposition

- **observe（本 skill 行为）**：本次 0 failed，无需跟进项；skill 不阻断 pipeline。
- **gate（CI 行为，非 skill 状态）**：CI 跑 `intent-validate` tool，靠其 exit code ≠ 0 拦合并。
- 跟进项：
  - [ ] （无失败项）后续 intent 增改后重跑本检查，累计 clean runs。

## Gate Readiness（observe → gate 退出判据）

| 项 | 值 |
|---|---|
| 本次 overall | pass |
| consecutive_clean_runs（含本次）| 1 |
| 升级阈值 N | 3 |
| **gate_ready** | false |

**判据**：`consecutive_clean_runs >= 3` 且本次 `overall == pass` → `gate_ready = true`。
本次为首轮 clean run（计数从 0 起算 +1 = 1），未达阈值，保持 observe 攒数。
再连续 2 次对全量 `.intent` 跑验证 0 failed/unknown 即可开 CI 硬闸。

**达到 gate_ready 后的开闸动作**（CI 配置建议）：

```yaml
# .github/workflows/intent-gate.yml
- name: intent consistency gate
  run: |
    popsicle tool run intent-validate path=products format=text
    # 任一 FAILED → tool exit 1 → step 失败 → 阻断合并；skipped 不算 failed，不误拦
```

---

## 检查清单

- [x] 枚举了项目内**所有** `.intent` 文件（9 个，products/*/intents/）
- [x] 每个文件结果均来自 intent-cli 的**真实 Z3 输出**，未臆造
- [x] 无 failed（故无反例需粘贴）
- [x] 无 skipped（能力边界项已在 intent-spec 阶段降级到 task，未进 `.intent`）
- [x] frontmatter 计数与正文一致（9 文件 / 5 VC / 5 verified / 0 failed / 0 skipped）
- [x] 首轮运行，consecutive_clean_runs=1，gate_ready=false
- [x] 已给出达到 gate_ready 后的 CI 开闸建议
