---
agent_context: [Project preferences]
- 界面 / Agent 语言：简体中文
- 产品目录：`products/`
- ADR：`products/<product>/decisions/adr/`
- PDR：`products/<product>/decisions/pdr/`
- Pipeline 审批：delegate-dangerous（危险操作需审批（其余代批））
- 非危险 `requires_approval` 阶段可由 agent 代批完成；危险阶段（`cutover`、`living-docs`）仍需用户显式 `--confirm`。
doc_type: cutover-author
id: doc-96
pipeline_run_id: 0000002a-0000-402a-8002-2a00000000002a
status: active
title: PROJ-42 Roadmap workflow cutover ADR
version: 1
artifact: cutover-adr
slug: proj-42-cutover
generated_by: cutover-author
slice: cli-ux
last_updated: 2026-06-11
cutover_adr: products/cli-ux/decisions/adr/ADR-022-roadmap-workflow-enhancements.md
query_anchors:
  - "PROJ-42 切流三门禁过了吗？"
  - "ADR-022 状态？"
---

# 切流 ADR 报告 — PROJ-42

## Summary

| 项 | 值 |
|---|---|
| Slice | `cli-ux`（增量，非新 slice）|
| Cutover ADR | [ADR-022](../../products/cli-ux/decisions/adr/ADR-022-roadmap-workflow-enhancements.md) |
| ADR Status | **Proposed**（待 pipeline `--confirm` 后 Accepted）|
| Equivalence | 5/5 pass（doc-95）|
| Implementation | doc-94 |

## 三门禁核对

| # | 门禁 | 证据 | 结果 |
|---|---|---|---|
| 1 | Intent Z3 | `intent-validate path=products/cli-ux` exit 0 | ✅ pass |
| 2 | Golden ≥5 | `cli-ux-roadmap-workflow/run-all.sh` 5/5 | ✅ pass |
| 3 | cargo test | `make check` exit 0 | ✅ pass |

豁免：无。

## 切流决策

1. `migration/traceability.md` PROJ-42 行：`in-shadow` → `cutover-done`（`--confirm` 后执行）
2. `migration/progress.md` cli-ux 备注追加「PROJ-42 Roadmap P1–P6 ✓」
3. ADR-022 Status：Proposed → Accepted（与用户 `--confirm` 同步）

## 已知 Divergence

（无）

## Cutover Gate Checklist

- [x] intent gate 已核对
- [x] equivalence gate 已核对（golden_pass=5）
- [x] cargo test 已核对
- [x] ADR-022 草稿已写入 `products/cli-ux/decisions/adr/`
- [x] 未通过项 blocker 列表为空

## Waiver Checklist

- [x] 无豁免 — N/A

## 待 `--confirm` 后执行

- [ ] `popsicle pipeline stage complete cutover --run 0000002a-0000-402a-8002-2a00000000002a --confirm`
- [ ] ADR-022 Status → Accepted + Approval 字段
- [ ] `migration/traceability.md` 状态列更新

## 检查清单

- [x] Context / Decision / Consequences / Compliance 已在 ADR-022
- [x] 切流范围已写明（cli-ux 增量）
- [x] divergence 已登记（无）
- [ ] Approval 与 pipeline `--confirm` 一致（待用户）
