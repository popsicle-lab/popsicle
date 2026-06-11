# Tech-Debt Inventory — artifact-system scope @ popsicle c76d729

> **Scope**: slice-2 (artifact-system) 8 模块。源 commit `c76d729`。
> **方法**: `rg` 注释扫描 + 直读。每条 cite file:line。
> **记录员声明**: 只记可观察标记，不评判严重度。

## 债务标记计数（8 模块合计）

| 标记 | 命中 | 备注 |
|---|---|---|
| `// TODO` / `FIXME` / `HACK` / `XXX`（注释）| **0** | 全范围无债务注释标记 |
| `#[deprecated]` | 0 | |
| `#[ignore]` 测试 | 0 | |
| `#[allow(...)]` | 见下表 | |

## 测试覆盖（事实层，`#[test]` 计数）

| 模块 | `#[test]` 数 | 备注 |
|---|---|---|
| `engine/markdown.rs` | 17 | 纯函数，测试最密 |
| `engine/guard.rs` | 14 | guard DSL 各分支 |
| `engine/context.rs` | 7 | Relevance 排序/装配 |
| `model/document.rs` | 3 | 序列化往返 |
| `engine/context_layer.rs` | 3 | 含 StaticLayer 测试桩（`context_layer.rs:177`）|
| `engine/extractor.rs` | 3 | story/testcase/bug 各 1 |
| `model/namespace.rs` | 2 | status parse |
| `model/work_item.rs` | **0** | 无单测 |
| **合计** | **49** | |

## 命名/迁移债务（事实层，来自 migration/progress.md + 源码）

| 项 | 事实 | 证据 |
|---|---|---|
| `work_item` → `task_chunk_entity` 重命名 | migration/progress.md row 2 标注 artifact-system 切片需把旧 `work_item` 重命名为 `task_chunk_entity` | `migration/progress.md:12` |
| `WorkItem` 无单测 | `model/work_item.rs` 内 0 个 `#[test]` | `model/work_item.rs` |
| guard 类型硬编码 | `check_single_guard` 用 `if let` 链枚举 3 种 guard，无 trait 扩展点；未知类型返回 `InvalidSkillDef` | `engine/guard.rs:65-96` |

## 跨 product 耦合债务（事实层标注，待 product-debate 裁决）

| 耦合 | 事实 | 证据 |
|---|---|---|
| `guard` → skill-runtime | guard 判定 `upstream_approved` 需 `PipelineDef`/`PipelineRun`/`StageState`/`SkillRegistry` | `engine/guard.rs:3-4`，`:103-169` |
| `context_layer` → skill-runtime | `MemoriesLayer` 依赖 `memory::Memory` | `engine/context_layer.rs:17,76-80` |
| `guard`/`context_layer` → storage | 依赖 `storage::DocumentRow` | `engine/guard.rs:5`、`engine/context_layer.rs:19` |
| doc/extract/summarize 命令壳 | 命令入口在 `popsicle-cli/src/commands/{doc.rs,extract.rs}`（cli-ux 范围），核心逻辑在本 product（markdown/extractor/guard）| `crates/popsicle-cli/src/commands/doc.rs`、`extract.rs` |

## 工具来源

| 工具 | 用途 | 状态 |
|---|---|---|
| `tokei` (`~/.cargo/bin/tokei`) | LoC（2494 总 / 2109 code）| ✅ |
| `ripgrep` | unwrap/unsafe/TODO/pub item/test 计数 | ✅ |
| `cargo metadata` / `cargo tree` | 依赖图 | ⚠️ 未跑 `[reduced fidelity]` |
| `cargo clippy` | dead_code/unused | ⚠️ 未跑 `[reduced fidelity]` |
| `git blame`（TODO 年龄）| — | ⚠️ skipped（TODO 命中=0，无意义）|

## Checklist

- [x] TODO/FIXME/deprecated/ignore 计数完整（均 0，已记录）
- [x] 测试覆盖按模块列出（work_item 0 测试已标注）
- [x] 命名/迁移债务（work_item→task_chunk）已 cite migration/progress.md
- [x] 跨 product 耦合已标注来源，未评判
- [x] 未跑工具已标 `[reduced fidelity]` / skipped
- [x] 无观点句
