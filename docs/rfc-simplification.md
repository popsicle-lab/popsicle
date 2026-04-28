# RFC: 概念精简（Concept Slimming）

**Status**: Implemented (breaking change, no migration)
**Scope**: Items #2, #5, #6, #8 from the simplification audit

## Motivation

Popsicle 的 README "Core Concepts" 列出 ~15 个一等概念，加上跨 24 个 CLI 子命令、~30K LOC、2.5K LOC 的 SQLite 索引层。重度 AI 协同的领域工具不应有这种心智负担。

本 RFC 通过四个合并把概念面积砍掉约 40%：

| ID | 改动 | 减少 |
|---|---|---|
| #2 | `Bug` / `UserStory` / `TestCase` → 单一 `WorkItem` | 3 model + 3 storage 表 + 3 CLI cmd |
| #5 | `Discussion` 折叠为 `Document` 的一个 `kind` | 1 model + 3 表 + 1 CLI cmd |
| #6 | `ProjectContext` + `Memory` 抽象为 `ContextLayer` | 散落的注入点 → 单一接口 |
| #8 | CLI 命令分组与精简 | 24 → ~14 个顶层子命令 |

**注意**：本 RFC 是**破坏性变更**。用户授权"不考虑迁移、不兼容之前"，因此本次实现直接删除旧表/旧命令；老 `.popsicle/popsicle.db` 需要 `popsicle init --force` 重建。

## #2 — WorkItem 统一

### 旧模型

```
crates/popsicle-core/src/model/{bug,story,testcase}.rs   ─ 3 个 struct + 多套 enum
crates/popsicle-core/src/storage/index.rs                ─ 6 个表（bugs, user_stories,
                                                                  test_cases, test_runs,
                                                                  acceptance_criteria, …）
crates/popsicle-cli/src/commands/{bug,story,test}.rs    ─ 3 个 CLI 命令
```

差异主要在 metadata（severity / acceptance criteria / steps）。这些差异不值得 3 倍代码量。

### 新模型

`crates/popsicle-core/src/model/work_item.rs`：

```rust
pub struct WorkItem {
    pub id: String,
    pub key: String,                // BUG-PRJ-1 / STORY-PRJ-1 / TC-PRJ-1
    pub kind: WorkItemKind,         // Bug | Story | TestCase
    pub title: String,
    pub description: String,
    pub status: String,             // 自由格式（不再强枚举）
    pub priority: Priority,
    pub labels: Vec<String>,
    pub issue_id: Option<String>,
    pub pipeline_run_id: Option<String>,
    pub source_doc_id: Option<String>,
    pub fields: serde_json::Value,  // kind-specific：steps、severity、acceptance 等
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum WorkItemKind { Bug, Story, TestCase }
```

存储表合并为单一 `work_items`：

```sql
CREATE TABLE work_items (
    id TEXT PRIMARY KEY,
    key TEXT NOT NULL UNIQUE,
    kind TEXT NOT NULL,         -- 'bug' | 'story' | 'testcase'
    title TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT '',
    priority TEXT NOT NULL DEFAULT 'medium',
    labels TEXT NOT NULL DEFAULT '[]',
    issue_id TEXT,
    pipeline_run_id TEXT,
    source_doc_id TEXT,
    fields TEXT NOT NULL DEFAULT '{}',  -- JSON for kind-specific extras
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

被牺牲的能力（被认为低价值）：

- `TestRunResult`（独立表跟踪每次 test run）— 删除；测试结果交给外部 CI 工具
- `AcceptanceCriterion`（独立表 + 与 TestCase 的多对多关联）— 退化为 `fields.acceptance` 中的字符串数组
- 严格枚举校验（BugSeverity / TestType / BugStatus）— 退化为 `labels` 或 `fields` 中的字符串
- `find_open_bug_by_test_case`（去重）— 删除

### CLI

```
旧:  popsicle bug create  --severity major --priority high "..."
     popsicle story create --persona ... --goal ... "..."
     popsicle test create  --type unit --priority p1 "..."

新:  popsicle item add --kind bug   --priority high --field severity=major "..."
     popsicle item add --kind story --field persona="..." --field goal="..." "..."
     popsicle item add --kind tc    --field steps='["a","b"]' "..."
     popsicle item list [--kind bug] [--issue ISSUE-1]
     popsicle item show BUG-PRJ-1
     popsicle item update BUG-PRJ-1 --status fixed
     popsicle item link  BUG-PRJ-1 --commit abc123
```

## #5 — Discussion 折叠为 Document.kind

### 旧模型

`Discussion` 是一等实体：3 张表（`discussions`、`discussion_messages`、`discussion_roles`）、独立 model 文件、独立 CLI 命令、独立 UI 页面。

它的本质是 **"`arch-debate` / `product-debate` skill 产出的特殊文档"**，强行成为一等公民违反了"Skill 是能力原子单元"的原则。

### 新模型

不再有 `Discussion` 实体。**Discussion 就是 `doc_type = "discussion"` 的 Document**，body 用既有的 markdown 渲染（Phase H2 标题 + 角色块）。

- 删除 `discussion.rs`、3 张表、所有 `*_discussion_*` 存储方法
- 删除 `popsicle discussion` CLI 命令
- Skill 引擎记录 discussion 时直接 `doc create --kind discussion`，body 通过现有的 `Document.body` 写入
- UI 端通过 `doc_type == "discussion"` 切换为气泡式渲染（保留视觉差异）

被牺牲：discussion message 的多次结构化追加（之前 `INSERT INTO discussion_messages`）。改为：每次 AI 输出整段后，文档 body 整段重写。这与 Markdown 文档的常规更新模式一致。

## #6 — ContextLayer 统一注入

### 旧模型

`engine::context` 内部硬编码两个数据源：

```rust
fn build_skill_context(...) -> Context {
    let project_ctx = scanner::scan(...);   // ProjectContext
    let mem = memory::store::current(...);  // Memory
    let upstream = ...;                      // Upstream documents
    // 硬编码组装顺序
}
```

未来若想加 RAG / 团队约定 / 实时 issue tracker，需要修改这个函数。

### 新模型

`crates/popsicle-core/src/engine/context_layer.rs`：

```rust
pub trait ContextLayer: Send + Sync {
    /// Layer name (for ordering / debug)
    fn name(&self) -> &str;
    /// Default relevance bucket
    fn relevance(&self) -> Relevance;
    /// Produce a context section (markdown). Empty string = skip.
    fn render(&self) -> String;
}
```

内置实现：

- `ProjectContextLayer { content: String }` — 折叠 `scanner` 输出的项目上下文
- `MemoriesLayer { memories: Vec<Memory> }` — 折叠跨会话记忆
- `HistoricalRefsLayer { refs: Vec<DocumentRow> }` — 折叠上下文检索出的历史参考
- `UpstreamDocsLayer { assembled: AssembledContext }` — 折叠现有的上游文档拼接逻辑

`assemble_layers(layers, base_prompt)` 按 `Relevance` 由低到高排序后拼接，最后附上 base prompt，这样高相关性的上文靠近 prompt 尾部（LLM 注意力偏近期）。`commands::prompt` 改为构造 `Vec<Box<dyn ContextLayer>>` 后交由 `assemble_layers` 渲染，不再硬编码顺序。

## #8 — CLI 命令分组

### 旧（24 个顶层）

```
init skill pipeline doc checklist git discussion issue bug story test
sync extract migrate reinit module tool spec namespace registry context
memory prompt completions ui
```

### 新（~14 个顶层）

```
init                 # 初始化项目
skill / pipeline     # 引擎核心：能力 & 编排
issue / spec         # 工作组织：需求 & 上下文
item                 # 统一的 bug/story/test
doc                  # 文档（含 checklist 子命令、含 extract）
prompt               # 取上下文 / 取 prompt（context 合并进来）
git                  # 提交追踪
memory               # 跨会话记忆
module / tool / registry  # 分发与扩展
sync                 # 云同步
admin                # migrate / reinit / namespace 等低频管理
completions / ui     # 终端工具
```

| 旧命令 | 去向 |
|---|---|
| `bug` / `story` / `test` | → `item --kind …` |
| `discussion` | 删除（→ `doc --kind discussion`） |
| `extract` | → `doc extract` 子命令 |
| `checklist` | → `doc check` 子命令 |
| `migrate` / `reinit` | → `admin migrate` / `admin reinit` |
| `namespace` | → `admin namespace`（低频，多数用户用默认 namespace） |
| `context` | → 合并进 `prompt`（`--context-only` flag） |

## 实施记录（变更摘要）

- **#2 WorkItem**：新增 `model::WorkItem` + 单表 `work_items`；删除 bug/story/testcase 三个 model 与所有旧表、存储方法、CLI。新 `popsicle item` 命令统一接收三种 kind。
- **#5 Discussion**：删除 `model::Discussion` + 3 张表 + 所有 `*_discussion_*` 存储 + `popsicle discussion` CLI + Tauri `list_discussions/get_discussion` 绑定。Discussion 语义交给 `Document.kind = "discussion"`。
- **#6 ContextLayer**：新增 `engine::context_layer` 模块，包含 `ContextLayer` trait、四个内置 layer 与 `assemble_layers` 函数。`commands::prompt` 改造为构造 layer 列表后调用，各个 section 拼接逻辑不再散落在 prompt 命令内部。
- **#8 CLI 分组**：DocCommand 新增 `extract` / `check` 子命令；新增 `commands::admin` 包装 `migrate` / `reinit` / `namespace`；顶层 `Command` enum 移除 5 个变体，添加 `Admin`。顶层子命令从 22 降到 17（`init / skill / pipeline / doc / git / issue / item / sync / module / tool / spec / registry / context / memory / prompt / admin / completions(+ui)`）。**未合并 `context` 进 `prompt`**：`context` 下含多个子命令且与 prompt 语义不一致，合并代价高于收益，暂保留。
- **DTO**：同步收敛DiscussionInfo / DiscussionFull 等。`popsicle-sync` 的 `EntityKind` 由 `Bug/Story/TestCase` 折叠为 `WorkItem`。
- **质量门**：`cargo fmt + clippy(-Dwarnings) + test --workspace --all-features` 全部通过，共 199 个单元测试。
- **UI Tauri 绑定**：Rust 端已同步移除 Discussion 与旧 work item 命令；前端页面 (`.tsx`) 仍引用旧 invoke 名，**待后续独立 PR 重构**（UI 只读且不在本 RFC 优先级内）。
- **文档**：本 RFC + README "Core Concepts" + `.github/copilot-instructions.md` 同步更新。

## 不影响的概念

`Skill` / `Pipeline` / `PipelineRun` / `Spec` / `Issue` / `Document` / `Module` / `Tool` / `Memory` / `Guard` / `Advisor` / `Project Context` 这些核心概念**保留语义**。本 RFC 只精简了"派生实体"和"散落的 CLI 表面"。
