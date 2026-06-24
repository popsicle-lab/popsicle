---
id: 6d56ef62-0c4a-4615-99b0-6e914d58aaa5
doc_type: equivalence-report
title: cli-ux self-host equivalence report
status: active
skill_name: equivalence-baseline
pipeline_run_id: 8198f9ff-b120-4b0d-8dae-9b379dc9a1d8
version: 1
---

---
artifact: equivalence-report
slug: cli-ux-self-host-equivalence-report
generated_by: equivalence-baseline
slice: cli-ux
last_updated: 2026-06-11
golden_total: 8
golden_pass: 8
golden_fail: 0
divergence_count: 3
equivalence_gate_pass: true
baseline_dir: docs/baseline/2026-06-11/cli-ux-self-host/
query_anchors:
  - "self-host golden 过了几条？"
  - "doctor provenance 算 golden 吗？"
---

# 等价性基线报告 — cli-ux-self-host-equivalence-report

## Summary

| 指标 | 值 |
|---|---|
| Slice | cli-ux（self-host Phase 1 / PROJ-10）|
| Golden 总数 | 8 |
| ✅ pass | 8 |
| **equivalence_gate_pass** | true |

## Golden 清单

| ID | 脚本 | 结果 |
|---|---|---|
| G-001 | golden-001-semantic-help.sh | PASS |
| G-002 | golden-002-issue-start-mock.sh | PASS |
| G-003 | golden-003-doc-artifact.sh | PASS |
| G-004 | golden-004-stage-complete.sh | PASS |
| G-005 | golden-005-admin-nested.sh | PASS |
| G-006 | golden-006-removed-commands.sh | PASS |
| G-007 | golden-007-smoke-workflow.sh | PASS |
| G-008 | golden-008-doctor-provenance.sh | PASS |

## 运行结果

```
bash docs/baseline/2026-06-11/cli-ux-self-host/run-all.sh → exit 0
```

## Divergence

| ID | 行为 | Legacy | New | ADR |
|---|---|---|---|---|
| D-001 | Workspace store | SQLite popsicle.db | TSV self-host | ADR-010 / PROJ-11 |
| D-002 | Issue numbering | legacy DB PROJ-N | self-host PROJ-N | ADR-010 |
| D-003 | CLI command count | ~22 commands | IDD MVP subset | ADR-008 |

## 门禁判定

- [x] ≥5 golden pass（8/8）
- [x] baseline README 可复现
- [x] equivalence_gate_pass true
