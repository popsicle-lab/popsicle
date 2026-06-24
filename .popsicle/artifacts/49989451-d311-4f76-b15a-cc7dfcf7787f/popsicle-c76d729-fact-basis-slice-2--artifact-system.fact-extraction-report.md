---
id: b27c5ea6-5460-4564-9cca-ba2b038f5c7b
doc_type: fact-extraction-report
title: popsicle@c76d729 fact-basis (slice-2 = artifact-system)
status: final
skill_name: fact-extractor
pipeline_run_id: 49989451-d311-4f76-b15a-cc7dfcf7787f
spec_id: cf6e7062-b75f-4925-b633-82a04e775a76
version: 1
parent_doc_id: null
tags: []
metadata: null
created_at: 2026-06-09T02:54:25.418145Z
updated_at: 2026-06-09T03:12:34.114163Z
---

# Fact Extraction Report — popsicle@c76d729（slice-2 = artifact-system）

> **基线日期**：2026-06-09
> **源 commit**：`c76d729db91c59009f0fa8f7c6f1e499eb0c7eb1`（`legacy/popsicle/` submodule，`v0.4.0-19-gc76d729`）
> **抽取者**：fact-extractor（copilot agent / Claude Opus 4.8）
> **范围**：**仅 slice-2 artifact-system 的 8 个 legacy 模块**（不是全仓库；全仓库基线见 slice-1 的 `popsicle-c76d729-fact-basis-slice-1--skill-runtime.fact-extraction-report.md`）。边界由 PDR-001 锁定。

本报告是 artifact-system 切片事实基的**入口**。每个声明都来自 4 份详细 artifact（落在 `docs/baseline/2026-06-09/`）之一；不引入无法追溯到详细 artifact 或源文件的事实。记录员立场：只记事实并 cite file:line，不发表观点（下游 product-debate / rfc-writer 形成观点）。

---

## Summary

| 指标 | 值 | 来源 |
|---|---|---|
| 范围内模块数 | 8（document / namespace / work_item / markdown / guard / context / context_layer / extractor）| [dependency-graph](../../docs/baseline/2026-06-09/dependency-graph.md) |
| 总行数 / Code / Comments / Blanks（tokei）| 2,494 / 2,109 / 118 / 267 | tokei（8 文件）|
| 公开 API 表面（pub fn/struct/enum/trait）| 38 | [api-contracts](../../docs/baseline/2026-06-09/api-contracts.md) |
| 唯一可扩展 `pub trait` | 1（`ContextLayer`，`engine/context_layer.rs:22`）| api-contracts |
| `unsafe` 块 | **0** | [unsafe-risk-report](../../docs/baseline/2026-06-09/unsafe-risk-report.md) |
| `.unwrap()`（总 / 生产 / 测试）| 44 / **19** / 25 | unsafe-risk-report |
| 生产 unwrap 全部位置 | extractor.rs 19 处（全为常量正则编译 + 已命中子串的二次 captures）| unsafe-risk-report |
| `.expect()` / `panic!()` / `todo!()` | 0 / 0 / 0 | unsafe-risk-report |
| TODO/FIXME/HACK/XXX 注释 | **0** | [tech-debt-inventory](../../docs/baseline/2026-06-09/tech-debt-inventory.md) |
| `#[deprecated]` / `#[ignore]` | 0 / 0 | tech-debt-inventory |
| `#[test]` 数 | 49（work_item.rs = 0）| tech-debt-inventory |
| guard DSL 类型数（硬编码）| 3（`upstream_approved` / `has_sections:` / `checklist_complete[:section]`）| [api-contracts](../../docs/baseline/2026-06-09/api-contracts.md) |

---

## Bounded Contexts

> 按本切片范围内的模块归类。**不**预先映射到 popsicle-new 的 crate 划分（那是 product-debate / arch-debate 的活）。

| Context | 模块（`crates/popsicle-core/src/`）| Code LoC | 主要类型 | 备注 |
|---|---|---|---|---|
| **Document 实体** | `model/document.rs` | 191 | `Document`、`RawDocument` | 通用 frontmatter+body；`status` 是 String，**无独立状态机**（`document.rs:13`）|
| **Namespace 实体** | `model/namespace.rs` | 93 | `Namespace`、`NamespaceStatus{Active,Completed,Archived}` | 多 spec 容器（`namespace.rs:7,28`）|
| **WorkItem 实体** | `model/work_item.rs` | 125 | `WorkItem`、`WorkItemKind` | 统一 bug/story/testcase + JSON `fields` blob；**切片内需重命名为 `task_chunk_entity`**（migration/progress.md:12）|
| **Markdown 编辑** | `engine/markdown.rs` | 382 | 6 pub fn（纯函数）| section 抽取/upsert/summary/tags；无 crate 内依赖 |
| **Guard** | `engine/guard.rs` | 858 | `GuardResult`、`check_guard`、`count_checkboxes` | 状态转换守卫 DSL，3 类型硬编码（`guard.rs:65-96`）|
| **Context 装配** | `engine/context.rs` | 244 | `ContextInput/ContextPart/AssembledContext`、`assemble_input_context` | 按 `Relevance` 排序：Low→summary、Medium→sections、High→全文（`context.rs:51-77`）|
| **Context 分层** | `engine/context_layer.rs` | 236 | `trait ContextLayer` + 4 内建层 | **本范围唯一扩展点**（`context_layer.rs:22`）|
| **WorkItem 提取** | `engine/extractor.rs` | 365 | 3 pub fn（regex 驱动）| `extract_user_stories/test_cases/bugs` → `Vec<WorkItem>`（`extractor.rs:10,69,125`）|

> §Unclassified：本切片范围内无未归类模块。`storage::DocumentRow` / `registry::SkillRegistry` / `memory::Memory` / `model::{PipelineDef,PipelineRun,StageState}` 是**出向依赖**（属 slice-1/storage），不在本切片范围，但构成接口面（见 Risk Hotspots + dependency-graph）。

---

## Domain Glossary

> 在本切片代码、注释、doc-comment 中反复出现的术语。下游 skill 用它维护统一语言。

| 术语 | 首次出现 | 含义（源于代码/注释）| 置信度 |
|---|---|---|---|
| **Document** | `model/document.rs:9`（`struct Document`）| 通用 artifact：YAML frontmatter + Markdown body；所有 skill 产物共用此模型；落 `.popsicle/artifacts/<run-id>/`。`status` 为 String（"active"/"final"）| high |
| **RawDocument** | `model/document.rs:49` | frontmatter/body 分割中间表示 | high |
| **Revision** | `model/document.rs:85`（`new_revision`）| 文档修订：version+1、parent_doc_id 链接、status 重置 active | high |
| **Namespace** | `model/namespace.rs:7` | 多 spec 容器；status ∈ {Active,Completed,Archived} | high |
| **WorkItem** | `model/work_item.rs:13` | 统一 user_story/bug/test_case 实体；kind 判别 + JSON `fields` blob；本切片重命名 task_chunk_entity | high |
| **WorkItemKind** | `model/work_item.rs:42` | 判别枚举；`key_prefix()` 生成 key 前缀 | high |
| **Guard** | `engine/guard.rs:26`（`check_guard`）| 状态转换前置条件 DSL；`;` 分隔多 guard 全过才过；3 类型硬编码 | high |
| **GuardResult** | `engine/guard.rs:9` | `{passed, guard_name, message}` | high |
| **has_sections / checklist_complete / upstream_approved** | `engine/guard.rs:75,79,84` | 3 个 guard 类型字面量 | high |
| **is_template_placeholder** | `engine/markdown.rs:23` | 判断 section 是否仍是模板占位（guard 判空依据）| high |
| **Relevance** | `engine/context.rs:9`（`model::Relevance`）| 上游输入相关度 Low/Medium/High；决定装配时注入摘要还是全文 | high |
| **AssembledContext** | `engine/context.rs:34` | 装配结果 `{parts, full_text}` | high |
| **ContextLayer** | `engine/context_layer.rs:22`（`trait`）| 可插拔 prompt 分层；4 内建层；本范围唯一 `pub trait` 扩展点 | high |
| **assemble_layers** | `engine/context_layer.rs:34` | 组合 `Vec<Box<dyn ContextLayer>>` + base_prompt | high |
| **extract_user_stories / test_cases / bugs** | `engine/extractor.rs:10,69,125` | 从 doc.body 正则抽取 WorkItem | high |

---

## Risk Hotspots

> 本切片范围内同时具备**失败模式密度 / 体量 / 跨边界耦合**的模块。供 product-debate / arch-debate 复核，不预设裁决。

| 模块 | Code LoC | 生产 unwrap | 测试数 | 主要风险（事实层）| 证据 |
|---|---|---|---|---|---|
| `engine/extractor.rs` | 365 | **19** | 3 | 全部生产 unwrap 在此：常量正则编译 + 已命中子串二次 captures；`popsicle doc extract` 主路径 | [unsafe-risk-report](../../docs/baseline/2026-06-09/unsafe-risk-report.md) |
| `engine/guard.rs` | 858 | 0 | 14 | 范围内最大单模块；guard 类型 `if let` 链硬编码、未知类型返回 `InvalidSkillDef`（不 panic）；是 IDD 状态转换咽喉 | `engine/guard.rs:65-96` |
| `model/work_item.rs` | 125 | 0 | **0** | 唯一 0 单测模块；且本切片要重命名为 task_chunk_entity（双重变更面）| `model/work_item.rs`；`migration/progress.md:12` |
| `engine/context_layer.rs` | 236 | 0 | 3 | 唯一扩展 trait，跨依赖 memory + storage；迁移时需保持 trait 契约 | `engine/context_layer.rs:17,19,22` |

跨 product 接口面（artifact-system → slice-1/storage，事实层）：

- `guard` 依赖 `PipelineDef`/`PipelineRun`/`StageState`/`SkillRegistry`（`engine/guard.rs:3-4`）判定 `upstream_approved`。
- `context_layer::MemoriesLayer` 依赖 `memory::Memory`（`engine/context_layer.rs:17`）。
- `guard`/`context_layer` 依赖 `storage::DocumentRow`（`engine/guard.rs:5`、`engine/context_layer.rs:19`）。

每一行都引用了详细 artifact 或源文件。

---

## 迁移切片范围核对（vs PDR-001 / init）

> 本报告在 init stage 之后跑（顺序 init → facts）。init 已把本切片锁定为 `artifact-system`、范围为「6 引擎/模型模块 + namespace + task_chunk + doc/extract/summarize 命令族」。本节回答：事实数据**支不支持**该范围？

**支持**：

1. 6 个 init 列出的核心模块（document/markdown/guard/context/context_layer/extractor）逐一存在且边界清晰：model 层纯数据（document/namespace/work_item 无 crate 内依赖），engine 层各模块依赖收敛（dependency-graph 已列全部边）。
2. `namespace` 实体（`model/namespace.rs`，93 LoC）独立、低耦合，纳入切片成本低。
3. `work_item → task_chunk_entity` 重命名落在 `model/work_item.rs`（125 LoC，0 单测）+ 其唯一生产者 `engine/extractor.rs`——重命名面是这两个文件，范围可控。

**需下游裁决（不支持/需调整）**：

- **guard 的跨 product 依赖**：`check_guard` 需 pipeline/run/registry（slice-1 资产）。artifact-system crate 是否反向依赖 skill-runtime crate？还是 guard 的 pipeline 判定部分留在 skill-runtime？→ arch-debate 裁决边界面（`engine/guard.rs:103-169`）。
- **doc/extract/summarize 命令归属**：命令壳在 `popsicle-cli/src/commands/{doc.rs,extract.rs}`（cli-ux/slice-3 范围），核心逻辑在本 product。「壳 vs 核」边界 → product-debate 标注。
- **context_layer 依赖 memory**：`MemoriesLayer` 跨到 slice-1 的 memory。迁移时 trait 保留在 artifact-system 但 MemoriesLayer 实现归属待定。

考虑过的替代切片范围（与 init 决策一致性核对）：

- **把 guard 留在 skill-runtime**：因为 guard 强依赖 pipeline/run。否决理由（事实层）：guard 的 3 类型中 2 类（has_sections/checklist_complete）只依赖 `Document`+markdown（本 product），仅 upstream_approved 依赖 pipeline——拆分会割裂 guard DSL。范围决策留 arch-debate。
- **把 namespace 推迟到 cli-ux 切片**：namespace 与 spec/issue（slice-1 已迁）同属编排实体。否决理由：namespace 是 spec 的容器，事实层与 document 体系无耦合（`namespace.rs` 无 crate 内依赖），归 artifact-system 还是 skill-runtime 由 product-debate 裁决；init 暂归 artifact-system。

---

## 详细 Artifact

| Artifact | 文件 | 状态 |
|---|---|---|
| Dependency graph | [docs/baseline/2026-06-09/dependency-graph.md](../../docs/baseline/2026-06-09/dependency-graph.md) | ✅ |
| API contracts | [docs/baseline/2026-06-09/api-contracts.md](../../docs/baseline/2026-06-09/api-contracts.md) | ✅ |
| Unsafe / risk report | [docs/baseline/2026-06-09/unsafe-risk-report.md](../../docs/baseline/2026-06-09/unsafe-risk-report.md) | ✅ |
| Tech-debt inventory | [docs/baseline/2026-06-09/tech-debt-inventory.md](../../docs/baseline/2026-06-09/tech-debt-inventory.md) | ✅ |

> 4 份详细 artifact 落在仓库根 `docs/baseline/2026-06-09/`（IDD 设计：fact-extractor 输出去 baseline，下游 skill 引用相对路径）。本顶层 report 落在 `.popsicle/artifacts/`（popsicle 数据库管理）。

---

## 工具来源

| 工具 | 用途 | 状态 |
|---|---|---|
| `tokei`（`~/.cargo/bin/tokei`）| LoC（2,494 总 / 2,109 code）| ✅ |
| `ripgrep`（`rg`）| unsafe/unwrap/expect/panic/TODO/pub item/test 计数 | ✅ |
| `cargo metadata` / `cargo tree` | 依赖图 | ⚠️ 未跑 `[reduced fidelity]`（用 `use` 直读替代）|
| `cargo clippy` | dead_code/unused | ⚠️ 未跑 `[reduced fidelity]` |
| `git blame`（TODO 年龄）| — | ⚠️ skipped（TODO 命中=0）|
| `ast-grep` | 结构匹配 | ⚠️ 不可用（未安装）|

---

## 关键事实修正（init / 上游资料的反馈）

> facts 阶段发现的、与 init 资料不一致或需补充的事实，登记于此，由 living-doc-author / 下一轮 PDR 修正。

| 项 | 在哪里 | 正确事实 | 来源 |
|---|---|---|---|
| init 估「~2,276 行」（仅 6 模块 wc）| project-init-plan §Legacy Source | 含 namespace+work_item 共 8 模块，tokei 口径 2,494 行（2,109 code）| tokei |
| init 未列 guard 的跨 product 依赖 | project-init-plan §First Migration Slice | guard 依赖 pipeline/run/registry（slice-1 资产），构成切片边界面 | `engine/guard.rs:3-4` |
| `Document` 有无状态机？| 通识 | **无**独立状态机；`status` 是 String，由 stage 完成度决定 active/final（与 `.github/copilot-instructions.md` 一致）| `model/document.rs:13` |

---

## Extraction Checklist

- [x] 5 个 artifact 都产出且交叉链接（本 report + 4 个伴生，落 docs/baseline/2026-06-09/）
- [x] Summary 表中每个值都引用了详细 artifact 或源文件
- [x] Bounded contexts 已列（8 模块；§Unclassified 说明出向依赖不在范围）
- [x] Domain glossary 含 15 个术语（≥10）且都带置信度
- [x] Risk hotspots 表含 4 条（≥1 切片范围）且都带证据指针
- [x] 迁移切片范围核对至少考虑了 2 个替代（guard 留 skill-runtime / namespace 推迟）
- [x] 工具来源表列出了所有实际使用过的工具（含未用/未装的也标注）
- [x] 报告中没有句子含 "should" / "ought to" / "is bad" / "is good"（发表观点检查）
- [x] 报告中没有句子凭空发明代码中不存在的需求
- [x] 每个近似数字要么换成精确值，要么标 `(估)` / `[reduced fidelity]`
