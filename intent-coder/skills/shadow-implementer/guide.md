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

## 进入 implement 前（硬门禁）

在 `slice-delivery` 的 **implement** stage 写代码之前，确认：

1. **Pipeline 选对**：若 PRD/intent 未覆盖本能力 → 先跑 `slice-spec` 或 `greenfield-product-spec`，不要直接 delivery。见 [`issue-author`](../issue-author/guide.md)。
2. **Intent**：`products/<slice>/intents/` 存在与本任务对应的 acceptance block；`intent-validate` 无新增失败。
3. **ADR**：File Manifest 列出将创建/修改的路径；无清单外文件。
4. **模块**：`.popsicle/modules/intent-coder/` 已安装（`popsicle init` 或 `admin sync-intent-coder`），skill 模板与 guide 可引用。

缺一项 → 停止实现，回到 spec 链或登记 divergence ADR。

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

## Issue ↔ Task 关联（implement 末）

在 `implementation-coverage` 提交前检查 Issue 的 task 关联（`popsicle issue show <key> --format json` 的 `task_link_*`）：

- [ ] 实现覆盖的既有 task 已 **linked**
- [ ] 暴露的新用户旅程登记为 **proposed**（`--proposed-task` 或后续 `living-doc` 晋升）
- [ ] 勿依赖固定 5 条 Guidance 启发式代替显式关联

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
