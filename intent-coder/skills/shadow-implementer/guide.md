# shadow-implementer 使用指南

把 **spec（intent + ADR）** 落到 **`crates/<slice>/` in-shadow 实现**。这是
`slice-delivery` pipeline 的第一棒，衔接 `intent-consistency-check` 与
`equivalence-baseline`。

```
slice-spec / migration-bootstrap（spec 完成）
    → shadow-implementer（本 skill）
    → equivalence-baseline（golden 对账）
    → cutover-author（切流 ADR）
    → living-doc-author（实现态保活）
```

## 定位

| 做 | 不做 |
|---|---|
| 按 ADR File Manifest 写/改 `crates/<slice>/` | 发明 scope 外 API |
| 每个 acceptance block → property test | 替代 equivalence 的 golden 对账 |
| 端口 trait 放对 crate、实现放对 crate | 自动切流（那是 cutover-author） |
| 产出 implementation-coverage 映射表 | 改 `.intent` 语义（走 PDR/intent-spec） |

## 输入

- `products/<slice>/intents/*.intent`（已 Z3 verified）
- `products/<slice>/decisions/adr/*.md`（Accepted）§ Consequences / File Manifest
- `products/<slice>/ARCHITECTURE.md`
- 已有 `crates/<slice>/`（可增量）

## 输出

- `{slug}.implementation-coverage.md`：intent → fn/test 1:1 表
- 副作用：代码落在 `crates/<slice>/` + `tests/intent_properties.rs`

## 硬规则

1. **清单之外不创建**：路径以 ADR/RFC File Manifest 为准。
2. **依赖方向**：`skill-runtime → artifact-system → storage`，无环。
3. **测试追溯**：每个 acceptance `intent` 至少一条 property test。
4. **可多轮**：implementing ↔ verifying 可反复，不要求一次 PR 全做完。

## 与 legacy 的关系

本 skill 产出的是 **in-shadow** 实现——legacy 仍是主路径，直到
`equivalence-baseline` + `cutover-author` 通过。故意语义改进（如
`DocumentRoundTrips` 要求 body 字节精确）须在 coverage 报告「待办」里
注明，交给 equivalence 登记 divergence。
