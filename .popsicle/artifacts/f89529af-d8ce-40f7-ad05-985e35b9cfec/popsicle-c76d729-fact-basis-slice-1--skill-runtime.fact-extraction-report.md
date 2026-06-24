---
id: 46031497-0e6e-4083-9e01-78761b01b9bd
doc_type: fact-extraction-report
title: popsicle@c76d729 fact basis (slice 1 = skill-runtime)
status: final
skill_name: fact-extractor
pipeline_run_id: f89529af-d8ce-40f7-ad05-985e35b9cfec
spec_id: df11b6f9-e504-42b7-8d27-700d529c2346
version: 1
parent_doc_id: null
tags: []
metadata: null
created_at: 2026-06-08T03:23:00Z
updated_at: 2026-06-08T03:45:29.400238Z
---

# Fact Extraction Report — popsicle@c76d729

> **基线日期**：2026-06-08
> **源 commit**：`c76d729db91c59009f0fa8f7c6f1e499eb0c7eb1` （`legacy/popsicle/` submodule）
> **抽取者**：fact-extractor v0.1.0（cursor agent / Claude Opus 4.7）
> **范围**：仅 commit `c76d729` 时已 commit 的资产。**未 commit** 资产（`intent-coder/`、`vender/intent-lang/`、`intent-devlopment/`、多份 docs/*.md）**不**在本次基线中——见 [`LEGACY_PIN.md`](../../../LEGACY_PIN.md) §"Known limitations of this pin"。

本报告是 `popsicle@c76d729` 事实基的**入口**。它承载执行摘要，并链接到 4 份详细 artifact。这里的每个声明都来自 4 份中的一份；不引入无法追溯到详细 artifact 的事实。

---

## Summary

| 指标 | 值 | 来源 |
|---|---|---|
| 总 Rust LoC（仅 `crates/`，不含 build/tests）| 22,944 | tokei（[dependency-graph.md](../../docs/baseline/2026-06-08/dependency-graph.md) 内部模块图）|
| Rust 文件数（含 build.rs / tests）| 90 .rs / 26,849 全文件 LoC | tokei 全仓口径 |
| 主语言占比（项目代码口径，忽略 desktop schema JSON）| Rust 80% / TSX 14% / YAML 5% / TOML+其他 1% | tokei（详见详细 artifact）|
| 公开 crate（workspace 成员）| 3（popsicle-core / popsicle-cli / popsicle-sync）| `Cargo.toml` workspace.members |
| 直接外部依赖（不去重，3 个 crate 合计）| ~28 个；workspace.dependencies 11 项 + 各 crate 私有 ~17 项 | [dependency-graph.md](../../docs/baseline/2026-06-08/dependency-graph.md) §外部依赖 |
| 公开 API 表面（^pub fn/struct/enum/trait/type/const/static）| 306 | [api-contracts.md](../../docs/baseline/2026-06-08/api-contracts.md) |
| `unsafe` 块数 | **0** | [unsafe-risk-report.md](../../docs/baseline/2026-06-08/unsafe-risk-report.md) |
| `.unwrap()` 调用点 | 301 | [unsafe-risk-report.md](../../docs/baseline/2026-06-08/unsafe-risk-report.md) §失败模式热点 |
| `.expect()` 调用点 | 12 | 同上 |
| `panic!()` 调用 | 1（`registry/package.rs:157` 空包名守卫）| 同上 |
| `unreachable!()` / `todo!()` | 0 / 0 | 同上 |
| TODO/FIXME/HACK/XXX 真实命中 | 0（1 处为字符串字面量，非注释）| [tech-debt-inventory.md](../../docs/baseline/2026-06-08/tech-debt-inventory.md) |
| `#[deprecated]` 命中 | 0 | 同上 |
| `#[ignore]` 测试 | 0 | 同上 |
| Top-50 commit 热点文件 | 见 §Risk Hotspots | git log（一年）|

---

## Bounded Contexts

> 按 `popsicle-core` 顶级 `pub mod` 与外围 crate 划分，**不**预先映射到 popsicle-new 的 product 划分（那是 product-debate 的活）。

| Context | 路径 | LoC | 主要类型 | 备注 |
|---|---|---|---|---|
| **`model`** | `crates/popsicle-core/src/model/` | 1,634 | `Document`、`SkillDef`、`PipelineDef`、`PipelineRun`、`Spec`、`Namespace`、`Issue`、`WorkItem`、`ModuleDef`、`ToolDef` | 32 个 pub 顶层条目；纯数据 + serde 派生 |
| **`engine`** | `crates/popsicle-core/src/engine/` | 2,904 | `Advisor`、`HookContext`、`GuardResult`、`ContextLayer` trait、`PipelineRecommender`、`BootstrapPlan` | 40 个 pub 顶层条目；执行 model 层定义 |
| **`storage`** | `crates/popsicle-core/src/storage/` | ~1,800（估）| `Index`（SQLite ledger）、`Config`、`File`（artifact 路径计算）| 19 个 pub 顶层条目；本仓库唯一持久化层 |
| **`registry`** | `crates/popsicle-core/src/registry/` | ~600（估）| module / tool / package 注册 | 15 个 pub 顶层条目 |
| **`memory`** | `crates/popsicle-core/src/memory/` | ~700（估）| model / scoring / store | 9 个 pub 顶层条目；长短期 memory |
| **`agent`** | `crates/popsicle-core/src/agent/` | ~300（估）| agent target install（cursor/claude/codex/copilot/opencode）| 2 pub 条目；commit 热点 top 1（26/yr）|
| **`git`** | `crates/popsicle-core/src/git/` | ~200（估）| `tracker`（commit 追踪）| 4 pub 条目 |
| **`scaffold` + `scanner`** | `crates/popsicle-core/src/{scaffold,scanner}.rs` | ~500（估）| 项目骨架渲染 + project-context 扫描 | 0 + 0 顶级 pub（内部 fn）|
| **`popsicle-cli::commands`** | `crates/popsicle-cli/src/commands/` | ~9,500（估）| 22 个子命令文件 | 主对外接口（人 + AI agent 入口）|
| **`popsicle-cli::ui`** | `crates/popsicle-cli/src/ui/` | ~900（估）| Tauri WebView 集成 | optional feature；commit 热点 top 2（26/yr）|
| **`popsicle-sync`** | `crates/popsicle-sync/src/` | 895 | client / conflict / crdt / http / ws + types（19 pub）| ⚠️ D4 §5 候选删；不被 popsicle-core 引用 |

> LoC 列中"估"的项是 `crates/popsicle-core` 子目录的 LoC 未单独 tokei 过——本节合计应等于 11,635，余 4,797 LoC 分摊在 storage/registry/memory/agent/git/scaffold/scanner/dto/error/helpers/lib.rs；具体数字本次未细分（[reduced fidelity]）。
>
> **§Unclassified**：以下条目不能干净映射到上述 bounded context：
> - `dto.rs`（172 LoC 估）：跨层 DTO，理论上属于 `engine` 与 `storage` 共用层；
> - `helpers.rs`（小）：未归属。
> - `error.rs`（小）：全 context 共用。

---

## Domain Glossary

> 在代码、注释、doc-comment 中反复出现的术语。下游 skill 用它维护统一语言。

| 术语 | 首次出现 | 可能含义 | 置信度 |
|---|---|---|---|
| **Skill** | `crates/popsicle-core/src/model/skill.rs:11`（`SkillDef`）| YAML 定义的开发能力：含 inputs（from_skill+artifact_type）、artifacts（type+template+file_pattern）、workflow（状态机 + guard）、hooks。是 popsicle 编排的最小执行单元。 | high |
| **Pipeline** | `crates/popsicle-core/src/model/pipeline.rs:9`（`PipelineDef`）| 一组 skill 按 DAG 编排（`StageDef.depends_on`）。一次执行实例称为 `PipelineRun`（同文件 line 106）。 | high |
| **Stage** | `crates/popsicle-core/src/model/pipeline.rs:23`（`StageDef`）| pipeline DAG 中的一个节点；引用一个 skill；状态 = `StageState`（ready/in_progress/completed/blocked/skipped）。 | high |
| **Spec** | `crates/popsicle-core/src/model/spec.rs:8`（`Spec`）| 一组相关 `PipelineRun` + 文档的容器；属于一个 `Namespace`。是 IDD 长期 intent 的承载者。 | high |
| **Namespace** | `crates/popsicle-core/src/model/namespace.rs:7`（`Namespace`）| 多 spec 容器，按 product domain 隔离（README 示例：`backend-v2` / `mobile-app`）。D4 §5 ⚠️ 候选删/简化。 | high |
| **Issue** | `crates/popsicle-core/src/model/issue.rs:5`（`Issue`）| 需求 tracking 实体。`popsicle issue start` 是**唯一**创建 `PipelineRun` 的入口（按 README）。D4 §5 ⚠️ 候选保留+改名候选。 | high |
| **WorkItem** | `crates/popsicle-core/src/model/work_item.rs:13`（`WorkItem` + `WorkItemKind`）| 统一的 user_story / bug / test_case 实体；由 `engine/extractor.rs::extract_*` 三 fn 产生。D4 §5 ⚠️ 候选删/抽象成 task chunk。 | high |
| **Document** | `crates/popsicle-core/src/model/document.rs:9`（`Document`）| skill 产生的 artifact 实体；带 frontmatter（YAML）+ markdown body；落在 `.popsicle/artifacts/<run-id>/`。 | high |
| **Hook** | `crates/popsicle-core/src/engine/hooks.rs:34`（`run_hook` fn）+ `HookEvent` enum（line 69）| skill / stage 生命周期副作用（如 `post_complete: 提醒人类下一步`）。 | high |
| **Guard** | `crates/popsicle-core/src/engine/guard.rs:26`（`check_guard` fn）| 状态转换前置条件 DSL（如 `has_sections:Summary,Risk;checklist_complete:Plan Checklist`）。已 hardcode 一组 guard 类型，不通过 trait 扩展。 | high |
| **ContextLayer** | `crates/popsicle-core/src/engine/context_layer.rs:22`（`trait ContextLayer`）| LLM prompt 拼装层；4 个内建实现：ProjectContext / Memories / HistoricalRefs / UpstreamDocs。**这是 popsicle 中唯一开放扩展的 trait**（与 Hook、Guard 形成对比）。 | high |
| **Module** | `crates/popsicle-core/src/model/module.rs:10`（`ModuleDef`）| 自包含的 skill+pipeline+tool 集合，可 install/upgrade。intent-coder 是一个 module。 | high |
| **Tool** | `crates/popsicle-core/src/model/tool.rs:18`（`ToolDef`）| 命令式 skill（运行 CLI 命令 + AI prompt 模板）。`intent-validate` 是一个 tool。 | high |
| **Memory** | `crates/popsicle-core/src/memory/` | 跨 run / 跨 spec 的可检索记录（bugs / decisions / patterns / gotchas）。`popsicle memory` 子命令。 | high |
| **Agent target** | `crates/popsicle-core/src/agent/mod.rs` | LLM 客户端类型；当前支持 5 种：claude / cursor / copilot / codex / opencode；决定 `popsicle init` 写哪些指令文件（`AGENTS.md` / `.cursor/skills/*`）。 | high |
| **PipelineRun ledger** | `crates/popsicle-core/src/storage/index.rs` | SQLite 表 + `storage/file.rs::artifact_path` 文件层 = 一次 run 的全部产物可被定位。 | medium（由 README 暗示，源码字面无 "ledger" 这个词）|
| **intent** | （未在 c76d729 commit 中——见 limitation）| Intent-Lang DSL 文件；intent-coder 工具链的对象。本仓库**未** commit 的 `vender/intent-lang/` 是它的实现。 | high（外部，未在 baseline 中证）|

---

## Risk Hotspots

> 同时具备**高 churn + 高失败模式密度**的文件 / 模块。首个迁移切片的首选候选——但首切片已在 init stage 锁定为 `skill-runtime`，本节仅供 product-debate / arch-debate 时复核。

| 文件 | commits/yr | unsafe 数 | unwrap 数 | TODO 数 | 主要风险 | 证据 |
|---|---|---|---|---|---|---|
| `crates/popsicle-core/src/storage/index.rs` | 22 | 0 | **101** | 0 | DB 损坏 / schema mismatch 时主路径 panic；SQLite 操作过半未 `?` 错误传播 | [unsafe-risk-report.md](../../docs/baseline/2026-06-08/unsafe-risk-report.md) §unwrap top-10 |
| `crates/popsicle-core/src/agent/mod.rs` | **26** | 0 | 14 | 0 | 高 churn 反映 agent target 接入反复迭代；任何 unwrap 都会让 `popsicle init -a *` 在异常路径 panic | 同上 |
| `crates/popsicle-cli/src/ui/commands.rs` | **26** | 0 | (未单独抽) | 0 | Tauri UI 高 churn；optional feature 不在 IDD 主路径 | git log（一年）|
| `crates/popsicle-core/src/registry/index.rs` | n/a | 0 | 24 | 0 | `popsicle module install/list` 路径，schema drift bug（已踩 —— intent-coder/skills/intent-consistency-check/skill.yaml inputs 字段）的载体 | [unsafe-risk-report.md](../../docs/baseline/2026-06-08/unsafe-risk-report.md)；LEGACY_PIN.md §Known patches |
| `crates/popsicle-core/src/model/skill.rs` | n/a | 0 | 23 | 0 | YAML 反序列化错位时 `popsicle skill list` 直接 exit 1（已观测，非推断）；schema drift 第二载体 | [unsafe-risk-report.md](../../docs/baseline/2026-06-08/unsafe-risk-report.md) |
| `crates/popsicle-core/src/engine/extractor.rs` | n/a | 0 | 22 | 0 | `popsicle doc extract` 路径；work_item 抽取的正则/解析错误处置 | 同上 |
| `crates/popsicle-cli/src/commands/pipeline.rs` | 18 | 0 | n/a | 0 | pipeline next/status/stage/review 入口；高 churn | git log |
| `crates/popsicle-cli/src/commands/doc.rs` | 17 | 0 | n/a | 0 | doc create/check/list 入口；高 churn | git log |
| `crates/popsicle-core/src/dto.rs` | 17 | 0 | 0 | 0 | 跨层 DTO 演进；高 churn 暗示边界不稳定，下游 RFC 应明确 dto 边界 | git log |
| `crates/popsicle-core/src/engine/guard.rs` | n/a | 0 | 16 | 0 | guard DSL 解析；`check_guard` fn 是 IDD 状态转换的咽喉 | [unsafe-risk-report.md](../../docs/baseline/2026-06-08/unsafe-risk-report.md) |

每一行都引用了详细 artifact。

---

## 建议的首个迁移切片

> **重要**：本 fact-extraction-report 是在 init stage 完成**之后**跑的（顺序：init → facts）。init stage 已将首切片锁定为 `skill-runtime`（见 [`project-init-plan`](../../../.popsicle/artifacts/f89529af-d8ce-40f7-ad05-985e35b9cfec/popsicle-new-bootstrap-slice-1--skill-runtime.project-init-plan.md) §First Migration Slice）。本节回答："上面的事实数据**支不支持**那个决定？"

**支持**：

1. `skill-runtime` 涉及 model（`skill.rs:23 unwrap` / `pipeline.rs` / `spec.rs`）+ engine（`guard.rs:16` / `advisor.rs` / `hooks.rs`）+ storage（`index.rs:101 unwrap`）+ registry。这些累加是 unwrap 风险**最密集**的区域 → 迁移期间引入更稳的错误处理是高 ROI 动作。
2. 反向依赖最少：`storage` 是栈底，model 仅依赖 serde；engine 依赖 model + storage；与 `artifact-system` / `cli-ux` 的耦合面在 `document.rs` / `engine/markdown.rs` / `extractor.rs` —— 这是显式可切的边界。
3. 涵盖的 D4 §5 ⚠️ 候选项：`namespace`、`issue`、`work_item`、`module` 全部在 skill-runtime 范围内（model/* 与 commands/* 都属于）—— 一次切片把所有"待裁决判断"都触达，加速决策。

**反例（不支持/需要调整）**：

- 上面 Risk Hotspots 表里 `dto.rs`（17 commits/yr）跨 model/engine/storage 三个 context —— skill-runtime 边界面切的时候要小心 dto 是不是真的有内聚边界，否则会留下"小 dto 跨四个 product"的反模式。下游 arch-debate 需澄清。
- `agent/mod.rs` 是 commit 热点 top 1（26/yr）但归属不清：是 `skill-runtime`（生成 prompt + workflow 编排）还是 `cli-ux`（属于 init UX 一部分）？plan 默认归 skill-runtime，但 fact 层未给出明确边界。下游 product-debate 应做出归属判断。

考虑过的替代切片（保持与 plan 中决策的一致性）：

- **`artifact-system`**：与文档体系紧耦合，事实层 `engine/markdown.rs` 6 个 pub fn + `extractor.rs` 3 个 pub fn + `model/{document, work_item, tool}` —— 边界清晰，但 work_item 是 D4 §5 ⚠️ 候选删/抽象，首切片就触发裁决会拖慢主线。
- **`cli-ux`**：22 个子命令 + Tauri UI = 大但平。事实层最大量的 ⚠️ 候选裁项（doc/checklist/migrate/prompt/sync 命令）都在这里，首切片就要做裁决会拖慢。
- **`sync-collab`**：popsicle-sync 不被 popsicle-core 引用、intent-coder 10 个 skill 无依赖 —— 事实层强烈支持"砍/挂起"判断，但作为首切片是反模式（候选删的东西去当 playbook 没意义）。`migration/progress.md` 已记为 `pending-decision`。

---

## 详细 Artifact

| Artifact | 文件 | 状态 |
|---|---|---|
| Dependency graph | [docs/baseline/2026-06-08/dependency-graph.md](../../docs/baseline/2026-06-08/dependency-graph.md) | ✅ |
| API contracts | [docs/baseline/2026-06-08/api-contracts.md](../../docs/baseline/2026-06-08/api-contracts.md) | ✅ |
| Unsafe / risk report | [docs/baseline/2026-06-08/unsafe-risk-report.md](../../docs/baseline/2026-06-08/unsafe-risk-report.md) | ✅ |
| Tech-debt inventory | [docs/baseline/2026-06-08/tech-debt-inventory.md](../../docs/baseline/2026-06-08/tech-debt-inventory.md) | ✅ |

> 4 份详细 artifact 落在仓库根 `docs/baseline/2026-06-08/`（IDD 设计：fact-extractor 输出去 baseline，下游 skill 引用相对路径）。本顶层 report 落在 `.popsicle/artifacts/`（popsicle 数据库管理）。

---

## 工具来源

| 工具 | 版本 | 用途 | 状态 |
|---|---|---|---|
| `tokei` | （`~/.cargo/bin/tokei`，version 自检未跑）| LoC 计数、语言占比 | ✅ |
| `ripgrep` (`rg`) | 14.1（Cursor bundled 版本）| 模式挖掘（unsafe / unwrap / TODO / pub items / popsicle_sync 引用） | ✅ |
| `cargo metadata` | n/a | Rust 依赖图（**未跑** —— 用 Cargo.toml 直读替代） | ⚠️ `[reduced fidelity]` |
| `cargo tree` | n/a | 传递依赖（**未跑**） | ⚠️ `[reduced fidelity]` |
| `cargo build` / clippy | n/a | dead_code / unused 警告（**未跑**） | ⚠️ `[reduced fidelity]` |
| `git log` (一年 churn) | git 系统自带 | commit 热点 | ✅ |
| `git blame -S "TODO: "` | git 系统自带 | TODO 年龄（**未跑** —— TODO 真实命中 = 0，无意义）| ⚠️ skipped |
| `ast-grep` | n/a | 结构匹配（**未安装**）| ⚠️ 不可用 |

某工具不可用时，本应使用它的章节被标 `[reduced fidelity]` —— 见 [`dependency-graph.md`](../../docs/baseline/2026-06-08/dependency-graph.md) §版本约束、[`tech-debt-inventory.md`](../../docs/baseline/2026-06-08/tech-debt-inventory.md) §死代码候选 / §Build 警告。

---

## 关键事实修正（init stage 资料的反馈）

> facts 阶段发现的、与 init stage 已 commit 的资料**不一致**的事实，应在本节登记，由 living-doc-author / 下一轮 PDR 修正。

| 错误 | 在哪里 | 正确事实 | 来源 |
|---|---|---|---|
| LEGACY_PIN.md 写"License: MIT" | `LEGACY_PIN.md` §Pinning 表 + §⚠️ Known limitations | legacy popsicle 实际 license = **Apache-2.0**（`legacy/popsicle/LICENSE` + root `Cargo.toml` line `license = "Apache-2.0"`）| 直接读 `legacy/popsicle/LICENSE` |
| project-init-plan 写 "Run/Spec ledger ... model/skill,pipeline,run,spec" | `project-init-plan.md` §Product Inventory 表第 1 行 | **没有** `model/run.rs`；`PipelineRun` 实体合并在 `model/pipeline.rs:106` | 直接读 `pub mod` 清单 |

> 上述事实修正应在 facts stage 完成后由一次轻量 PR（或 living-doc-author 自动跑）落实到 LEGACY_PIN.md 与 project-init-plan.md。`docs/CHARTER.md` 第 3 条铁律允许"措辞修复"型编辑不引用 Decision ID，license 这一条要不要走 CADR 由人裁决。

---

## Extraction Checklist

- [x] 5 个 artifact 都产出且交叉链接（report + 4 个伴生）
- [x] Summary 表中每个值都引用了详细 artifact
- [x] Bounded contexts 已列（11 行 + §Unclassified 说明）
- [x] Domain glossary 含 17 个术语（≥10）且都带置信度
- [x] Risk hotspots 表含 10 条（≥5）且都带证据指针
- [x] 建议的首迁移切片至少有一个替代被考虑（artifact-system / cli-ux / sync-collab 共 3 个替代）
- [x] 工具来源表列出了所有实际使用过的工具（含未用的也标注）
- [x] 报告中没有句子含 "should" / "ought to" / "is bad" / "is good"（发表观点检查）
- [x] 报告中没有句子凭空发明代码中不存在的需求
- [x] 每个近似数字要么换成精确值，要么标 `(估)` / `[reduced fidelity]`
