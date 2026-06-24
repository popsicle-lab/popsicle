---
doc_type: cutover-author
id: doc-50
pipeline_run_id: 00000018-0000-4018-8001-18000000000018
status: active
title: PROJ-24 usability cutover ADR
version: 1
---

# PROJ-24 usability cutover ADR

> **Promoted to**: `products/cli-ux/decisions/adr/ADR-012-self-host-usability-completion.md`
> **Stage**: cutover (slice-delivery)
> **Date**: 2026-06-11

本文档是 ADR-012 的工作副本;正式决策记录见 promoted 路径。核心:doc check 与
issue close 落地、默认管线重映射到 bundled 模板(含新增 bugfix)、模板按需
自愈安装、smoke 隔离与残留清理。

## Cutover Gate Checklist

- [x] intent gate:intent-validate exit 0
- [x] equivalence gate:18/18 golden pass
- [x] cargo test 全工作区 exit 0
- [x] doctor provenance match = true
- [x] ADR-012 已 promoted 至 products/cli-ux/decisions/adr/

## Waiver Checklist

- [x] 无豁免
