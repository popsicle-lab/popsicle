---
id: ad397077-fa3c-485a-9e3f-0df7a12961dd
doc_type: intent-consistency-report
title: cli-ux intent consistency command tree redesign
status: final
skill_name: intent-consistency-check
pipeline_run_id: 82936830-374d-4ac4-b0f6-5639a679bfc8
spec_id: 308c231f-0e91-4922-87f0-22b63be857b6
version: 1
parent_doc_id: null
tags: ["cli-ux", "PDR-001", "ADR-007", "intent-check"]
metadata: null
created_at: 2026-06-09T14:58:51.757714Z
updated_at: 2026-06-09T15:02:00.000000Z
---

---
artifact: intent-consistency-report
slug: cli-ux-command-tree-redesign
generated_by: intent-consistency-check
mode: observe
last_updated: 2026-06-10
intent_files_checked: 3
vcs_total: 7
vcs_verified: 7
vcs_failed: 0
vcs_skipped: 0
overall: pass
consecutive_clean_runs: 3
gate_ready: true
query_anchors:
  - "我的 cli-ux .intent 自洽吗？"
  - "cli-ux 哪条意图验证失败了，反例是什么？"
  - "cli-ux 是否可以升级为 CI gate？"
---

# Intent 一致性报告 — cli-ux-command-tree-redesign

> 由 `intent-consistency-check` skill 调用 intent-lang check 生成。
> 本报告只验证逻辑一致性，不覆盖时间、性能或运行时实现细节；这些约束进入 task 成功标志与 delivery 测试。

## Summary

| 指标 | 数 |
|---|---|
| 检查的 `.intent` 文件 | 3 |
| VC 总数 | 7 |
| verified | 7 |
| failed | 0 |
| skipped | 0 |
| 总体结论 | PASS |
| 模式 | observe（报告但不阻断 pipeline）|

一句话结论：cli-ux 的新设计 intent 已自洽；老 CLI facts 仅作为参考输入，本次未建立字节级兼容 baseline。

## Per-File Results

| 文件 | verified | failed | skipped | exit | 结论 |
|---|---:|---:|---:|---:|---|
| `products/cli-ux/intents/acceptance.intent` | 6 | 0 | 0 | 0 | pass |
| `products/cli-ux/intents/invariants.intent` | 1 | 0 | 0 | 0 | pass |
| `products/cli-ux/intents/contracts.intent` | 0 | 0 | 0 | 0 | pass |

## Verified Intents

| 文件 | verified intent |
|---|---|
| `acceptance.intent` | `InitShowsNextStep` |
| `acceptance.intent` | `IssueStartCreatesRun` |
| `acceptance.intent` | `DocCommandWritesArtifact` |
| `acceptance.intent` | `StageAdvanceReflectsState` |
| `acceptance.intent` | `ErrorsAreActionable` |
| `acceptance.intent` | `AdminCommandsAreExplicit` |
| `invariants.intent` | `RenderTopLevelHelp` |

## Failures

（无）

## Skipped

| VC | 文件 | 原因 |
|---|---|---|
| — | — | — |

## Disposition

- `observe`: 本 skill 产出报告，不阻断 pipeline。
- `gate`: CI 可在未来调用 intent-validate / intent-cli 的 exit code 做硬闸。
- 跟进项：delivery 阶段把 6 条 acceptance、1 条 command-surface invariant 映射到 CLI 行为测试或 golden 对账。

## Gate Readiness

| 项 | 值 |
|---|---|
| 本次 overall | pass |
| consecutive_clean_runs（含本次）| 3 |
| 升级阈值 N | 3 |
| gate_ready | true |

判据：连续 3 次对 cli-ux 全量 `.intent` 验证都 0 failed/unknown 后，再把 CI 从 observe 升级为 gate。

2026-06-10 补充实跑：连续 3 轮调用
`legacy/popsicle/vender/intent-lang/target/release/intent-cli check --format json`
检查 `acceptance.intent` / `invariants.intent` / `contracts.intent`，三轮均 exit 0；
每轮结果均为 acceptance 6 verified、invariants 1 verified、contracts 0 VC / ok。

## 检查清单

- [x] 枚举了 cli-ux 产品内所有 `.intent` 文件
- [x] 每个文件的结果都来自真实 intent check 输出
- [x] failed 为 0，无需粘贴反例
- [x] skipped 为 0
- [x] frontmatter 计数与正文一致
- [x] 计算出 consecutive_clean_runs 与 gate_ready
