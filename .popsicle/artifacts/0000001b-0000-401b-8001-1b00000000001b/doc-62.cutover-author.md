---
doc_type: cutover-author
id: doc-62
pipeline_run_id: 0000001b-0000-401b-8001-1b00000000001b
status: active
title: PROJ-27 UI cutover ADR-015
version: 1
---

# ADR-015 · cli-ux UI cutover

> 正式正文：`products/cli-ux/decisions/adr/ADR-015-tauri-ui-self-host-bridge.md`

## Context

PROJ-27 Tauri UI MVP+ 完成；golden 4/4；逆转 ADR-014 D-402 限定范围。

## Decision

1. 切流 `ui/` + `crates/cli-ux/src/ui/*` + `workspace_readers.rs`
2. 入口 `popsicle ui`；CI 主路径仍无 ui feature
3. Divergence D-501/D-502 见 ADR-015

## Compliance

| 门禁 | 结果 |
|---|---|
| Intent Z3 | pass |
| Equivalence | pass（4 golden + ADR）|
| make check | pass |
| make build-ui | pass |

## Cutover Gate Checklist

- [x] intent gate
- [x] equivalence gate
- [x] cargo test
- [x] 无 blocker

## Waiver Checklist

- [x] golden<5 由 divergence ADR 补偿（N/A 书面豁免）

## 检查清单

- [x] 范围与 divergence 已登记
- [x] Approval 与 ADR-015 Accepted 一致

## Approval

- **Status**: Accepted
- **Approval date**: 2026-06-11
