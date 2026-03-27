---
name: Memory RFC and Implementation
overview: 更新现有 rfc-auto-memory.md 修正实现偏差，新建 rfc-document-index.md 描述第 3 层文档语义索引，然后按 P0/P1/P2 顺序实现完整三层记忆体系。
todos:
  - id: rfc-update-memory
    content: "修正 docs/rfc-auto-memory.md: scoring 公式、context-aware injection、open questions、implementation plan"
    status: completed
  - id: rfc-new-docindex
    content: "新建 docs/rfc-document-index.md: 完整 RFC 描述第 3 层文档语义索引设计"
    status: completed
  - id: p0-fix-scoring
    content: "P0: prompt.rs 中 load_ranked_memories 传入 skill tags + 变更文件，让 scoring 生效"
    status: completed
  - id: p1-schema-migration
    content: "P1a: SQLite documents 表加 summary + tags 列，更新 upsert/query/DocumentRow"
    status: completed
  - id: p1-summarize-cmd
    content: "P1b: 实现 popsicle doc summarize 命令 + approve 时自动触发摘要生成"
    status: completed
  - id: p2-fts5
    content: "P2a: 启用 FTS5，建虚拟表 + 同步触发器 + search_documents 方法"
    status: completed
  - id: p2-context-search
    content: "P2b: 实现 popsicle context search 命令"
    status: completed
  - id: p2-prompt-related
    content: "P2c: prompt --related flag 注入历史相关文档索引"
    status: completed
  - id: tests
    content: 为每个 Phase 编写对应测试
    status: completed
isProject: false
---

# Popsicle 三层记忆体系：RFC 更新 + 完整实现

## 第一部分：RFC 文档更新

### 1. 修正 [docs/rfc-auto-memory.md](docs/rfc-auto-memory.md)

现有 RFC 与实现之间存在多处偏差，需要将 RFC 更新为反映实际实现 + 本次改进：

**Scoring 公式修正**（第 120-126 行）：

- RFC 写的是 `type_weight x recency_weight x match_score`，其中 `recency_weight` 按 pipeline run 序号衰减
- 实际实现是 `type_weight x layer_weight x match_score x stale_penalty`，用 layer 区分而非 run 序号
- 更新 RFC 为实际公式，并说明 layer_weight 是 recency_weight 的简化替代（与 `.cursor/plans/` 中的实现计划一致）

**新增 "Context-Aware Injection" 小节**：

- 描述 `rank_memories` 应接收当前 skill 的 tags 和变更文件作为 context
- 说明 `prompt.rs` 中 `load_ranked_memories` 需要接受 skill name 和 run id 参数来构建 context_tags / context_files

**Open Questions 更新**：

- 标记已解决的问题（ID 格式 = 自增数字；gc 在 prompt 时自动触发过期清理；记忆在 prompt.rs 中独立组装而非传入 assemble_input_context）
- 新增：context_tags 的来源策略——从 skill.yaml 的 inputs 提取 tags，还是从当前 run 的文档 tags 提取

**Implementation Plan 更新**：标记已完成的 Phase，新增 Phase 6 "Context-Aware Scoring"

### 2. 新建 [docs/rfc-document-index.md](docs/rfc-document-index.md)

按 Popsicle RFC 模板结构编写，核心内容：

**Summary**：为 Popsicle 引入文档语义索引层，通过在 documents 表扩展 summary + tags 列、利用 SQLite FTS5 全文搜索，实现跨 pipeline run 的 spec 文档检索。

**Motivation**：

- 当前 `build_input_context` 只查 `WHERE pipeline_run_id = ?`，无法跨 run
- 新需求/变更需求时无法找到历史相关 spec
- 举例：SSO 需求需要参考历史认证 RFC，但 DAG 拓扑无法跨 run 关联

**Detailed Design**：

1. **数据模型扩展** — documents 表新增 `summary TEXT` + `tags TEXT`（JSON 数组）
2. **摘要/标签生成** — 文档进入 final 状态时（`on_complete` hook），调用 `popsicle doc summarize` 命令生成；也支持手动触发 `popsicle doc summarize <doc-id>`
3. **FTS5 索引** — 在 summary + tags + title 上建 FTS5 虚拟表
4. **检索漏斗** — 结构化过滤 -> FTS5 BM25 搜索 -> 时间衰减排序
5. **CLI 入口** — `popsicle context search <query>` 返回文档索引列表
6. **Prompt 集成** — `popsicle prompt --run <id>` 新增 `--related` flag 自动注入跨 run 相关文档的摘要索引

**Interface Changes**：

- `popsicle doc summarize [doc-id]` — 为文档生成/更新 summary + tags
- `popsicle context search <query>` — 跨 run 文档搜索
- `popsicle prompt --related` — 注入历史相关文档索引

**Rationale**：引用本次讨论的分析——结构化元数据 + FTS5 + LLM 标签对 60-400 个文档规模足够，不需要 embedding

---

## 第二部分：实现

### Phase 1 (P0): 修复 Memory Scoring 上下文传递

**目标**：让 `rank_memories` 的 tag/file 匹配真正生效

修改 [crates/popsicle-cli/src/commands/prompt.rs](crates/popsicle-cli/src/commands/prompt.rs)：

- `load_ranked_memories()` 签名改为 `load_ranked_memories(skill_name: &str, run_id: Option<&str>)` 
- 从 skill.yaml 的 inputs 中提取 `from_skill` 名称和 `artifact_type` 作为 context_tags
- 如果有 run_id，从 IndexDb 查询该 run 的文档，收集所有文档的 skill_name 作为额外 tags
- 将收集到的 tags 和 files 传入 `rank_memories(&memories, &context_tags, &context_files, limit)`
- 更新 `execute()` 中的调用点

### Phase 2 (P1): Documents 表扩展 + 摘要/标签生成

**2a. SQLite schema 迁移**

修改 [crates/popsicle-core/src/storage/index.rs](crates/popsicle-core/src/storage/index.rs)：

- `migrate()` 中新增 ALTER TABLE 迁移：`documents` 表加 `summary TEXT DEFAULT ''` 和 `tags TEXT DEFAULT '[]'`
- `upsert_document()` 更新为写入 summary + tags
- `query_documents()` 更新为读取 summary + tags
- `DocumentRow` 新增 `summary` + `tags` 字段

修改 [crates/popsicle-core/src/model/document.rs](crates/popsicle-core/src/model/document.rs)（如有需要）：

- `Document` 模型确认已有 tags 字段；新增 `summary` 字段

**2b. `popsicle doc summarize` 命令**

新增逻辑到 [crates/popsicle-cli/src/commands/doc.rs](crates/popsicle-cli/src/commands/doc.rs)：

- 新增 `Summarize` 子命令，接受可选 `doc-id`（无参数则处理当前 run 所有未摘要文档）
- 利用现有 `engine::markdown::extract_summary()` 生成 summary
- tags 提取策略：从文档 H2 标题 + frontmatter 中提取关键词（纯规则，不依赖 LLM）；后续可扩展为 LLM 提取
- 将 summary + tags 写回 IndexDb

**2c. 文档 approve 时自动触发**

修改 [crates/popsicle-cli/src/commands/doc.rs](crates/popsicle-cli/src/commands/doc.rs) 中 transition 逻辑：

- 当文档进入 final 状态（`is_final == true`）时，在触发 `on_complete` hook 之后，自动调用摘要/标签生成逻辑
- 将结果写入 IndexDb

### Phase 3 (P2): FTS5 全文搜索 + context search 命令

**3a. 启用 FTS5**

修改 [Cargo.toml](Cargo.toml)：

- rusqlite features 加 `"bundled-full"` 或确认 bundled 已包含 FTS5（rusqlite 0.32 的 bundled feature 默认包含 FTS5）

修改 [crates/popsicle-core/src/storage/index.rs](crates/popsicle-core/src/storage/index.rs)：

- `migrate()` 中新增 FTS5 虚拟表：`CREATE VIRTUAL TABLE IF NOT EXISTS documents_fts USING fts5(title, summary, tags, content=documents, content_rowid=rowid)`
- 新增触发器保持 FTS 索引与 documents 表同步
- 新增 `search_documents(query: &str, filters: SearchFilters) -> Vec<DocumentRow>` 方法

**3b. `popsicle context search` 命令**

修改 [crates/popsicle-cli/src/commands/context.rs](crates/popsicle-cli/src/commands/context.rs)：

- 新增 `Search(SearchArgs)` 子命令
- SearchArgs: `query: String`, `--status`, `--skill`, `--limit`（默认 10）
- 调用 IndexDb 的 FTS5 搜索，输出文档索引列表（id, title, summary, status, skill_name, run_id, file_path, relevance_score）

**3c. Prompt 集成 — 历史文档索引注入**

修改 [crates/popsicle-cli/src/commands/prompt.rs](crates/popsicle-cli/src/commands/prompt.rs)：

- `PromptArgs` 新增 `--related` flag
- 当 `--related` 启用时，用当前 skill 的 tags + 当前 run 的 title 作为查询词，调用 FTS5 搜索跨 run 文档
- 过滤掉当前 run 的文档，只保留 approved/accepted 状态的
- 将结果格式化为 "Historical References" 区块（仅摘要 + 路径），插入到 Memories 和 Input Context 之间
- 更新 `build_full_prompt` 接受新的 historical_refs 参数

### Phase 4: 测试

- Phase 1: 测试 `rank_memories` 传入 context_tags/files 后排序正确性
- Phase 2: 测试 documents 表迁移、summarize 命令、approve 自动摘要
- Phase 3: 测试 FTS5 搜索准确性、context search 命令输出、prompt --related 注入格式

