---
agent_context: [Project preferences]
- 界面 / Agent 语言：简体中文
- 产品目录：`products/`
- ADR：`products/<product>/decisions/adr/`
- PDR：`products/<product>/decisions/pdr/`
- Pipeline 审批：delegate-dangerous（危险操作需审批（其余代批））
- 非危险 `requires_approval` 阶段可由 agent 代批完成；危险阶段（`cutover`、`living-docs`）仍需用户显式 `--confirm`。
doc_type: living-doc-author
id: doc-97
pipeline_run_id: 0000002a-0000-402a-8002-2a00000000002a
status: active
title: PROJ-42 living-doc sync
version: 1
artifact: living-doc-sync-report
slug: proj-42-living-doc
generated_by: living-doc-author
slice: cli-ux
last_updated: 2026-06-11
targets: implementation-status,architecture-manifest,product-header
query_anchors:
  - "PROJ-42 活文档刷了什么？"
  - "tasks/README Roadmap 表在哪？"
---

# 活文档同步报告 — PROJ-42

## Summary

| 指标 | 值 |
|---|---|
| Product | `cli-ux` |
| Targets | `implementation-status`, `architecture-manifest`, `product-header` |
| Drift 修复 | 3 文件刷新 |
| 待人工 | 0（retro 路径；正式 task 化可选）|

## 健康度快照（scan_product_health）

- `cli-ux`：task_count > 0，intent blocks present，PRODUCT.md ✅
- 无 broken-ref / orphan 新增（PROJ-42 未改 task 图拓扑）

## 刷新动作

| 文件 | 变更 |
|---|---|
| `products/cli-ux/PRODUCT.md` | 双行头 → ADR-022；用户入口补 workflow/epic/UI |
| `products/cli-ux/ARCHITECTURE.md` | File Manifest +8 行（ADR-022 路径）|
| `products/cli-ux/tasks/README.md` | Status 行 + Roadmap P1–P6 映射表 |
| `migration/progress.md` | cli-ux 备注 PROJ-42；Last-Decision-Ref |
| `migration/traceability.md` | PROJ-42 → cutover-done + ADR-022 |
| `products/cli-ux/decisions/adr/ADR-022-*.md` | Status Accepted |

## Drift 信号

| 类型 | 计数 | 处置 |
|---|---|---|
| 过期 staleness | 0 | — |
| 断链 broken-ref | 0 | — |
| 孤儿 orphan | 0 | — |
| 未验证 unverified | 0 | intent-validate 无回归 |

## 待人工处置

（无）— 若需正式 acceptance block（如 `WorkflowProfilePersists`），走可选 slice-spec。

## 检查清单

- [x] `tasks/README.md` 已实施状态已更新
- [x] `ARCHITECTURE.md` File Manifest 含 ADR-022 路径
- [x] `PRODUCT.md` 双行头与 Last-Decision-Ref 已刷新
- [x] 未越界修改 task 正文
- [x] traceability / progress 与 cutover 一致
