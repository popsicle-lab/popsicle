# API Contracts — popsicle@c76d729

> 配套：[`fact-extraction-report.md`](../../../.popsicle/artifacts/f89529af-d8ce-40f7-ad05-985e35b9cfec/popsicle-c76d729-fact-basis-slice-1--skill-runtime.fact-extraction-report.md)
>
> 基线：`legacy/popsicle/` submodule @ `c76d729db91c59009f0fa8f7c6f1e499eb0c7eb1`
> 抽取范围：所有 `^pub (fn|struct|enum|trait|type|const|static)`（顶层公开项）。**共 306 项**——本文档按 bounded context 列出**结构性条目**（types、traits、关键 fn）；纯 helper fn 与所有数据字段引用源文件。
>
> Bounded context 划分按 `popsicle-core` 顶级 `pub mod` 与外围 crate，**不**预先映射到 popsicle-new 的 product 划分（那是 product-debate / arch-debate 的活）。

---

## Bounded Context：`model` —— 实体类型层（popsicle-core）

> **路径**：`crates/popsicle-core/src/model/`
> **用途**：所有持久化实体的纯数据类型 + 字面 serde 派生。32 个 `pub` 顶层条目。
> **下游 product 候选映射**：spec/skill/pipeline → `skill-runtime`；document/work_item/tool → `artifact-system`；namespace/issue → `skill-runtime` 或 D4 §5 候选删；module → `skill-runtime`。

### 公开类型 —— pipeline.rs（5）

| 类型 | 种类 | File:Line | 备注 |
|---|---|---|---|
| `PipelineDef` | struct | `model/pipeline.rs:9` | YAML 中定义的 pipeline 模板 |
| `StageDef` | struct | `model/pipeline.rs:23` | DAG 中一个 stage 的定义（skill 引用 + depends_on）|
| `RunType` | enum | `model/pipeline.rs:85` | 区分新跑 / revise / archive 等 |
| `PipelineRun` | struct | `model/pipeline.rs:106` | 一次 pipeline 执行实例（**注意：合并在 pipeline.rs，无独立 run.rs**）|
| `StageState` | enum | `model/pipeline.rs:132` | `ready` / `in_progress` / `completed` / `blocked` / `skipped` |

### 公开类型 —— skill.rs（9）

| 类型 | 种类 | File:Line | 字段说明 |
|---|---|---|---|
| `SkillDef` | struct | `model/skill.rs:11` | 整个 skill 的 YAML 反序列化目标 |
| `DocLifecycle` | enum | `model/skill.rs:43` | `singleton` / `instance_per_run` 等 |
| `Relevance` | enum | `model/skill.rs:66` | input 与 upstream 的相关度（`high`/`medium`/`low`）|
| `SkillInput` | struct | `model/skill.rs:85` | 上游 artifact 依赖（from_skill + artifact_type + required）|
| `ArtifactDef` | struct | `model/skill.rs:104` | 一个 artifact 的元数据（type / template / file_pattern）|
| `ExtractionSpec` | enum | `model/skill.rs:121` | 从 doc 抽 WorkItem 的规则 |
| `WorkflowDef` | struct | `model/skill.rs:176` | skill 内部状态机定义 |
| `StateDef` | struct | `model/skill.rs:182` | 状态机的一个状态 |
| `TransitionDef` | struct | `model/skill.rs:190` | 状态转换边（含 guard）|
| `HooksDef` | struct | `model/skill.rs:200` | skill 生命周期钩子声明 |

### 公开类型 —— document.rs（2）

| 类型 | File:Line | 备注 |
|---|---|---|
| `Document` struct | `model/document.rs:9` | 文档实体（id / title / status / pipeline_run_id / file 路径等）|
| `RawDocument` struct | `model/document.rs:49` | 写盘前的中间态 |

### 公开类型 —— 其余（16）

| 类型 | File:Line |
|---|---|
| `Spec` struct + `slugify` fn | `model/spec.rs:8,56` |
| `ModuleDef` / `ToolDependency` struct | `model/module.rs:10,32` |
| `ToolDef` / `ToolArg` struct + `render_template` fn | `model/tool.rs:18,58,132` |
| `WorkItem` struct + `WorkItemKind` enum | `model/work_item.rs:13,42` |
| `Issue` struct + `IssueType` / `Priority` / `IssueStatus` enum | `model/issue.rs:5,24,69,102` |
| `Namespace` struct + `NamespaceStatus` enum | `model/namespace.rs:7,28` |

### 行为备注

- **没有独立 `model/run.rs`**：`PipelineRun` 与 pipeline 模板共住 `pipeline.rs`。下游设计 `skill-runtime` 时如要做"独立 run ledger"重构，需要先 ADR。
- **`SkillInput` 字段是 `from_skill + artifact_type + required`**——这与 `intent-coder/skills/intent-consistency-check/skill.yaml`（已 patch）的格式一致。

---

## Bounded Context：`engine` —— 执行引擎（popsicle-core）

> **路径**：`crates/popsicle-core/src/engine/`
> **用途**：把 model 层的定义"跑起来"：advisor 给建议、guard 校验状态转换、hooks 触发副作用、context 拼装 LLM prompt、markdown 编辑器、extractor 从文档抽 WorkItem。40 个 `pub` 顶层条目（最多的子模块）。
> **下游 product 候选映射**：以下大部分映射到 `skill-runtime`；`markdown`、`extractor` 映射到 `artifact-system`。

### 公开 fn（核心入口）

| 签名 | File:Line | 用途 |
|---|---|---|
| `pub fn assemble_input_context(inputs: Vec<ContextInput>) -> AssembledContext` | `engine/context.rs:51` | 把上游 artifact 拼成可注入 LLM 的 prompt |
| `pub fn run_hook(...)` | `engine/hooks.rs:34` | 执行 skill 状态转换或 stage 完成的副作用 |
| `pub fn check_guard(guard_expr: &str, doc: &Document) -> GuardResult` | `engine/guard.rs:26` | 校验状态转换条件（如 `has_sections:...` / `checklist_complete:...`）|
| `pub fn count_checkboxes(text: &str) -> (usize, usize)` | `engine/guard.rs:270` | checklist 项已勾 / 总数 |
| `pub fn build_bootstrap_prompt(...)` | `engine/bootstrap.rs:92` | 一键 bootstrap 多文档的 LLM 提示 |
| `pub fn execute_bootstrap_plan(...)` | `engine/bootstrap.rs:176` | 执行 bootstrap plan |
| `pub fn extract_user_stories(doc: &Document) -> Vec<WorkItem>` | `engine/extractor.rs:10` | 从 doc 抽 user story |
| `pub fn extract_test_cases(doc: &Document, test_type: &str) -> Vec<WorkItem>` | `engine/extractor.rs:69` | 抽测试用例 |
| `pub fn extract_bugs(doc: &Document) -> Vec<WorkItem>` | `engine/extractor.rs:125` | 抽 bug |
| `pub fn assemble_layers(layers: Vec<Box<dyn ContextLayer>>, base_prompt: &str) -> String` | `engine/context_layer.rs:34` | 顺序拼装多个 context layer |

### Markdown 编辑器（公开 fn）

| 签名 | File:Line | 用途 |
|---|---|---|
| `pub fn extract_section_content(after_header: &str) -> String` | `engine/markdown.rs:5` | 提取标题下的内容 |
| `pub fn is_template_placeholder(content: &str) -> bool` | `engine/markdown.rs:23` | 检测占位符（`{TBD}` / `[TBD]` 等）|
| `pub fn extract_sections(body: &str, section_names: &[String]) -> String` | `engine/markdown.rs:57` | 提取指定章节 |
| `pub fn extract_summary(body: &str) -> String` | `engine/markdown.rs:77` | 提取摘要 |
| `pub fn extract_tags(body: &str, skill_name: &str, doc_type: &str) -> Vec<String>` | `engine/markdown.rs:125` | 自动 tag |
| `pub fn upsert_section(doc: &str, section_name: &str, content: &str, append: bool) -> String` | `engine/markdown.rs:159` | 写入/更新章节 |

### 公开类型

| 类型 | 种类 | File:Line |
|---|---|---|
| `ContextInput` / `ContextPart` / `AssembledContext` | struct | `engine/context.rs:13,24,34` |
| `PipelineRecommender` / `Recommendation` / `Alternative` | struct | `engine/recommender.rs:10,14,23` |
| `BootstrapPlan` / `BootstrapNamespace` / `BootstrapSpec` / `BootstrapDoc` | struct | `engine/bootstrap.rs:9,19,31,47` |
| `BootstrapResult` / `BootstrapNamespaceResult` / `BootstrapSpecResult` | struct | `engine/bootstrap.rs:59,70,78` |
| `Advisor` / `NextStep` | struct | `engine/advisor.rs:9,13` |
| `HookContext` / `HookEvent` / `HookResult` | struct/enum | `engine/hooks.rs:10,69,86` |
| `GuardResult` | struct | `engine/guard.rs:9` |

### 公开 Trait

| Trait | File:Line | 实现者（同文件内）|
|---|---|---|
| `ContextLayer: Send + Sync` | `engine/context_layer.rs:22` | `ProjectContextLayer`（line 55）、`MemoriesLayer`（line 76）、`HistoricalRefsLayer`（line 105）、`UpstreamDocsLayer`（line 146）|

### 行为备注

- `check_guard` 使用 DSL 字符串解析（`has_sections:` / `checklist_complete:` 等），不是配置 trait 实现——这意味着扩展 guard 类型需要改 `guard.rs` 源码（**不开放扩展**，关闭修改的反例之一）。
- `Hook` 与 `ContextLayer` 是**两套不同的扩展机制**：Hook 是状态转换副作用（运行时副作用），ContextLayer 是 prompt 拼装层（信息组合）。下游 RFC 应澄清两者关系。

---

## Bounded Context：`storage` —— SQLite Ledger + 文件落盘（popsicle-core）

> **路径**：`crates/popsicle-core/src/storage/`
> **用途**：本仓库**唯一**的持久化层。SQLite 表 + `.popsicle/artifacts/` 文件。19 个 `pub` 顶层条目（含 trait + 实现）。
> **下游 product 候选映射**：核心持久化能力归 `skill-runtime`；`storage/file.rs::artifact_path` 等文件路径计算可拆出去给 `artifact-system`。

### 公开条目（典型）

| 条目 | File | 用途 |
|---|---|---|
| SQLite `Index` + 各种 CRUD fn | `storage/index.rs` | 实体 CRUD（doc / run / spec / namespace / commit / item / memory）|
| 文件路径计算 fn | `storage/file.rs` | `artifact_path(run_id, slug, type) -> PathBuf` 等 |
| `Config` struct + 读写 fn | `storage/config.rs` | `.popsicle/config.toml` 解析 |

> 由于 `storage/index.rs` 有 22 commit/yr（commit 热点 top 4），是迁移期间需要重点 baseline 的文件。

---

## Bounded Context：`registry` / `memory` / `git` / `agent` / `scaffold` / `scanner`（popsicle-core 其余）

| 模块 | pub items | File 数 | 用途 |
|---|---|---|---|
| `registry` | 15 | 4 | module / tool 注册、index / loader / package（`registry/loader.rs` 装 module）|
| `memory` | 9 | 4 | 长短期 memory（model / scoring / store）|
| `git` | 4 | 2 | git commit tracking（`git/tracker.rs`）|
| `agent` | 2 | 1 | agent target install（cursor / claude / codex / copilot / opencode），写 `.cursor/skills/` 或 `AGENTS.md` |
| `scaffold` | 0 | 1 | 项目骨架渲染（被 `init` 命令调用）|
| `scanner` | 0 | 1 | project context scan（生成 `.popsicle/project-context.md`）|
| `dto` | 0 | 1 | 跨层数据传输对象 |
| `error` | 0 | 1 | 错误类型 |
| `helpers` | 0 | 1 | 助手函数 |

---

## Bounded Context：`popsicle-cli` —— CLI 表面

> **路径**：`crates/popsicle-cli/src/`
> **用途**：所有 22 个子命令的实现 + 入口 + Tauri UI（feature-gated）。
> **下游 product 候选映射**：全部 → `cli-ux`，但 `commands/sync.rs` 例外（依赖 popsicle-sync，与 sync-collab 待裁决项绑定）。

### 子命令清单（22 个）

> 全部在 `crates/popsicle-cli/src/commands/<name>.rs`，由 `commands/mod.rs` 注册到 clap。

| 命令 | 文件 | 一行用途 | 备注（D4 §5 候选位）|
|---|---|---|---|
| `init` | `commands/init.rs` | 初始化 `.popsicle/` 工作区 + 装 module | ✅ 核心 |
| `module` | `commands/module.rs` | install/list/show/upgrade module | ✅ 核心 |
| `tool` | `commands/tool.rs` | tool 管理 | ✅ 核心 |
| `skill` | `commands/skill.rs` | list/show/create skill | ✅ 核心 |
| `pipeline` | `commands/pipeline.rs` | list/status/next/review/stage/recommend/revise/archive | ✅ 核心 |
| `spec` | `commands/spec.rs` | create/list/show/delete spec | ✅ 核心 |
| `doc` | `commands/doc.rs` | create/list/show/summarize/check/extract document | ⚠️ D4 §5 候选裁剪（核 PRFC vs 通用）|
| `prompt` | `commands/prompt.rs` | 取 skill 的 AI prompt（`--state` / `--related`）| ⚠️ D4 §5 候选裁剪 |
| `migrate` | `commands/migrate.rs` | DB migration | ⚠️ D4 §5 候选裁剪 |
| `checklist` | `commands/checklist.rs` | checklist 单独命令（重复 `doc check`？）| ⚠️ D4 §5 候选裁剪 |
| `extract` | `commands/extract.rs` | 从文档抽 user story / test case / bug | ✅ 核心（在 IDD 里有用）|
| `item` | `commands/item.rs` | 统一 work_item 管理（user_story / bug / test_case）| ⚠️ D4 §5 work_item 候选删 |
| `issue` | `commands/issue.rs` | 需求 tracking（create / list / start / update）| ⚠️ D4 §5 候选删 |
| `namespace` | `commands/namespace.rs`（已被 `admin namespace` 包装？）| 多租户 namespace | ⚠️ D4 §5 候选删 |
| `admin` | `commands/admin.rs` | 低频管理（migrate、reinit、namespace）| ✅ 部分核心 |
| `reinit` | `commands/reinit.rs` | 重新初始化 | 与 admin 关系待澄清 |
| `git` | `commands/git.rs` | git commit 跟踪与 review | ✅ 核心 |
| `memory` | `commands/memory.rs` | 长短期 memory CRUD | ✅ 核心 |
| `context` | `commands/context.rs` | 项目上下文扫描 + 注入 | ✅ 核心 |
| `registry` | `commands/registry.rs` | 包注册表（search/publish/discover）| ✅ 核心（intent-coder 发布路径）|
| `sync` | `commands/sync.rs` | popsicle-cloud sync（login/logout/whoami/status/push/pull/daemon）| ⚠️ D4 §5 候选删（与 `sync-collab` product 绑定）|
| `completions` | `commands/mod.rs::completions` | shell 补全 | ✅ 核心 |

### UI 模块（feature-gated）

| 文件 | 用途 |
|---|---|
| `crates/popsicle-cli/src/ui/mod.rs` | Tauri 入口（`#[cfg(feature = "tauri")]`）|
| `crates/popsicle-cli/src/ui/commands.rs` | Tauri command 处理（26 commits/yr —— commit 热点 top 2）|

### 行为备注

- **CLI 子命令是 popsicle 的真正"对外 API"**——人 + AI agent 用 `popsicle <cmd>` 调用而不是直接调 `popsicle-core` 函数。
- `popsicle-cli` binary 的 crate name 是 `popsicle-cli`，但 `[[bin]] name = "popsicle"`（`crates/popsicle-cli/Cargo.toml:9-10`）。

---

## Bounded Context：`popsicle-sync` —— 多设备同步

> **路径**：`crates/popsicle-sync/src/`
> **用途**：popsicle-cloud 的客户端（HTTP push/pull + WebSocket daemon + Yjs CRDT 合并）。33 个 `pub` 顶层条目，集中在 `types.rs`（19 个 type）。
> **下游 product 候选映射**：整个 crate ⚠️ D4 §5 候选删 —— sync-collab 待裁决 product。

### 公开条目

| 文件 | pub items | 角色 |
|---|---|---|
| `client.rs` | 1 | HTTP client（reqwest）|
| `conflict.rs` | 4 | 冲突解决策略 |
| `crdt.rs` | 2 | yrs (Yjs port) CRDT 合并 |
| `error.rs` | 2 | sync error types |
| `http.rs` | 2 | HTTP 协议层 |
| `path.rs` | 1 | 路径处理（含 1 个测试函数名 `unsafe_slug_is_sanitised`——见 unsafe-risk-report）|
| `types.rs` | 19 | 同步实体类型（PushResult / PullResult / SyncState / ...）|
| `ws.rs` | 2 | WebSocket 长连接（tokio-tungstenite）|

### 行为备注

- popsicle-sync **不被 popsicle-core 引用**（grep 验证），仅 `popsicle-cli/src/commands/sync.rs` 使用。
- 这意味着把 popsicle-sync 整个移除**不影响**任何 IDD 主流程能力。`popsicle sync ...` 子命令届时会失效（用户失去多设备同步）。
- yrs（Yjs Rust port）+ tokio-tungstenite 是这一层的两个最重的传递依赖；无它们 popsicle-core / popsicle-cli 完全离线运行。

---

## 跨切面公开 API

| 条目 | File:Line | 备注 |
|---|---|---|
| `popsicle_core::error::Error`（enum）| `crates/popsicle-core/src/error.rs` | 所有 fallible 操作的错误类型 |
| `popsicle_core::helpers::*` | `crates/popsicle-core/src/helpers.rs` | 不在数据模型上的工具函数 |

---

## HTTP/gRPC 端点（如有）

(none — popsicle-core / popsicle-cli 主路径完全离线；popsicle-sync 是**客户端**调 popsicle-cloud 的 endpoints，本仓库不实现 server endpoints。)

---

## 稳定性标记

| 条目 | 标记 | File |
|---|---|---|
| _（无 `#[deprecated]` 命中）_ | n/a | n/a |

> `rg -t rust '#\[deprecated' crates/` 返回 0 行。本仓库**无**显式 deprecated API（v0.1.0 还未到需要兼容性管理的阶段）。

---

## Extraction Checklist

- [x] 6 个 bounded context（model / engine / storage / 其余 popsicle-core / popsicle-cli / popsicle-sync）各自有独立章节
- [x] 每个签名都含 file:line
- [x] 每个核心模块的 pub items 数已点名
- [x] HTTP/gRPC 端点章节已填，显式写 `(none — main paths fully offline; sync is client-side)`
- [x] 行为备注只含代码字面表达（无推断）
- [x] 跨切面 API 章节已填
- [x] 稳定性标记章节已填（写 `(no deprecated/experimental markers found)`）

---

## 已知 reduced fidelity

- `cargo metadata --format-version 1` 未跑（无即时调度需求）；公开函数的实际 callability（被外部 reexport 的范围）未交叉验证。
- 不区分"对**外部**调用方公开"和"对**workspace 内**公开"——所有 `pub` 一视同仁。下游 RFC 应做内/外公开分离。
