---
artifact: port-coverage
slug: {slug}
generated_by: port
slice: {slice-name}
migration_mode: verbatim
legacy_pin: REPLACE_WITH_REAL_LEGACY_PIN
last_updated: {date}
query_anchors:
  - "这段 new 代码是从 legacy 哪里平移来的？"
  - "平移是逐字节等价还是做了适配？"
---

# 平移覆盖 — {slug}

> 由 `port` 产出（feedback S3）。本切片是 **verbatim 平移**：把 legacy 代码原样搬到 new 位置，
> **不改逻辑**。golden 是 characterization（快照自证），等价平凡——如实声明，不假装 legacy↔new 差分，
> 不套「shadow / strangler-fig」叙事（见 shadow-implementer guide 的「verbatim vs shadow」）。

## Port Coverage

> 每段 legacy 源 ↔ new 位置。`verbatim`=逐字节等价；`adapted`=机械适配（须说明，且不得改可观测行为）。

| legacy 源 | new 位置 | 类型 | 说明 |
|---|---|---|---|
| `legacy@<sha>:crates/store-engine/src/lib.rs#L1-L120` | `crates/store/src/lib.rs#L1-L118` | verbatim | 原样 |
| `legacy@<sha>:.../writer.rs#L40-L88` | `crates/store/src/writer.rs#L38-L90` | adapted | 仅改 import 路径，行为不变 |

## Verbatim Justification

> 为什么这些平移是等价的（characterization 而非 diff）。`adapted` 段逐一说明适配点及为何不改行为。

- verbatim 段：逐字节搬运，`cargo build` 通过；characterization golden 快照锁定输出。
- adapted 段：……（若任何 adapted 改变了可观测行为 → 已登记为 divergence 交 cutover，或转 shadow-implementer）

## Port Checklist

- [ ] 已确认 `migration_mode: verbatim`（需真写新逻辑的应改用 shadow-implementer）
- [ ] 每段 new 代码都追回一个 `legacy@<sha>:<path>#L` 源
- [ ] 每段标了 verbatim / adapted；adapted 说明了适配点且不改可观测行为
- [ ] `cargo build` 通过（平移完整）
- [ ] 未发明 scope 外 API；未套用 shadow/strangler-fig 叙事
- [ ] 已声明 golden=characterization（交 equivalence-baseline）
