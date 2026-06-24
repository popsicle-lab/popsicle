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

## Cutover Gate Checklist

- [ ] intent gate 已核对（附 report 日期 / consecutive_clean_runs）
- [ ] equivalence gate 已核对（附 golden_pass 数）
- [ ] cargo test 已核对（附 exit code）
- [ ] 未通过项已列明 blocker

## Waiver Checklist

- [ ] 用户书面确认豁免哪一门禁（无豁免时写 N/A 并勾选）
- [ ] 豁免理由写入 ADR § Compliance（无豁免时写 N/A 并勾选）
- [ ] 补偿措施已列出（如「仅 in-shadow，不切 CLI」；无豁免时写 N/A 并勾选）

## Migration

切流后 legacy 该范围进入 **Sunset 候选**（物理删除另开 ADR，不在本 ADR 范围）。

## 检查清单

- [ ] Context / Decision / Consequences / Compliance 已填写
- [ ] 切流范围列出 legacy ↔ new 路径
- [ ] 已知 divergence 已登记（无则写「无」）
- [ ] Approval 状态与 pipeline `--confirm` 一致

## Approval

- **Status**: Proposed
- **Approved by**: （待 `pipeline stage complete cutover --confirm`）
- **Approval date**:
