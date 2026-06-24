---
doc_type: cutover-author
id: doc-72
pipeline_run_id: 00000024-0000-4024-8002-24000000000024
status: active
title: ADR-018 UI modern layout cutover (PROJ-36)
version: 1
---

# ADR-018 · UI modern layout cutover (PROJ-36)

> **Promoted to**: `products/cli-ux/decisions/adr/ADR-018-ui-modern-layout.md`
> **Status**: Accepted
> **Date**: 2026-06-11
> **Source-Equivalence**: doc-71.equivalence-baseline.md

## Context

PROJ-36 完成 `ui/` 布局与交互刷新（sidebar、breadcrumb、Issues/Pipeline/Products
master-detail），无新增 Tauri IPC。等价性 golden 5/5；美学差异 D-701 由 ADR-018 登记。

## Decision

1. **切流范围**：`legacy/popsicle/ui/` 整页导航 → `ui/` modern shell + split layouts
2. **主路径**：DMG / `popsicle ui` 消费新布局；CLI IPC 契约不变（extends ADR-015/016）
3. **Divergence**：D-701 typography/spacing 不在 acceptance.intent；见 ADR-018 §Divergences

## Alternatives

| 方案 | 否决理由 |
|---|---|
| 继续 legacy 14-page shell | PROJ-36 已交付且 golden 全绿 |
| 无 ADR 硬切美学面 | D-701 必须登记 |

## Consequences

- `migration/progress.md` — Last-Decision-Ref ADR-018
- `migration/traceability.md` — ui layout 行 cutover-done
- `products/cli-ux/PRODUCT.md` — ADR-018 committed roadmap（living-doc-author）

## Compliance

| 门禁 | 证据 | 结果 |
|---|---|---|
| Equivalence ≥5 golden | doc-71 + cli-ux-ui-modern/run-all.sh | pass（5/5）|
| UI production build | golden-001 | pass |
| make check | workspace fmt/clippy/test | pass |
| No new IPC | diff scoped to ui/ shell | pass |

## Cutover Gate Checklist

- [x] intent gate 已核对（acceptance blocks unchanged for IPC）
- [x] equivalence gate 已核对（golden_pass 5）
- [x] cargo test / make check 已核对
- [x] 未通过项已列明 blocker（无 blocker）

## Waiver Checklist

- [x] 用户书面确认豁免哪一门禁（N/A — 全部门禁 pass）
- [x] 豁免理由写入 ADR § Compliance（N/A）
- [x] 补偿措施已列出（N/A）

## Migration

legacy `legacy/popsicle/ui/` 布局进入 Sunset 候选；物理删除另开 ADR。

## 检查清单

- [x] Context / Decision / Consequences / Compliance 已填写
- [x] 切流范围列出 legacy ↔ new 路径
- [x] 已知 divergence D-701 已登记
- [x] Approval 状态与 ADR-018 Accepted 一致

## Approval

- **Status**: Accepted
- **Approved by**: PROJ-36 slice-delivery（pipeline cutover --confirm）
- **Approval date**: 2026-06-11
