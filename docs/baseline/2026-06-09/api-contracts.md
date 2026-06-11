# API Contracts — artifact-system scope @ popsicle c76d729

> **Scope**: slice-2 (artifact-system) 8 模块的 public 表面。源 commit `c76d729`。
> **方法**: `rg '^\s*pub (fn|struct|enum|trait|type|const)'` + 直读签名。每条 cite file:line。
> **记录员声明**: 只记 public 契约，不评判设计。

## model/document.rs — Document 实体

| 条目 | 签名（节选）| 行 |
|---|---|---|
| `struct Document` | 通用文档：YAML frontmatter + Markdown body；所有 skill artifact 共用此模型 | `model/document.rs:9` |
| `struct RawDocument` | `{ frontmatter: String, body: String }`，frontmatter/body 分割结果 | `model/document.rs:49` |
| `Document::new(doc_type, title, skill_name, pipeline_run_id, spec_id) -> Self` | 新文档；`status="active"`、`version=1`、`id=uuid_v4` | `model/document.rs:55` |
| `Document::new_revision(&self, new_run_id) -> Self` | 修订版：`version+1`、`parent_doc_id=Some(self.id)`、`status="active"` | `model/document.rs:85` |
| `Document::to_file_content(&self) -> Result<String>` | 序列化为 `---\n{yaml}---\n\n{body}` | `model/document.rs:108` |
| `Document::from_file_content(content, file_path) -> Result<Self>` | 解析 frontmatter+body；缺 `---` 返回 `InvalidDocumentFormat` | `model/document.rs:114`（守卫 `:126-129`）|

> 关键字段（`model/document.rs:9-41`）：`id, doc_type, title, status, skill_name, pipeline_run_id, spec_id, version:u32, parent_doc_id:Option<String>, tags, summary(#[serde(skip)]), metadata, created_at, updated_at, body(#[serde(skip)]), file_path(#[serde(skip)])`。`status` 是 `String`（无独立状态机枚举）。

## model/namespace.rs — Namespace 实体

| 条目 | 签名 | 行 |
|---|---|---|
| `struct Namespace` | 多 spec 容器 | `model/namespace.rs:7` |
| `enum NamespaceStatus { Active, Completed, Archived }` | | `model/namespace.rs:28` |
| `impl Display for NamespaceStatus` | | `model/namespace.rs:34` |
| `impl FromStr for NamespaceStatus` | | `model/namespace.rs:44` |
| `Namespace::new(name, description) -> Self` | | `model/namespace.rs:57` |

## model/work_item.rs — WorkItem 实体（即将重命名为 task_chunk_entity）

| 条目 | 签名 | 行 |
|---|---|---|
| `struct WorkItem` | 统一 user_story/bug/test_case 实体 | `model/work_item.rs:13` |
| `enum WorkItemKind` | （bug / story / testcase）| `model/work_item.rs:42` |
| `WorkItemKind::key_prefix(&self) -> &'static str` | 生成 key 前缀（如 BUG-/TC-）| `model/work_item.rs:50` |
| `WorkItemKind::as_str(&self) -> &'static str` | | `model/work_item.rs:58` |
| `impl Display/FromStr for WorkItemKind` | | `model/work_item.rs:67,73` |
| `WorkItem::new(key, kind, title) -> Self` | | `model/work_item.rs:86` |
| `WorkItem::field_str(&self, name) -> Option<&str>` | 从 JSON `fields` blob 读字段 | `model/work_item.rs:112` |
| `WorkItem::set_field(&mut self, name, value: serde_json::Value)` | 写 JSON `fields` blob | `model/work_item.rs:117` |

> kind-specific 数据存 JSON `fields` blob（`field_str`/`set_field`），与 `.github/copilot-instructions.md` 描述的统一表设计一致。

## engine/markdown.rs — Markdown 智能编辑（6 pub fn，纯函数）

| fn | 签名 | 行 |
|---|---|---|
| `extract_section_content(after_header) -> String` | 取 header 之后到下一个 H2 的内容 | `engine/markdown.rs:5` |
| `is_template_placeholder(content) -> bool` | 判断是否仍是模板占位（guard 判空依据）| `engine/markdown.rs:23` |
| `extract_sections(body, section_names) -> String` | 抽取指定 H2 段集合 | `engine/markdown.rs:57` |
| `extract_summary(body) -> String` | 抽取摘要（Relevance::Low 装配用）| `engine/markdown.rs:77` |
| `extract_tags(body, skill_name, doc_type) -> Vec<String>` | 生成索引 tag | `engine/markdown.rs:125` |
| `upsert_section(doc, section_name, content, append) -> String` | 插入/更新/追加 H2 段 | `engine/markdown.rs:159` |

## engine/guard.rs — 状态转换守卫（3 pub 条目）

| 条目 | 签名 | 行 |
|---|---|---|
| `struct GuardResult { passed: bool, guard_name: String, message: String }` | | `engine/guard.rs:9` |
| `check_guard(guard, doc, all_docs, registry, pipeline, run) -> Result<GuardResult>` | guard 入口；`;` 分隔多 guard，全过才过 | `engine/guard.rs:26` |
| `count_checkboxes(text) -> (usize, usize)` | 数 `- [x]`/`- [ ]`，返回 (checked, unchecked) | `engine/guard.rs:270` |

**guard DSL 支持的全部类型（`check_single_guard` 硬编码，`engine/guard.rs:65-96`）**：
1. `upstream_approved` — 检查上游 stage 全 `Completed`/`Skipped`（`:75-76`，逻辑 `:103-169`）
2. `has_sections:<csv>` — 各 H2 段存在且非模板占位（`:79-82`，逻辑 `:173-213`）
3. `checklist_complete` / `checklist_complete:<section>` — 复选框全勾（`:84-90`，逻辑 `:217-266`）
4. 未知 guard → `Err(PopsicleError::InvalidSkillDef("Unknown guard type: ..."))`（`:92-95`）

> 事实：guard 类型是 `if let` 链硬编码，无 trait 扩展点（`engine/guard.rs:65-96`）。

## engine/context.rs — Relevance 排序的 context 装配

| 条目 | 签名 | 行 |
|---|---|---|
| `struct ContextInput` | 单个上游输入 | `engine/context.rs:13` |
| `struct ContextPart` | 装配后的一段 | `engine/context.rs:24` |
| `struct AssembledContext` | `{ parts, full_text }` | `engine/context.rs:34` |
| `assemble_input_context(inputs) -> AssembledContext` | 按 `Relevance` 排序；Low→`extract_summary`、Medium→`extract_sections`(或全文)、High→全文 | `engine/context.rs:51`（逻辑 `:52-77`）|

## engine/context_layer.rs — 可插拔 prompt 分层（唯一扩展 trait）

| 条目 | 签名 | 行 |
|---|---|---|
| `trait ContextLayer: Send + Sync` | prompt section 抽象 | `engine/context_layer.rs:22` |
| `assemble_layers(layers: Vec<Box<dyn ContextLayer>>, base_prompt) -> String` | 组合各层 | `engine/context_layer.rs:34` |
| `struct ProjectContextLayer` + `impl ContextLayer` | 项目上下文层 | `engine/context_layer.rs:55,59` |
| `struct MemoriesLayer` + `impl ContextLayer` | 记忆层（依赖 `memory::Memory`）| `engine/context_layer.rs:76,80` |
| `struct HistoricalRefsLayer` + `impl ContextLayer` | 历史引用层 | `engine/context_layer.rs:105,109` |
| `struct UpstreamDocsLayer` + `impl ContextLayer` | 上游文档层（依赖 `storage::DocumentRow`）| `engine/context_layer.rs:146,150` |

> 事实：`ContextLayer` 是本范围内唯一的 `pub trait`（可扩展点）；guard/extractor 均为硬编码函数。与 `.github/copilot-instructions.md` "Built-in layers: ProjectContextLayer, MemoriesLayer, HistoricalRefsLayer, UpstreamDocsLayer" 一致。

## engine/extractor.rs — WorkItem 提取（3 pub fn，regex 驱动）

| fn | 签名 | 行 |
|---|---|---|
| `extract_user_stories(doc) -> Vec<WorkItem>` | 从 PRD 抽 Story；fields=persona/goal/benefit/acceptance | `engine/extractor.rs:10` |
| `extract_test_cases(doc, test_type) -> Vec<WorkItem>` | 从 test-spec 抽 TestCase；fields=test_type/priority_level/steps | `engine/extractor.rs:69` |
| `extract_bugs(doc) -> Vec<WorkItem>` | 抽 Bug | `engine/extractor.rs:125` |

## 公开表面统计

| 模块 | pub 顶层条目数 |
|---|---|
| document | 6 |
| namespace | 3 |
| work_item | 7 |
| markdown | 6 |
| guard | 3 |
| context | 4 |
| context_layer | 6 |
| extractor | 3 |
| **合计** | **38** |

## Checklist

- [x] 每条 public 条目都 cite file:line
- [x] guard DSL 的全部类型已枚举（含未知 guard 的错误分支）
- [x] 标注了唯一扩展 trait（ContextLayer）vs 硬编码函数（guard/extractor）
- [x] 跨 product 依赖（registry/pipeline/run/memory/storage）已在 dependency-graph.md 记录
- [x] 无观点句
