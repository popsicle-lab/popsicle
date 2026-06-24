---
doc_type: cutover-author
id: doc-54
pipeline_run_id: 00000019-0000-4019-8001-19000000000019
status: active
title: PROJ-25 SQLite Phase 2 cutover ADR
version: 1
---

# PROJ-25 SQLite Phase 2 cutover ADR

> **Promoted to**: `products/cli-ux/decisions/adr/ADR-013-sqlite-phase2-storage.md`
> **Stage**: cutover (slice-delivery)
> **Date**: 2026-06-11

本文档是 ADR-013 的工作副本;正式决策见 promoted 路径。核心:SQLite 单文件
后端(state.db,避开 legacy popsicle.db)、后端自动检测、admin migrate 真迁移
(幂等+留底)、会话 JSON 保持文件形态、doctor 后端动态报告。

## Cutover Gate Checklist

- [x] intent gate:intent-validate exit 0
- [x] equivalence gate:23/23 golden pass
- [x] cargo test 全工作区 79/79
- [x] 真实工作区迁移完成且数据无损(8 issues)
- [x] ADR-013 已 promoted 至 products/cli-ux/decisions/adr/

## Waiver Checklist

- [x] 无豁免
