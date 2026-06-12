# Retro 文档债清单（已交付增量）

> **何时用**：代码已合并，但 `products/<product>/` 里还没有对应 PDR / task / intent。
> **不要用** `slice-spec` pipeline（facts 阶段面向 legacy 迁移，不是 retro）。

## Checklist

- [ ] 在 `products/<product>/decisions/pdr/` 写 PDR，含 **Intent Mapping** 表
- [ ] 在 `products/<product>/tasks/<journey>/` 新增或更新 task 文件
- [ ] 在 `products/<product>/intents/acceptance.intent` 增加 acceptance block（`task_id` 双射）
- [ ] `popsicle tool run intent-validate path=products/<product>/intents` 通过
- [ ] 若实现先于 spec，在 cutover ADR 或 Divergence 表登记
- [ ] `living-doc-author --target implementation-status,product-header`（可选）

## Issue 追踪（可选）

可建 Issue **不启动 pipeline**，或在描述里写 `retro` 便于 UI 提示；完成后 `issue close`。

## 参考

- [`issue-author`](../skills/issue-author/guide.md) § 已交付能力补 spec
- [`living-doc-author`](../skills/living-doc-author/guide.md) — 保活对账
