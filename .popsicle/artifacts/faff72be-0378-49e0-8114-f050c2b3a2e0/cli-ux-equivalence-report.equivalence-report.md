---
id: ea0233cf-ea6e-4374-a570-6198fd552138
doc_type: equivalence-report
title: cli-ux equivalence report
status: final
skill_name: equivalence-baseline
pipeline_run_id: faff72be-0378-49e0-8114-f050c2b3a2e0
spec_id: 308c231f-0e91-4922-87f0-22b63be857b6
version: 1
parent_doc_id: null
tags: []
metadata: null
created_at: 2026-06-10T04:10:00Z
updated_at: 2026-06-10T04:10:00Z
---

---
artifact: equivalence-report
slug: cli-ux-equivalence-report
generated_by: equivalence-baseline
slice: cli-ux
last_updated: 2026-06-10
golden_total: 6
golden_pass: 6
golden_fail: 0
divergence_count: 1
equivalence_gate_pass: true
baseline_dir: docs/baseline/2026-06-10/cli-ux/
baseline_manifest: docs/baseline/2026-06-10/cli-ux/baseline.yaml
query_anchors:
  - "legacy 和 new 行为一致吗？"
  - "golden 对账过了几条？"
---

# 等价性基线报告 — cli-ux-equivalence-report

> 由 `equivalence-baseline` 产出。legacy pin 见 `LEGACY_PIN.md`。

## Summary

| 指标 | 值 |
|---|---|
| Slice | cli-ux |
| Legacy pin | `c76d729db91c59009f0fa8f7c6f1e499eb0c7eb1`（见 LEGACY_PIN.md）|
| Golden 总数 | 6 |
| pass（semantic golden）| 6 |
| fail | 0 |
| divergence（已 ADR 登记）| 1（scope decision，见下表）|
| **equivalence_gate_pass** | true |

门禁：`golden_pass >= 5` → pass（6/6 semantic golden）。

## Golden Inventory

- [x] slice 已确认：`cli-ux`
- [x] ≥5 条 golden 已列出（6 条）
- [x] 每条能追溯到 acceptance block 或 invariant block
- [x] 已知 divergence 已单独列出

## Baseline Manifest

本报告的 golden/pass/fail/divergence 数字来自
`docs/baseline/2026-06-10/cli-ux/baseline.yaml`。

- [x] `baseline.yaml` 已创建
- [x] Summary 计数与 `baseline.yaml` 一致
- [x] Golden 清单状态与 `baseline.yaml` 一致

## Golden 清单

| ID | 描述 | 脚本 | Legacy | New | 结果 | diff 摘要 |
|---|---|---|---|---|---|---|
| G-001 | help surface exposes IDD main path and hides removed commands | `golden-001-help-surface.sh` | legacy `popsicle` command surface | `top_level_help` / `contains_removed_top_level_command` | PASS | — |
| G-002 | issue start returns run id and lock signal | `golden-002-issue-start.sh` | issue + pipeline run creation semantics | `start_issue_run` | PASS | — |
| G-003 | doc create writes artifact and document row | `golden-003-doc-create.sh` | doc artifact + documents row semantics | `create_document_artifact` | PASS | — |
| G-004 | stage complete requires confirm then advances | `golden-004-stage-complete.sh` | pipeline stage approval semantics | `complete_pipeline_stage` | PASS | — |
| G-005 | admin maintenance commands are nested | `golden-005-admin-tree.sh` | top-level `migrate` / `reinit` | `admin migrate` / `admin reinit` parse tree | PASS | — |
| G-006 | removed commands return actionable errors | `golden-006-removed-commands.sh` | `checklist` / `item` / `sync` command families | actionable removed-command errors | PASS | — |

## 运行结果

```
./docs/baseline/2026-06-10/cli-ux/run-all.sh
exit 0

All cli-ux golden baselines passed.
```

补充实跑：

```
cargo test -p cli-ux
exit 0

tests/golden.rs: 6 passed; 0 failed
tests/intent_properties.rs: 7 passed; 0 failed
```

## Traceability（拟写入 migration/traceability.md）

| Legacy 路径 | 新位置 | 责任 Spec | 切流 ADR | 等价性 baseline | 状态 |
|---|---|---|---|---|---|
| `crates/popsicle-cli/src/main.rs`（top-level command surface）| `crates/cli-ux/src/lib.rs::TOP_LEVEL_COMMANDS` / `top_level_help` | slice-3-cli-ux | ADR-008（待 cutover）| G-001 | in-shadow |
| `crates/popsicle-cli/src/commands/issue.rs` + `pipeline.rs`（issue start/run signal）| `crates/cli-ux/src/lib.rs::start_issue_run` | slice-3-cli-ux | ADR-008（待 cutover）| G-002 | in-shadow |
| `crates/popsicle-cli/src/commands/doc.rs`（artifact + document row）| `crates/cli-ux/src/lib.rs::create_document_artifact` | slice-3-cli-ux | ADR-008（待 cutover）| G-003 | in-shadow |
| `crates/popsicle-cli/src/commands/pipeline.rs`（stage complete approval）| `crates/cli-ux/src/lib.rs::complete_pipeline_stage` | slice-3-cli-ux | ADR-008（待 cutover）| G-004 | in-shadow |
| `crates/popsicle-cli/src/commands/{admin,migrate,reinit}.rs` | `crates/cli-ux/src/lib.rs::AdminCommand` / `parse_args` | slice-3-cli-ux | ADR-008（待 cutover）| G-005 | in-shadow |
| legacy `checklist` / `item` / `sync` command families | `REMOVED_TOP_LEVEL_COMMANDS` + actionable errors | slice-3-cli-ux | ADR-008（待 cutover）| G-006 | in-shadow |

## Divergence

| ID | 行为 | Legacy | New | 原因 | ADR |
|---|---|---|---|---|---|
| D-001 | CLI byte parity / full command compatibility | legacy stdout/stderr wording and 22 top-level command surface | semantic IDD command shell with preserve/redesign/drop/defer disposition | PDR-001 states this is not a full compatibility rewrite; acceptance captures command effects, not byte parity | ADR-007 |

## 门禁判定

- [x] ≥5 golden pass
- [x] baseline 目录已创建且 README 可复现
- [x] baseline manifest 与报告计数一致
- [x] traceability 草稿已写

## 检查清单

- [x] 每条 golden 有实跑证据
- [x] pass/fail 数字与 `baseline.yaml` / Summary 一致
- [x] divergence 未隐瞒
- [x] equivalence_gate_pass 可复算
