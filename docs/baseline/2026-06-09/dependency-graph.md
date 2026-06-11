# Dependency Graph — artifact-system scope @ popsicle c76d729

> **Scope**: 仅 slice-2 (artifact-system) 的 8 个 legacy 模块。源 commit `c76d729`，`legacy/popsicle/` submodule。
> **工具**: `ripgrep`（`use` 抽取）+ 直读源文件。`cargo metadata` / `cargo tree` 未跑（`[reduced fidelity]`，与 slice-1 一致）。
> **记录员声明**: 本文件只记事实并 cite file:line，不发表观点。

## 范围内模块（8）

| 模块 | 路径（`crates/popsicle-core/src/`）| Code LoC（tokei）|
|---|---|---|
| document | `model/document.rs` | 191（全文件）|
| namespace | `model/namespace.rs` | 93 |
| work_item | `model/work_item.rs` | 125 |
| markdown | `engine/markdown.rs` | 382 |
| guard | `engine/guard.rs` | 858 |
| context | `engine/context.rs` | 244 |
| context_layer | `engine/context_layer.rs` | 236 |
| extractor | `engine/extractor.rs` | 365 |

> 8 文件合计 tokei：Lines 2494 / Code 2109 / Comments 118 / Blanks 267。

## 内部依赖边（`use crate::...`，已 cite）

| 源模块 | 依赖 | 证据 |
|---|---|---|
| guard | `engine::markdown` | `engine/guard.rs:1` |
| guard | `error::{PopsicleError, Result}` | `engine/guard.rs:2` |
| guard | `model::{Document, PipelineDef, PipelineRun, StageState}` | `engine/guard.rs:3` |
| guard | `registry::SkillRegistry` | `engine/guard.rs:4` |
| guard | `storage::DocumentRow` | `engine/guard.rs:5` |
| context | `engine::markdown` | `engine/context.rs:8` |
| context | `model::Relevance` | `engine/context.rs:9` |
| context_layer | `engine::context::AssembledContext` | `engine/context_layer.rs:16` |
| context_layer | `memory::Memory` | `engine/context_layer.rs:17` |
| context_layer | `model::Relevance` | `engine/context_layer.rs:18` |
| context_layer | `storage::DocumentRow` | `engine/context_layer.rs:19` |
| extractor | `model::document::Document` | `engine/extractor.rs:4` |
| extractor | `model::work_item::{WorkItem, WorkItemKind}` | `engine/extractor.rs:5` |
| document | （无 `use crate::`；仅 std/chrono/serde/uuid）| `model/document.rs:1-4` |
| namespace | （无 `use crate::`）| — |
| work_item | （无 `use crate::`）| — |
| markdown | （无 `use crate::`；纯字符串处理）| — |

## 依赖方向（事实层，未评判）

```
document  ──┐ (纯数据，无 crate 内依赖)
namespace ──┤
work_item ──┘
markdown   ── (纯字符串工具，无 crate 内依赖)
   ▲   ▲
   │   └──────── context ── 依赖 markdown + model::Relevance
   │                 ▲
   │                 └────── context_layer ── 依赖 context::AssembledContext + memory::Memory + storage::DocumentRow
   └──────────── guard ── 依赖 markdown + model::{Document,PipelineDef,PipelineRun,StageState} + registry::SkillRegistry + storage::DocumentRow
extractor ── 依赖 model::document::Document + model::work_item::{WorkItem,WorkItemKind}
```

## 跨 product 出向依赖（artifact-system → 外部 context，事实层标注）

| 被依赖 | 属于（推断来源，待 product-debate 裁决）| 出现处 |
|---|---|---|
| `registry::SkillRegistry` | skill-runtime（registry 是 slice-1 范围）| `engine/guard.rs:4` |
| `model::{PipelineDef, PipelineRun, StageState}` | skill-runtime（pipeline/run 是 slice-1 范围）| `engine/guard.rs:3` |
| `storage::DocumentRow` | storage（未分配 product；栈底持久化）| `engine/guard.rs:5`、`engine/context_layer.rs:19` |
| `memory::Memory` | skill-runtime（memory 是 slice-1 范围）| `engine/context_layer.rs:17` |

> 上述出向依赖是 artifact-system 与 slice-1（skill-runtime）的真实接口面：`guard` 需要 pipeline/run/registry
> 来判定 `upstream_approved`；`context_layer` 需要 memory + storage 来拼装 prompt。边界归属由 product-debate / arch-debate 裁决。

## Checklist

- [x] 范围内每个模块都列了路径 + LoC
- [x] 每条内部依赖边都 cite file:line
- [x] 跨 product 出向依赖已标注（未评判归属，留给 product-debate）
- [x] 未跑的工具已标 `[reduced fidelity]`
- [x] 无 "should/ought to/is bad" 等观点句
