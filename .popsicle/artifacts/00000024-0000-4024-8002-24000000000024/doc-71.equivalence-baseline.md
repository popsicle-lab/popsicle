---
doc_type: equivalence-baseline
id: doc-71
pipeline_run_id: 00000024-0000-4024-8002-24000000000024
status: active
title: cli-ux UI modern layout equivalence (PROJ-36)
version: 1
artifact: equivalence-report
slug: cli-ux-ui-modern-equivalence
generated_by: equivalence-baseline
slice: cli-ux-ui-modern
last_updated: 2026-06-11
golden_total: 5
golden_pass: 5
golden_fail: 0
divergence_count: 1
equivalence_gate_pass: true
baseline_dir: docs/baseline/2026-06-11/cli-ux-ui-modern/
baseline_manifest: docs/baseline/2026-06-11/cli-ux-ui-modern/baseline.yaml
query_anchors:
  - "legacy 和 new 行为一致吗？"
  - "golden 对账过了几条？"
---

# 等价性基线报告 — cli-ux-ui-modern-equivalence

> 由 `equivalence-baseline` 产出。Issue PROJ-36 · slice-4-ui layout refresh。

## Summary

| 指标 | 值 |
|---|---|
| Slice | cli-ux-ui-modern |
| Legacy pin | `c76d729db91c59009f0fa8f7c6f1e499eb0c7eb1`（见 LEGACY_PIN.md）|
| Golden 总数 | 5 |
| pass | 5 |
| fail | 0 |
| divergence（已 ADR 登记）| 1 |
| **equivalence_gate_pass** | true |

门禁：`golden_pass >= 5` → pass（5/5）。

## Golden Inventory

- [x] slice 已确认（PROJ-36 / slice-4-ui）
- [x] 5 条 golden 已列出（含输入/legacy/new/比较方式）
- [x] 每条能追溯到 PROJ-36 implement 或 ADR-015/016
- [x] 已知 divergence D-701 已单独列出

## Baseline Manifest

本报告数字来自 `docs/baseline/2026-06-11/cli-ux-ui-modern/baseline.yaml`。

- [x] `baseline.yaml` 已创建
- [x] Summary 计数与 `baseline.yaml` 一致（golden_total 5, golden_pass 5）
- [x] Golden 清单状态与 `baseline.yaml` 一致

## Golden 清单

| ID | 描述 | 脚本 | Legacy | New | 结果 | diff 摘要 |
|---|---|---|---|---|---|---|
| G-001 | ui/ Vite production build | `golden-001-ui-build.sh` | legacy/popsicle/ui/ 14-page shell | ui/ modern tokens + split views | PASS | visual refresh; no IPC change |
| G-002 | cargo build --features ui | `golden-002-cargo-ui.sh` | legacy popsicle-cli + ui | crates/cli-ux feature ui | PASS | — |
| G-003 | layout shell modules | `golden-003-layout-shell.sh` | monolithic pages | navigation.ts + sidebar + breadcrumbs | PASS | new layout primitives |
| G-004 | issues master-detail | `golden-004-issues-split.sh` | full-page issue nav | IssuesView split >=1100px | PASS | — |
| G-005 | pipeline + products split | `golden-005-pipeline-split.sh` | stacked pipeline panel | graph + inspector; explorer split | PASS | — |

## 运行结果

```
bash docs/baseline/2026-06-11/cli-ux-ui-modern/run-all.sh
→ exit 0 — All cli-ux-ui-modern golden baselines passed.
```

## Traceability（拟写入 migration/traceability.md）

| Legacy 路径 | 新位置 | 责任 Spec | 切流 ADR | 等价性 baseline | 状态 |
|---|---|---|---|---|---|
| `legacy/popsicle/ui/` 整页导航布局 | `ui/` master-detail + collapsible sidebar | slice-4-ui | ADR-018 | `docs/baseline/2026-06-11/cli-ux-ui-modern/` | cutover-done |

## Divergence

| ID | 行为 | Legacy | New | 原因 | ADR |
|---|---|---|---|---|---|
| D-701 | Visual typography/spacing | legacy ui styling | zinc design system + CSS primitives in ui/src/index.css | Aesthetic constraints not in acceptance.intent; guarded by structural goldens + build | ADR-018 |

## 门禁判定

- [x] 5 golden pass
- [x] divergence D-701 已 ADR Accepted（ADR-018）
- [x] baseline 目录已创建且 README 可复现
- [x] traceability 草稿已写

## 检查清单

- [x] 每条 golden 有实跑证据
- [x] pass/fail 数字与 baseline.yaml / Summary 一致
- [x] divergence 未隐瞒
- [x] equivalence_gate_pass 可复算
