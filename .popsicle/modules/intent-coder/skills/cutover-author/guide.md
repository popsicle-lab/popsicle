# cutover-author 使用指南

单个 slice 的 **Strangler Fig 切流审批闸**。核验三门禁后固化切流 ADR，
更新 `migration/progress.md` 与 `migration/traceability.md`。

## 三门禁

| # | 门禁 | 证据 |
|---|---|---|
| 1 | Intent Z3 | `intent-consistency-report`：`gate_ready=true` 或连续 3 次 pass |
| 2 | Golden 等价 | `equivalence-report`：`equivalence_gate_pass=true` |
| 3 | 构建 | `cargo test -p <slice>` exit 0 |

豁免须用户明确确认，并写入 ADR § Compliance。

## 产出

- `products/<slice>/decisions/adr/ADR-XXX-<slice>-cutover.md`（Accepted）
- `migration/progress.md` 状态 → `cutover-done`
- `migration/traceability.md` 正式行

## 与 living-doc-author 的分工

cutover 改**决策与看板**；living-doc 改**活文档实现态**
（tasks/README 已实施列、ARCHITECTURE File Manifest、PRODUCT 双行头）。

## 红线

- 三门禁未过且未豁免 → 不 Accepted
- 不修改 legacy submodule（sunset 另开 ADR）
- ADR Accepted 后依 charter 不可改——错了写 Supersedes ADR
