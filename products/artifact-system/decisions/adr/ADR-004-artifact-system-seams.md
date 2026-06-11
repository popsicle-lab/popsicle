# ADR-004 · artifact-system 跨界接缝（guard 端口 / ContextLayer 注册 / DocumentRow 下沉）

> **Status**: Accepted
> **Date**: 2026-06-09
> **Product**: artifact-system
> **Generated-by**: rfc-writer（骨架）→ adr-writer 固化
> **Source-RFC**: rfc-004-artifact-system-guard-端口契约--extractor-total-化--contextlayer-注册.rfc.md
> **Source-Debate**: artifact-system-架构辩论（arch-debate befaaaae，方案 A）
> **Source-PDR**: PDR-001-artifact-system-tasks-mvp.md（scope）；product-debate 5415991a（方案 C 边界）
> **CADR**: 否（仅 artifact-system / skill-runtime 两 product 间内部接缝，不触 charter 四铁律 / 全局 Layer Map）

## Context

product-debate（方案 C）定逻辑边界：artifact-system 持 guard 纯文档校验，upstream_approved 回调 / MemoriesLayer / namespace 归 skill-runtime。arch-debate（方案 A，hexagonal ports-and-adapters）定技术形态。本 ADR 固化方案 A 为 Accepted，解锁 RFC-004 的 3 个 contracts goal。

## Decision

1. **guard upstream 端口**：artifact-system 定义 `trait UpstreamApprovalChecker { fn check_upstream_approved(&self, doc: &Document) -> GuardResult }`——签名**仅含 artifact-owned 类型**（Document/GuardResult），**回传 GuardResult 而非 bool**。`check_guard(guard, doc, Option<&dyn UpstreamApprovalChecker>)` 全函数；未知 guard / 缺 checker → 确定性 `InvalidSkillDef`，绝不 panic。skill-runtime 实现端口、内部闭包捕获 pipeline/run/registry。
2. **ContextLayer 注册**：artifact-system 导出 `ContextLayer` trait + 3 自洽 layer；skill-runtime 运行时 `register_layer` 注入 `MemoriesLayer`。装配按多级确定性键排序（Relevance 降序 → 固定优先级 → 稳定 id），不依赖 HashMap 迭代序。artifact-system 零 memory 编译依赖。
3. **DocumentRow 下沉**：移入栈底 `storage` crate，artifact-system / skill-runtime 皆依赖之。
4. **extractor total 化**：19 处 production unwrap → 0（`LazyLock` 正则常量 + `?`/`ok_or`）。

依赖方向 skill-runtime → artifact-system → storage，**无环**。

## Alternatives

| 方案 | 否决理由 |
|---|---|
| 端口定义在 skill-runtime（arch-debate B）| artifact-system 反依赖 skill-runtime → 成环（违反 HC-1）|
| 回调回传 bool | 丢 legacy upstream error kind/文案 → 破坏 golden 字节对账 |
| 闭包注入 + 全 layer 留 artifact-system（arch-debate C）| 闭包捕获 4 legacy 类型臃肿；MemoriesLayer 偷渡 memory 语义 |
| 编译期 feature-flag 注册 MemoriesLayer | artifact-system 出现 memory optional dep，污染边界 |

## Consequences

> 与 RFC-004 § File Manifest 镜像一致（落地物：ARCHITECTURE / contracts / invariants + ADR-004 文件自身不自列）。

- `products/artifact-system/intents/contracts.intent` 3 个 goal 解锁 [ADR-004 候选] → [ADR-004 Accepted]
- `products/artifact-system/intents/invariants.intent` 待 intent-spec-writer 落 `GuardResultIsTotal` / `ContextOrderIndependentOfRegistration` / `TaskChunkRenamePreservesFields`
- `products/artifact-system/ARCHITECTURE.md` § 跨界接缝契约 待 living-doc-author 记录端口/注册/下沉
- 实现期：`crates/artifact-system/`（guard 端口 + 纯校验 + context_layer trait + extractor total）、`crates/storage/`（DocumentRow）、`crates/skill-runtime/`（端口实现 + MemoriesLayer 注册）

## Migration

shadow 并存 → golden 分桶对账（GuardResult.passed + 文案 + Document 序列化 + extract 产物 diff=0）→ cutover（cli-ux/slice-3）。

## Compliance

- 不触 charter 四铁律 / 全局 Layer Map → 非 CADR
- 依赖图无环（`cargo metadata` 断言 artifact-system 出向依赖不含 skill-runtime）

## Approval

- **Status**: Accepted
- **Approved by**: @curtiseng（经 `pipeline stage complete adr --confirm`）
- **Approval date**: 2026-06-09
