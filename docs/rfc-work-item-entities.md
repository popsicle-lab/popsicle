## Summary

本 RFC 提出在 Popsicle 中引入三种独立的 Work Item 实体——Bug、UserStory、TestCase——作为 Issue 的下级结构化子实体。通过文档提取引擎（extractor）从 PRD、test-spec、bug-report 等 Skill 产出物中自动解析并持久化到 SQLite，结合 Skill Hook 实现 pipeline 完成时的自动提取，以及测试失败时的自动 Bug 创建和去重，形成从需求到实现到验证的完整可追溯闭环。

## Motivation

### 当前痛点

1. **需求不可追溯** — PRD 文档中的 User Story 以自然语言嵌入在 Markdown 中，没有独立的结构化表示。无法回答"这个 Story 是否被测试覆盖""哪些 Story 已验证"等问题。

2. **测试规格无原子粒度** — test-spec 文档是完整文档，每个测试用例没有独立标识（ID/Key）。无法追踪单个测试用例的通过/失败历史、无法将测试用例关联到 User Story 的验收条件。

3. **Bug 追踪断裂** — `bug-tracker` Skill 产出的 bug-report 文档是叙述性文档，bug 信息分散在 Markdown 中。测试失败时需要手动创建 Bug，无法自动关联到失败的 TestCase，也无法去重（同一测试用例反复失败不应产生多个 Bug）。

4. **Issue 容器空心化** — Issue 模型作为工作项容器，内部只有 Pipeline Run 和 Document 的聚合。缺乏 User Story、TestCase、Bug 级别的原子粒度追踪，无法提供真正的项目进度和质量视图。

### 与现有机制的关系

| 机制 | 粒度 | 可追溯 | 自动化 |
|------|------|--------|--------|
| Issue | 工作项容器 | Pipeline Run 级 | `popsicle issue start` |
| Document | 完整文档 | Skill 级 | Guard + Hook |
| **Bug** | **原子级 Bug** | **TestCase → Bug → Fix Commit** | **测试失败自动创建** |
| **UserStory** | **原子级需求** | **Story → AC → TestCase** | **PRD 完成时提取** |
| **TestCase** | **原子级测试** | **TC → TestRunResult** | **test-spec 完成时提取** |

Work Item Entities 不替代 Document 或 Issue，而是在文档的 Markdown 内容与 Issue 的工作流之间补充原子粒度的结构化追踪层。

## Proposal

### Detailed Design

#### 1. 架构总览

```
Issue (容器)
  ├── UserStory (从 PRD 提取)
  │     └── AcceptanceCriterion
  │           └── TestCase (关联)
  ├── TestCase (从 test-spec 提取)
  │     └── TestRunResult (运行记录)
  └── Bug (从 bug-report 提取 / 测试失败自动创建)
        └── related_test_case_id (关联)
```

三种实体通过 `issue_id` 和 `pipeline_run_id` 关联到 Issue 和 Pipeline Run，但也支持独立存在（`issue_id` 为 `Option<String>`），以适应手动创建或跨 Issue 复用的场景。

#### 2. 数据模型

**Bug**

```rust
pub struct Bug {
    pub id: String,
    pub key: String,                    // BUG-{PREFIX}-{SEQ}
    pub title: String,
    pub description: String,
    pub severity: BugSeverity,          // Blocker, Critical, Major, Minor, Trivial
    pub priority: Priority,             // 复用 issue.rs 的 Priority
    pub status: BugStatus,              // Open, Confirmed, InProgress, Fixed, Verified, Closed, WontFix
    pub steps_to_reproduce: Vec<String>,
    pub expected_behavior: String,
    pub actual_behavior: String,
    pub environment: Option<String>,
    pub stack_trace: Option<String>,
    pub source: BugSource,              // Manual, TestFailure, DocExtracted
    pub related_test_case_id: Option<String>,
    pub related_commit_sha: Option<String>,
    pub fix_commit_sha: Option<String>,
    pub issue_id: Option<String>,
    pub pipeline_run_id: Option<String>,
    pub labels: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

`BugSource` 区分 Bug 的来源：`Manual`（手动创建）、`TestFailure`（测试失败自动创建）、`DocExtracted`（从文档提取）。这允许追踪 Bug 的发现方式。

**TestCase + TestRunResult**

```rust
pub struct TestCase {
    pub id: String,
    pub key: String,                    // TC-{PREFIX}-{SEQ}
    pub title: String,
    pub description: String,
    pub test_type: TestType,            // Unit, Api, E2e, Ui
    pub priority_level: TestPriority,   // P0, P1, P2
    pub status: TestCaseStatus,         // Draft, Ready, Automated, Deprecated
    pub preconditions: Vec<String>,
    pub steps: Vec<String>,
    pub expected_result: String,
    pub source_doc_id: Option<String>,
    pub user_story_id: Option<String>,
    pub issue_id: Option<String>,
    pub pipeline_run_id: Option<String>,
    pub labels: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct TestRunResult {
    pub id: String,
    pub test_case_id: String,
    pub passed: bool,
    pub duration_ms: Option<u64>,
    pub error_message: Option<String>,
    pub commit_sha: Option<String>,
    pub run_at: DateTime<Utc>,
}
```

TestCase 与 TestRunResult 是一对多关系：一个 TestCase 可以有多次运行记录，`latest_test_run` 返回最近一次结果用于覆盖率计算。

**UserStory + AcceptanceCriterion**

```rust
pub struct UserStory {
    pub id: String,
    pub key: String,                    // US-{PREFIX}-{SEQ}
    pub title: String,
    pub description: String,
    pub persona: String,
    pub goal: String,
    pub benefit: String,
    pub priority: Priority,
    pub status: UserStoryStatus,        // Draft, Accepted, Implemented, Verified
    pub source_doc_id: Option<String>,
    pub issue_id: Option<String>,
    pub pipeline_run_id: Option<String>,
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct AcceptanceCriterion {
    pub id: String,
    pub description: String,
    pub verified: bool,
    pub test_case_ids: Vec<String>,
}
```

AcceptanceCriterion 的 `test_case_ids` 提供了 UserStory → TestCase 的可追溯链接，可通过 `story link` 命令将 AC 关联到 TestCase。

#### 3. 存储层

在 SQLite `IndexDb` 中新增 5 张表：

| 表 | 主键 | 核心索引 |
|---|---|---|
| `bugs` | id | key, status, severity, issue_id, pipeline_run_id |
| `test_cases` | id | key, test_type, priority_level, status, user_story_id, pipeline_run_id |
| `test_runs` | id | test_case_id |
| `user_stories` | id | key, status, issue_id, pipeline_run_id |
| `acceptance_criteria` | id | user_story_id |

Key 序列号通过 `next_*_seq(prefix)` 函数生成，查询当前最大序号后 +1，格式为 `{TYPE}-{PREFIX}-{SEQ}`（如 `BUG-PRJ-1`、`TC-PRJ-42`、`US-PRJ-7`）。

`Vec<String>` 类型的字段（`steps_to_reproduce`、`steps`、`preconditions`、`labels`、`test_case_ids`）序列化为 JSON 字符串存储。

#### 4. 文档提取引擎

新建 `engine/extractor.rs`，提供三个纯函数，从 Document 的 Markdown body 中提取结构化实体：

**`extract_user_stories(doc: &Document) -> Vec<UserStory>`**

解析 PRD 的 `## User Stories & Acceptance Criteria` section：
- 识别 `### Story N: [Title]` 模式作为 Story 分隔
- 解析 `**As a** ... **I want to** ... **So that** ...` 模式提取 persona/goal/benefit
- 解析 `- [ ]` / `- [x]` checklist 项作为 AcceptanceCriterion

**`extract_test_cases(doc: &Document, test_type: TestType) -> Vec<TestCase>`**

解析 test-spec 文档：
- H3 标题作为 TestCase title（跳过已知的 section header 如 "Summary"、"Overview"）
- `- [ ]` checklist 或 `- ` bullet 项作为 steps
- 从 `## P0` / `## P1` / `## P2` section 上下文推断 `priority_level`
- `test_type` 由调用方指定（来自 Skill 的 artifact type）

**`extract_bugs(doc: &Document) -> Vec<Bug>`**

解析 bug-report 文档：
- H3 标题（可选 `BUG-XXXX:` 前缀）作为 Bug title
- 解析 `**Severity**:` / `**Expected**:` / `**Actual**:` / `**Steps to reproduce**:` 结构化字段
- 排除模板占位符（标题包含 `[Title]` 的条目）

设计原则：结构化解析优先，不引入 LLM 依赖。这与 Guard 的确定性验证哲学一致——提取逻辑可预测、可测试、零运行时开销。模板格式通过 Skill 的 template 文件保证一致性，使得结构化解析可行。

#### 5. 自动化闭环

**Skill Hook 触发提取**

6 个 Skill 的 `hooks.on_complete` 配置自动提取命令：

| Skill | Hook 命令 |
|-------|-----------|
| prd | `popsicle extract user-stories --from-doc $POPSICLE_DOC_ID` |
| api-test-spec | `popsicle extract test-cases --from-doc $POPSICLE_DOC_ID --type api` |
| e2e-test-spec | `popsicle extract test-cases --from-doc $POPSICLE_DOC_ID --type e2e` |
| priority-test-spec | `popsicle extract test-cases --from-doc $POPSICLE_DOC_ID --type unit` |
| ui-test | `popsicle extract test-cases --from-doc $POPSICLE_DOC_ID --type ui` |
| bug-tracker | `popsicle extract bugs --from-doc $POPSICLE_DOC_ID` |

`$POPSICLE_DOC_ID` 和 `$POPSICLE_RUN_ID` 由现有的 `HookContext` 机制提供，无需扩展。

**测试失败 → Bug 自动创建**

```bash
popsicle bug record --from-test TC-5 --error "assertion failed: expected 200 got 500" --run <run-id>
```

逻辑：
1. 查找 TestCase TC-5
2. 创建 TestRunResult（passed=false）
3. 检查是否已有相同 `related_test_case_id` 的 Open Bug（去重）
4. 若无，创建新 Bug，自动关联 `related_test_case_id`、继承 `issue_id` 和 `pipeline_run_id`

**Issue 关联传播**

extract 命令通过 `$POPSICLE_RUN_ID` 查找关联的 Issue（`find_issue_by_run_id`），自动将 `issue_id` 写入新创建的实体。这确保了从 `popsicle issue start` 开始，所有派生的 UserStory/TestCase/Bug 都自动关联回 Issue。

### Interface Changes

#### CLI 命令族

```bash
# Bug 管理
popsicle bug create --title "..." [--severity major] [--issue <key>] [--run <run-id>]
popsicle bug list [--severity <sev>] [--status <status>] [--issue <key>]
popsicle bug show <key>
popsicle bug update <key> [--status fixed] [--fix-commit <sha>]
popsicle bug link <key> --commit <sha>
popsicle bug record --from-test <tc-key> --error "..." [--run <run-id>]

# TestCase 管理
popsicle test list [--type e2e] [--priority p0] [--status <s>]
popsicle test show <key>
popsicle test extract --from-doc <doc-id> [--type e2e]
popsicle test run-result <key> --passed/--failed [--commit <sha>] [--error "..."]
popsicle test coverage [--run <run-id>]

# UserStory 管理
popsicle story list [--issue <key>] [--status <s>]
popsicle story show <key>
popsicle story create --title "..." [--issue <key>] [--persona "..."]
popsicle story extract --from-doc <doc-id>
popsicle story update <key> --status accepted
popsicle story link <key> --ac <ac-id> --test-case <tc-key>

# 统一提取入口（Hook 调用目标）
popsicle extract user-stories --from-doc <doc-id>
popsicle extract test-cases --from-doc <doc-id> [--type e2e]
popsicle extract bugs --from-doc <doc-id>
```

所有命令支持 `--format json`。

#### Tauri UI 命令

9 个新的 Tauri 命令：

- `list_bugs` / `get_bug` / `create_bug` / `update_bug`
- `list_test_cases` / `get_test_case` / `get_test_coverage`
- `list_user_stories` / `get_user_story`

#### DTO 类型

- `BugInfo`（列表）/ `BugFull`（详情）
- `TestCaseInfo` / `TestCaseFull` / `TestRunInfo` / `TestCoverageSummary`
- `UserStoryInfo` / `UserStoryFull` / `AcceptanceCriterionInfo`

## Rationale and Alternatives

### Why This Approach (方案 B: 独立实体 + Issue 容器)

1. **原子粒度追溯** — UserStory、TestCase、Bug 作为独立实体拥有自己的 ID 和生命周期，可以建立 Story → AC → TestCase → TestRunResult → Bug 的完整追溯链。这是 Document 级别的追踪无法提供的。

2. **灵活关联** — 每个实体可以独立于 Issue 存在（手动创建的 Bug、跨 Issue 复用的 TestCase），也可以通过 `issue_id` 关联到 Issue。这比将它们硬编码为 Issue 的子文档更灵活。

3. **自动化友好** — 结构化实体天然适配自动化操作：去重（按 `related_test_case_id` 查找已有 Bug）、覆盖率统计（`test coverage`）、进度追踪（`story list --status`）。这些操作在纯 Markdown 文档上需要复杂的文本解析。

4. **与现有模型正交** — Bug/UserStory/TestCase 与 Document/Issue/PipelineRun 是不同维度的概念，独立建模避免了在现有模型上硬塞字段的尴尬。

### Alternative A: Document 扩展方案

在现有 Document 模型上增加 `doc_type` 变体（BugReport、UserStory、TestCase），复用 Document 的存储和工作流。

- **Pros**：最小代码量，复用现有 CRUD 和 UI
- **Cons**：Document 的字段结构（body/status/workflow）与 Bug/TestCase 的领域字段不匹配；Document 的 status 是 workflow state（draft → review → approved），与 BugStatus（open → fixed → verified）语义不同；强行共享模型会导致大量 `Option` 字段和类型判断逻辑

不选择的原因：领域模型的强行统一会导致更多的复杂性，而非更少。

### Alternative C: 完全内嵌 Issue 方案

将 Bug/UserStory/TestCase 作为 Issue 的内嵌子结构（如 `Issue.bugs: Vec<Bug>`），不建独立表。

- **Pros**：查询时自动聚合，无需 JOIN
- **Cons**：跨 Issue 复用困难（一个 TestCase 可能被多个 Issue 引用）；Issue 模型膨胀；独立查询（如"所有 P0 测试用例"）需要遍历全部 Issue

不选择的原因：实体间的关联是多对多的，内嵌方案会人为限制为一对多。

### Cost of Inaction

不实现 Work Item Entities，Popsicle 的追踪能力将停留在 Document 级别：
- 无法回答"有多少 User Story 已验证"
- 无法追踪单个测试用例的通过/失败历史
- 测试失败时需要手动创建 Bug 并手动关联
- Issue 的进度视图只能看到文档状态，无法看到需求和质量的原子粒度

## Open Questions

- Questions resolved during design:
  - Bug/TestCase/UserStory 是独立实体还是 Document 子类型？→ 选择独立实体（方案 B），原因见 Rationale
  - 提取引擎是否引入 LLM？→ 不引入，结构化解析优先，模板保证格式一致性

- Questions to resolve during usage:
  - 提取引擎对非标准格式的 Markdown 容错性如何？需要在实际文档上验证解析质量
  - `test coverage` 的覆盖率定义是否足够？当前仅统计 pass/fail/no-run 三态，是否需要支持部分通过或条件跳过？
  - AcceptanceCriterion 到 TestCase 的关联是否应支持自动推断（基于描述相似度），还是仅依赖手动 `story link`？

## Implementation Plan

- [x] Phase 1 — Bug 模型：`bug.rs`、存储层 CRUD、DTO、CLI 命令、Tauri 命令
- [x] Phase 2 — TestCase 模型：`testcase.rs`、存储层 CRUD、提取引擎 `extractor.rs`、DTO、CLI 命令、Tauri 命令
- [x] Phase 3 — UserStory 模型：`story.rs`、存储层 CRUD、提取引擎扩展、DTO、CLI 命令、Tauri 命令
- [x] Phase 4 — 自动化闭环：统一 `extract` 命令、6 个 Skill Hook、`bug record --from-test` 去重逻辑、Issue 关联传播

### 文件变更总览

**新文件 (8)**
- `crates/popsicle-core/src/model/bug.rs`
- `crates/popsicle-core/src/model/testcase.rs`
- `crates/popsicle-core/src/model/story.rs`
- `crates/popsicle-core/src/engine/extractor.rs`
- `crates/popsicle-cli/src/commands/bug.rs`
- `crates/popsicle-cli/src/commands/test.rs`
- `crates/popsicle-cli/src/commands/story.rs`
- `crates/popsicle-cli/src/commands/extract.rs`

**修改文件 (10)**
- `crates/popsicle-core/src/model/mod.rs` — 注册 3 个新模块
- `crates/popsicle-core/src/engine/mod.rs` — 导出 extractor
- `crates/popsicle-core/src/storage/index.rs` — 5 张新表 + CRUD 方法
- `crates/popsicle-core/src/dto.rs` — 9 个新 DTO 类型
- `crates/popsicle-cli/src/commands/mod.rs` — 注册 4 个新命令模块
- `crates/popsicle-cli/src/ui/commands.rs` — 9 个 Tauri 命令
- `crates/popsicle-cli/src/ui/mod.rs` — invoke_handler 注册
- 6 个 `skills/*/skill.yaml` — 添加 on_complete hook
