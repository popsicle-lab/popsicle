---
id: 1dd0053a-0d5f-4852-817e-5cafcaa35814
doc_type: intent-consistency-report
title: cli-ux self-hosting intent consistency
status: final
skill_name: intent-consistency-check
pipeline_run_id: 70be4a70-03da-47a1-9366-9ba889f8052f
spec_id: 308c231f-0e91-4922-87f0-22b63be857b6
version: 1
parent_doc_id: null
tags:
- cli-ux
- self-hosting
- intent-check
metadata: null
created_at: 2026-06-10T11:20:00Z
updated_at: 2026-06-10T18:23:21.688979Z
---

---
artifact: intent-consistency-report
slug: cli-ux-self-hosting
generated_by: intent-consistency-check
mode: observe
last_updated: 2026-06-10
intent_files_checked: 13
vcs_total: 28
vcs_verified: 28
vcs_failed: 0
vcs_skipped: 0
overall: pass
consecutive_clean_runs: 1
gate_ready: false
---

# Intent 一致性报告 — cli-ux-self-hosting

## Summary

| 指标 | 数 |
|---|---:|
| 检查的 `.intent` 文件 | 13 |
| VC 总数 | 28 |
| verified | 28 |
| failed | 0 |
| skipped | 0 |
| 总体结论 | PASS |

一句话结论：PDR-002 新增的 self-hosting/provenance intent 与既有 artifact-system、cli-ux、saas-billing、skill-runtime intent 全量自洽。

## cli-ux Results

| 文件 | verified | failed | skipped |
|---|---:|---:|---:|
| `products/cli-ux/intents/acceptance.intent` | 8 | 0 | 0 |
| `products/cli-ux/intents/invariants.intent` | 1 | 0 | 0 |
| `products/cli-ux/intents/self-hosting-invariants.intent` | 1 | 0 | 0 |
| `products/cli-ux/intents/contracts.intent` | 0 | 0 | 0 |

## Verified New Intents

| Intent | File |
|---|---|
| `SelfHostedWorkflowSmokePasses` | `acceptance.intent` |
| `BinaryProvenanceVisible` | `acceptance.intent` |
| `SelfHostSmokeUsesCurrentWorkspaceBinary` | `self-hosting-invariants.intent` |

## Failures

（无）

## Skipped

（无）

## Follow-up

- `gate_ready=false` because this is the first clean run for PDR-002.
- Delivery must prove the same behavior through `./target/debug/popsicle` smoke.

## Checklist

- [x] All project intent files checked
- [x] Counts match real `intent-validate path=products` output
- [x] Failed and skipped are zero
