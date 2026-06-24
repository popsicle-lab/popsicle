---
doc_type: equivalence-baseline
id: doc-99
pipeline_run_id: 0000002b-0000-402b-8002-2b00000000002b
status: active
title: PROJ-43 equivalence baseline
version: 1
artifact: equivalence-report
slug: proj-43
golden_total: 5
golden_pass: 5
golden_fail: 0
divergence_count: 0
equivalence_gate_pass: true
baseline_dir: docs/baseline/2026-06-11/cli-ux-issue-tasks/
---

# 等价性基线报告 — PROJ-43

## Summary

| 指标 | 值 |
|---|---|
| Golden 总数 | 5 |
| pass | 5 |
| equivalence_gate_pass | true |

## Golden Inventory

- [x] slice 已确认
- [x] ≥5 条 golden 已列出
- [x] baseline.yaml 一致
- [x] divergence 无

## Golden 清单

| ID | 脚本 | 结果 |
|---|---|---|
| G-001 | golden-001-issue-tasks-persist.sh | PASS |
| G-002 | golden-002-epic-migrate.sh | PASS |
| G-003 | golden-003-guidance-linked.sh | PASS |
| G-004 | golden-004-issue-author-skill.sh | PASS |
| G-005 | golden-005-ui-task-links.sh | PASS |

运行：`bash docs/baseline/2026-06-11/cli-ux-issue-tasks/run-all.sh`
