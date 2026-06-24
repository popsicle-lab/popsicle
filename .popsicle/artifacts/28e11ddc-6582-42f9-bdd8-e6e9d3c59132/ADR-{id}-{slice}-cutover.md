---
id: d1231bb2-1b88-429c-a422-47a0c8f21b67
doc_type: cutover-adr
title: ADR-005 skill-runtime cutover
status: final
skill_name: cutover-author
pipeline_run_id: 28e11ddc-6582-42f9-bdd8-e6e9d3c59132
spec_id: df11b6f9-e504-42b7-8d27-700d529c2346
version: 1
parent_doc_id: null
tags: []
metadata: null
created_at: 2026-06-09T09:09:38.714805Z
updated_at: 2026-06-09T09:09:42.454377Z
---

# ADR-{id} · {slice-name} cutover（in-shadow → cutover-done）

> **Status**: Proposed
> **Date**: {date}
> **Product**: {slice-name}
> **Generated-by**: cutover-author
> **Source-Equivalence**: {slug}.equivalence-report.md
> **Source-Coverage**: {slug}.implementation-coverage.md

## Context

本 slice 已完成 in-shadow 实现（`crates/{slice-name}/`）与 golden 对账。
legacy `popsicle` 在该范围内仍为并行主路径；本 ADR 授权将
`migration/progress.md` 中本 slice 标为 **cutover-done**，并登记 traceability。

## Decision

1. **切流范围**：（列出 legacy 路径前缀 ↔ `crates/{slice-name}/` 模块）
2. **主路径切换**：（CLI 仍走 legacy / 部分命令切 new / 仅 lib 消费——写清）
3. **已知 divergence**：（引用 equivalence-report §Divergence；无则写「无」）

## Alternatives

| 方案 | 否决理由 |
|---|---|
| 继续 in-shadow 不切 | 阻塞后续 slice 依赖本 slice 端口 |
| 无 golden 硬切 | 违反 CONTRIBUTING §4 |

## Consequences

- `migration/progress.md` —— slice 状态 → `cutover-done`
- `migration/traceability.md` —— 追加本表行（baseline 路径、ADR 引用）
- `products/{slice}/ARCHITECTURE.md` —— File Manifest 状态列更新（living-doc-author）
- （若适用）`docs/MIGRATION.md` —— 人类工程师心智模型增量

## Compliance

| 门禁 | 证据 | 结果 |
|---|---|---|
| Intent Z3 gate_ready | intent-consistency-report {date} | pass / fail |
| Equivalence ≥5 golden | equivalence-report {date} | pass / fail |
| cargo test | implementation-coverage {date} | exit  |

## Migration

切流后 legacy 该范围进入 **Sunset 候选**（物理删除另开 ADR，不在本 ADR 范围）。

## Approval

- **Status**: Proposed
- **Approved by**: （待 `pipeline stage complete cutover --confirm`）
- **Approval date**:
