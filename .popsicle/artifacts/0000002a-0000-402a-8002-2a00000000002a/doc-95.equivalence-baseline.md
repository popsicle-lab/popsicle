---
agent_context: [Project preferences]
- 界面 / Agent 语言：简体中文
- 产品目录：`products/`
- ADR：`products/<product>/decisions/adr/`
- PDR：`products/<product>/decisions/pdr/`
- Pipeline 审批：delegate-dangerous（危险操作需审批（其余代批））
- 非危险 `requires_approval` 阶段可由 agent 代批完成；危险阶段（`cutover`、`living-docs`）仍需用户显式 `--confirm`。
doc_type: equivalence-baseline
id: doc-95
pipeline_run_id: 0000002a-0000-402a-8002-2a00000000002a
status: active
title: PROJ-42 Roadmap workflow equivalence
version: 1
artifact: equivalence-report
slug: proj-42-roadmap-equivalence
generated_by: equivalence-baseline
slice: cli-ux
last_updated: 2026-06-11
golden_total: 5
golden_pass: 5
golden_fail: 0
divergence_count: 0
equivalence_gate_pass: true
baseline_dir: docs/baseline/2026-06-11/cli-ux-roadmap-workflow/
baseline_manifest: docs/baseline/2026-06-11/cli-ux-roadmap-workflow/baseline.yaml
query_anchors:
  - "PROJ-42 golden 对账过了几条？"
  - "Roadmap P1-P6 有无 divergence？"
---

# 等价性基线报告 — PROJ-42 Roadmap workflow

> Greenfield/retro 增强：无 legacy 字节 parity 要求；5 条 golden 实跑通过。

## Summary

| 指标 | 值 |
|---|---|
| Slice | `cli-ux` |
| Legacy pin | `c76d729…`（见 LEGACY_PIN.md）|
| Golden 总数 | 5 |
| ✅ pass | 5 |
| ❌ fail | 0 |
| ⚠️ divergence（已 ADR 登记）| 0 |
| **equivalence_gate_pass** | **true** |

门禁：`golden_pass >= 5` → **pass**。

## Golden Inventory

- [x] slice 已确认：`cli-ux-roadmap-workflow`
- [x] ≥5 条 golden 已列出
- [x] 每条追溯到 README Roadmap P1–P6
- [x] 无 divergence

## Baseline Manifest

- [x] `baseline.yaml` 已创建
- [x] Summary 计数与 `baseline.yaml` 一致
- [x] Golden 清单状态与 `baseline.yaml` 一致

## Golden 清单

| ID | 描述 | 脚本 | Legacy | New | 结果 | diff 摘要 |
|---|---|---|---|---|---|---|
| G-001 | WorkflowProfile 默认 pipeline | `golden-001-workflow-profile.sh` | 单一默认 | `workflow.profile` 驱动 | PASS | 新能力 |
| G-002 | epic_task_id 持久化 | `golden-002-epic-task-persist.sh` | 无 epic 列 | SQLite+TSV+CLI | PASS | |
| G-003 | product health 扫描 | `golden-003-product-health.sh` | 无仪表盘 | `scan_product_health` | PASS | |
| G-004 | retro checklist | `golden-004-retro-checklist.sh` | 无 | guide + RetroDocBanner | PASS | |
| G-005 | UI 组件 + build | `golden-005-ui-workflow.sh` | 扁平列表 | 分组/mermaid/health | PASS | UI-only |

## 运行结果

```
bash docs/baseline/2026-06-11/cli-ux-roadmap-workflow/run-all.sh
exit 0 — All cli-ux-roadmap-workflow golden baselines passed.
```

已链接进 `docs/baseline/2026-06-11/cli-ux-sqlite-phase2/run-all.sh` 链尾。

## Traceability（已写入 migration/traceability.md）

| Legacy 路径 | 新位置 | 责任 Spec | 切流 ADR | 等价性 baseline | 状态 |
|---|---|---|---|---|---|
| （无 legacy 等价）| `project_config.rs` + `workspace_readers.rs` + `ui/` | cli-ux | PROJ-42 待 cutover | `docs/baseline/2026-06-11/cli-ux-roadmap-workflow/` | in-shadow |

## Divergence

（无）

## 门禁判定

- [x] ≥5 golden pass
- [x] baseline 目录已创建且 README 可复现
- [x] traceability 行已起草
- [x] equivalence_gate_pass = true

## 检查清单

- [x] 每条 golden 有实跑证据
- [x] pass/fail 数字与 `baseline.yaml` 一致
- [x] divergence 未隐瞒
- [x] equivalence_gate_pass 可复算
