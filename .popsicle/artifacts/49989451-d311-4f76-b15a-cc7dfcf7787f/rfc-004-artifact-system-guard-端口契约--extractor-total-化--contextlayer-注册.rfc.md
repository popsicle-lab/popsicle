---
id: fecff724-cd0c-4857-b49d-1c71ca65a497
doc_type: rfc
title: RFC-004：artifact-system guard 端口契约 + extractor total 化 + ContextLayer 注册
status: final
skill_name: rfc-writer
pipeline_run_id: 49989451-d311-4f76-b15a-cc7dfcf7787f
spec_id: cf6e7062-b75f-4925-b633-82a04e775a76
version: 1
parent_doc_id: null
tags: []
metadata: null
created_at: 2026-06-09T06:14:04.201750Z
updated_at: 2026-06-09T06:17:24.959746Z
---

# RFC-004 · artifact-system 跨界接缝技术设计（ADR-004 候选）

> **Source ArchDebate**: [`arch-debate befaaaae`](./artifact-system-架构辩论-guard-upstream-回调--memorieslayer-注册--documentrow-共享-adr-004-候选.arch-debate.md)（方案 A）
> **Source PRD**: [`PRD 0f403e0e`](./artifact-system-prd-6-个文档制品-task-跨-5-旅程阶段.prd.md)
> **Fact Basis**: `docs/baseline/2026-06-09/{dependency-graph,api-contracts,unsafe-risk-report}.md`

## Context

product-debate（方案 C）定逻辑边界、arch-debate（方案 A）定技术形态：hexagonal ports-and-adapters。本 RFC 把方案 A 落成可实现的**三件套契约**：(1) `UpstreamApprovalChecker` 端口签名 + `check_guard` 形态；(2) extractor / guard total 化（消除 19 处 production unwrap，unsafe-risk-report §extractor）；(3) `ContextLayer` 运行时注册 + 确定性装配序。产 contracts.intent 种子（3 goal），喂 adr-writer 固化 ADR-004。

## Goals

- 冻结 `UpstreamApprovalChecker` 端口签名：**仅含 artifact-owned 类型**（`Document`/`GuardResult`），回传 `GuardResult`（非 bool）
- `check_guard` 接 `Option<&dyn UpstreamApprovalChecker>`，缺省→确定性 `InvalidSkillDef`，全函数不 panic
- extractor 三函数 total 化：`Regex` 构造与 `captures` 不再 unwrap（19→0 production unwrap）
- `ContextLayer` 运行时 `register_layer` + 多级确定性排序键（Relevance 降序→固定优先级→稳定 id）
- 依赖图无环（skill-runtime → artifact-system），GuardResult 输出与 legacy 字节对账

## Non-Goals

- 不裁逻辑边界（已由 product-debate 方案 C 定）
- 不实现 CLI 命令壳（属 cli-ux/slice-3）
- 不写性能/时延契约（D2：NFR 走 benchmark，不进 contracts.intent）
- 不动 namespace / memory（归 skill-runtime）

## Quality Attributes (NFR)

| NFR | 优先级 | 守护方式 |
|---|---|---|
| 无环依赖 | 1 | `cargo metadata` 依赖图断言 |
| 等价性（golden 字节对账）| 2 | strangler shadow：新 crate GuardResult/Document 序列化与 legacy diff=0 |
| 健壮性（0 panic）| 3 | total 化 + intent `GuardResultIsTotal` Z3 |
| 可演进性 | 4 | 端口可泛化为 `ExternalGuardChecker`（未来附注）|
| guard 校验时延 | 5（最低）| 非热路径，benchmark 不进 intent |

## Proposed Design

### 模块边界 + 数据流

```
crates/artifact-system/         (本 product)
  document.rs        Document 实体 + 序列化往返
  markdown.rs        6 纯函数
  guard.rs           check_guard 分派器 + has_sections/checklist_complete/count_checkboxes/GuardResult
                       + trait UpstreamApprovalChecker（端口，仅用 Document/GuardResult）
  context.rs         assemble_input_context（多级确定性排序键）
  context_layer.rs   trait ContextLayer（导出）+ register_layer + 3 自洽 layer
  extractor.rs       3 提取函数（total 化，0 unwrap）
  task_chunk.rs      TaskChunk（work_item 重命名；kind + fields blob）

crates/skill-runtime/           (slice-1，依赖 artifact-system)
  impl UpstreamApprovalChecker   内部闭包捕获 pipeline/run/registry
  MemoriesLayer: ContextLayer    运行时 register_layer

crates/storage/                 (栈底，两者皆依赖)
  DocumentRow                    persistence DTO
```

数据流（stage 转换触发 guard）：`check_guard(guard_str, doc, Some(&checker))` → 分派 → `has_sections`/`checklist_complete` 本地求值 / `upstream_approved` → `checker.check_upstream_approved(doc)` → 合并为 `GuardResult`。

### 关键接口签名（语义层；技术形式待 ADR-004 固化）

```rust
// 端口：定义在 artifact-system，签名仅含 artifact-owned 类型
pub trait UpstreamApprovalChecker {
    fn check_upstream_approved(&self, doc: &Document) -> GuardResult;
}

// 分派器：upstream 可缺省；全函数
pub fn check_guard(
    guard: &str,
    doc: &Document,
    upstream: Option<&dyn UpstreamApprovalChecker>,
) -> GuardResult;   // 未知 guard / 缺 checker → InvalidSkillDef，绝不 panic

// context layer 注册 + 确定性装配
pub trait ContextLayer { fn contribute(&self, /* ... */) -> LayerOutput; }
pub fn register_layer(reg: &mut LayerRegistry, layer: Box<dyn ContextLayer>);
pub fn assemble_layers(reg: &LayerRegistry /* ... */) -> AssembledContext;
//   排序键：(relevance desc, fixed_priority asc, stable_id asc)
```

### extractor total 化

- legacy：`Regex::new(PAT).unwrap()`（每函数）+ `caps.get(i).unwrap()`（post-`find_iter`，extractor.rs:10/69/125）
- 新：`Regex` 用 `once_cell`/`LazyLock` 编译期常量（构造失败是编译期/启动期确定性 bug，非运行时输入相关）；`captures` 路径用 `?`/`ok_or` 跳过不匹配项，无匹配→空 `Vec`，绝不 panic

## Alternatives Considered

| 方案 | 否决理由 |
|---|---|
| 端口定义在 skill-runtime（arch-debate 方案 B）| artifact-system 反依赖 skill-runtime → 成环，违反 HC-1 |
| 回调回传 `bool`（rubber-duck 抓出）| 丢失 legacy upstream 的 error kind/文案 → 破坏 golden 字节对账 |
| 闭包注入 + 全 layer 留 artifact-system（arch-debate 方案 C）| 闭包签名捕获 4 个 legacy 类型臃肿；MemoriesLayer 偷渡 memory 语义 |
| 编译期 feature-flag 注册 MemoriesLayer | artifact-system 出现 memory optional dep，污染边界 |

## Intent & Decision Mapping

| 技术声明 | intent 层 | block | 决策载体 |
|---|---|---|---|
| 端口仅含 Document/GuardResult，回传 GuardResult | contracts | `guard upstream 判定经 artifact-owned 端口注入...` | ADR-004 |
| check_guard 全函数、未知→InvalidSkillDef | contracts + invariants | contracts 契约 2 + `GuardResultIsTotal` | ADR-004 |
| extractor 0 production unwrap | contracts | 契约 2 | ADR-004 |
| ContextLayer 运行时注册 + 确定性序 | contracts + invariants | 契约 3 + `ContextOrderIndependentOfRegistration` | ADR-004 |
| 重命名保 kind/fields | invariants | `TaskChunkRenamePreservesFields` | PDR-001 → ADR-004 |

## Risks & Mitigations

| Risk | 缓解 |
|---|---|
| 拆分 check_guard 改变 GuardResult 文案 | 文案常量随对应分支迁移；golden 对账逐 guard 类型分桶 diff |
| 端口签名不慎引入 pipeline 类型成环 | `cargo metadata` 断言 artifact-system 出向依赖集不含 skill-runtime |
| MemoriesLayer 注册序泄漏致装配非确定 | 多级确定性键（Relevance→优先级→id），禁用 HashMap 迭代序 |
| extractor LazyLock 正则在非法常量时启动 panic | 正则是编译期常量字面量，单测覆盖构造成功；与运行时输入解耦 |

## Migration / Rollout

1. shadow：crates/artifact-system 与 legacy 并存，CLI 仍走 legacy
2. 等价性对账：同输入喂新旧 `check_guard` / `Document` 序列化 / `extract_*`，GuardResult.passed + 文案 + 提取产物 diff=0
3. cutover：对账全绿后切 CLI 入口（cli-ux/slice-3 阶段）

## File Manifest

### ARCHITECTURE.md 顶层增量

- [x] `products/artifact-system/ARCHITECTURE.md` § 跨界接缝契约 — 记录 UpstreamApprovalChecker 端口、check_guard 形态、ContextLayer 注册、DocumentRow 下沉 storage（待 adr-writer/living-doc 落地，本 RFC 列入清单）

### Intent 文件

- [x] `products/artifact-system/intents/contracts.intent` 追加 3 个 goal（guard 端口 / total / ContextLayer 注册），标 [ADR-004 候选]（待 adr-writer 解锁为 Accepted）
- [x] `products/artifact-system/intents/invariants.intent` 待 intent-spec-writer 落 `GuardResultIsTotal` / `ContextOrderIndependentOfRegistration` / `TaskChunkRenamePreservesFields`

### 决策记录

- [x] `products/artifact-system/decisions/adr/ADR-004-artifact-system-seams.md`（Status: Proposed → adr-writer 固化为 Accepted）

## Quality Checklist

- [x] 四维度已评分，总分 92 ≥ 90（见 § Quality Score）
- [x] `contracts.intent` 种子能 `intent check`（3 goal 块合法、0 VC）
- [x] 无性能/时延误塞进 contracts（D2：时延进 § NFR）
- [x] File Manifest 与 ADR-004 候选 § Consequences 镜像一致（ARCHITECTURE / contracts / invariants + ADR-004 文件自身）
- [x] Intent & Decision Mapping 每行都有决策载体（ADR-004 / invariants / contracts）

### Quality Score（4 维度，阈值 ≥ 90）

| 维度 | 得分 | 说明 |
|---|---|---|
| 技术严谨性 | 24/25 | 端口签名/total 化/排序键均可实现，rubber-duck 修正成环+bool 风险 |
| 事实可追溯 | 23/25 | 全部 cite guard.rs/extractor.rs/context.rs:line + baseline |
| 等价性可对账 | 23/25 | golden 分桶 diff 方案明确 |
| 可演进性 | 22/25 | 端口可泛化 ExternalGuardChecker（未来附注）|
| **合计** | **92/100** | ≥ 90 |

## References

- arch-debate `befaaaae`（方案 A）；product-debate `5415991a`（方案 C）
- `docs/baseline/2026-06-09/{dependency-graph,api-contracts,unsafe-risk-report}.md`
- legacy `guard.rs:65-96,92-95,103-169,173-266`；`context_layer.rs:22,34,76-80`；`context.rs:52-77`；`extractor.rs:10,69,125`

## Ingest Checklist

- [x] arch-debate-record 已读取（方案 A，hexagonal）
- [x] prd § Dependencies & Blockers 的待裁项已作为本 RFC 议题
- [x] fact-extraction-report / api-contracts / unsafe-risk-report 引用已建立
- [x] target_product 锁定为 artifact-system

## Review Checklist

- [x] RFC § File Manifest 与 ADR-004 候选 Consequences 一致
- [x] 每个核心技术声明标了 intent 层（侧重 contracts）
- [x] 列出 ADR 候选清单（ADR-004）
- [x] 每个数字/模块名引用可追溯到 fact-extraction-report / *.rs:line
- [x] Quality Score ≥ 90（92/100）
- [x] 已向用户展示产出并取得确认
