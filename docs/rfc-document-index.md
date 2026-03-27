## Summary

本 RFC 提出为 Popsicle 引入文档语义索引层（Document Index），通过在 SQLite `documents` 表扩展 `summary` + `tags` 列、利用 SQLite FTS5 全文搜索，实现跨 pipeline run 的 spec 文档检索。这是 Popsicle 三层记忆体系中的第 3 层，补充现有的 DAG 拓扑检索（第 1 层）和操作性经验记忆（第 2 层）。

## Motivation

### 当前痛点

1. **跨 run 文档不可见**：`popsicle prompt --run <id>` 的 `build_input_context` 只查 `WHERE pipeline_run_id = ?`，硬编码了单 run 范围。新需求无法自动关联历史 spec。

2. **新需求缺乏历史参考**：当开始一个新 pipeline run（如"SSO 单点登录"），Agent 无法知道项目中已有"用户认证"和"权限管理"的 RFC/ADR，可能做出与历史设计矛盾的决策。

3. **变更影响不可追踪**：当需要变更某个技术方案（如"把 Session 从 Redis 改为 JWT 无状态"），无法自动找到所有受影响的历史 spec 文档。

### 与现有机制的关系

| 层 | 机制 | 范围 | 检索方式 | 精度 |
|----|------|------|---------|------|
| 第 1 层 | DAG 拓扑 | 当前 run 内 | `skill.yaml` inputs + `depends_on` | 100%（确定性） |
| 第 2 层 | Auto-Memory | 跨 run 经验 | tag/file 规则匹配 | 中等 |
| **第 3 层** | **Document Index** | **全项目历史 spec** | **结构化过滤 + FTS5** | **高** |

第 3 层不替代第 1 层（run 内依赖仍走 DAG），而是补充跨 run 的文档发现能力。

### 为什么不用 Embedding / 向量检索

经过分析（详见 Rationale），对于 Popsicle 的文档规模（60-400 个结构化 spec 文档），结构化元数据 + FTS5 全文搜索已经足够：

- 每个文档已有丰富的结构化元数据（skill_name, doc_type, status, pipeline_run_id）
- 语义鸿沟的主要来源是缺少摘要和标签，而非缺少向量表示
- 引入 embedding 模型会打破 Popsicle 的轻量 CLI 哲学（增加 ~80MB 模型依赖）
- 不同 embedding 模型的向量不互认，换模型需全量重建索引
- FTS5 保留了向 embedding 升级的路径（未来可在 summary 列上加向量列）

## Proposal

### Detailed Design

#### 1. 数据模型扩展

在 SQLite `documents` 表新增两列：

```sql
ALTER TABLE documents ADD COLUMN summary TEXT DEFAULT '';
ALTER TABLE documents ADD COLUMN doc_tags TEXT DEFAULT '[]';
```

- **summary**：3-5 行自然语言摘要，由 `engine::markdown::extract_summary()` 生成
- **doc_tags**：JSON 数组格式的语义标签，从文档 H2 标题和 frontmatter 中提取

`DocumentRow` 和 `Document` 模型同步扩展。

#### 2. 摘要/标签生成

摘要和标签生成采用**两级策略**：规则提取作为即时 fallback，LLM 生成作为高质量覆盖。

##### 2a. 规则提取（自动 fallback）

**触发时机**：文档进入 final 状态时自动触发（在 `doc transition` 的 `is_final` 分支中）。

**摘要生成**：复用现有 `engine::markdown::extract_summary()`，提取首段 + H2 目录。

**标签提取**（纯规则，不依赖 LLM）：
1. 文档的 `skill_name`（如 `rfc-writer`）
2. 文档的 `doc_type`（如 `rfc`）
3. 所有 H2 标题的关键词（去除常见停用词）
4. frontmatter 中的 tags 字段（如果有）

##### 2b. Agent 驱动的 LLM 生成（高质量覆盖）

Popsicle 遵循**依赖反转**原则：不直接调用 LLM API，而是提供工具让 Agent（Cursor / Claude Code）驱动 LLM 生成。

**设计理念**：
- Popsicle 是纯 CLI 工具，不引入 HTTP client 或 LLM SDK 依赖
- Agent 已经拥有 LLM 调用能力，Popsicle 只需提供 prompt 生成和结果写入接口
- 规则提取保证即使 Agent 不调用 LLM，文档也有基本的索引能力

**工作流程**（approve 后立即执行）：

```
doc approve
    │
    ├─ 自动执行规则提取 → 写入 summary + tags（fallback）
    │
    └─ Agent 执行 LLM 增强流程：
         │
         ├─ popsicle doc summarize <id> --generate-prompt
         │   → 输出包含文档内容的结构化 prompt
         │
         ├─ Agent 将 prompt 发送给 LLM，获取 JSON 响应
         │
         └─ popsicle doc summarize <id> --summary "..." --tags "a,b,c"
             → 覆盖规则提取的结果
```

**CLI 接口**：

```bash
# Mode 1: 规则提取（默认，批量）
popsicle doc summarize              # 当前 run 所有未摘要文档
popsicle doc summarize <doc-id>     # 指定文档

# Mode 2: 生成 LLM prompt
popsicle doc summarize <doc-id> --generate-prompt
popsicle doc summarize <doc-id> --generate-prompt --format json

# Mode 3: 直接写入 LLM 结果
popsicle doc summarize <doc-id> --summary "LLM 生成的摘要" --tags "tag1,tag2,tag3"
```

**`--generate-prompt` 输出格式**（JSON）：

```json
{
  "doc_id": "abc-123",
  "title": "JWT 用户认证方案",
  "prompt": "You are analyzing a technical document...\n\nDocument content:\n---\n...\n---\n\nPlease provide:\n1. A concise summary...\n2. A list of semantic tags...\n\nRespond in JSON format:\n{\"summary\": \"...\", \"tags\": [...]}"
}
```

#### 3. FTS5 全文搜索索引

```sql
CREATE VIRTUAL TABLE IF NOT EXISTS documents_fts USING fts5(
    title,
    summary,
    doc_tags,
    content=documents,
    content_rowid=rowid
);
```

通过 SQLite 触发器保持 FTS 索引与 documents 表同步：

```sql
CREATE TRIGGER IF NOT EXISTS documents_ai AFTER INSERT ON documents BEGIN
    INSERT INTO documents_fts(rowid, title, summary, doc_tags)
    VALUES (new.rowid, new.title, new.summary, new.doc_tags);
END;

CREATE TRIGGER IF NOT EXISTS documents_au AFTER UPDATE ON documents BEGIN
    INSERT INTO documents_fts(documents_fts, rowid, title, summary, doc_tags)
    VALUES ('delete', old.rowid, old.title, old.summary, old.doc_tags);
    INSERT INTO documents_fts(rowid, title, summary, doc_tags)
    VALUES (new.rowid, new.title, new.summary, new.doc_tags);
END;

CREATE TRIGGER IF NOT EXISTS documents_ad AFTER DELETE ON documents BEGIN
    INSERT INTO documents_fts(documents_fts, rowid, title, summary, doc_tags)
    VALUES ('delete', old.rowid, old.title, old.summary, old.doc_tags);
END;
```

#### 4. 检索漏斗

```
查询输入
    │
    ▼
┌─── 第 1 步：结构化过滤 ──────────────────────────┐
│  WHERE status IN ('approved', 'accepted')        │
│  AND pipeline_run_id != 当前 run（排除当前 run）   │
│  可选：AND skill_name IN (...)                    │
└──────────────────────┬───────────────────────────┘
                       │
                       ▼
┌─── 第 2 步：FTS5 BM25 搜索 ──────────────────────┐
│  documents_fts MATCH ?                            │
│  ORDER BY bm25(documents_fts)                     │
│  LIMIT ?                                          │
└──────────────────────┬───────────────────────────┘
                       │
                       ▼
  返回文档索引列表：
  (id, title, summary, status, skill_name, run_id, file_path, bm25_score)
```

#### 5. Prompt 集成

`popsicle prompt --run <id> --related` 启用跨 run 相关文档注入：

1. 用当前 skill 的 tags + 当前 run 的 title 构建 FTS5 查询
2. 过滤掉当前 run 的文档
3. 将结果格式化为 "Historical References" 区块（仅摘要 + 路径）
4. 插入到 Memories 和 Input Context 之间

```
Prompt 组装顺序：
  1. Project Context (background)
  2. Project Memories
  3. Historical References (新增，仅摘要+路径)   ← 第 3 层
  4. Input Context (upstream docs)                ← 第 1 层
  5. Prompt 指令
```

注入格式：

```markdown
## Historical References (from previous runs)

以下是项目中可能相关的历史设计文档，如需详细内容请读取对应文件：

- **[RFC] JWT 用户认证方案** (approved) — .popsicle/artifacts/run-1/auth-rfc.md
  用户认证模块的技术方案，包含登录、注册、token 刷新三个核心 API。

- **[ADR] 选择 Redis 存 Session** (approved) — .popsicle/artifacts/run-1/session-adr.md
  选择 Redis 作为 Session 存储的决策记录。
```

### Interface Changes

#### 新增 CLI 命令

```bash
# 为文档生成/更新摘要和标签（规则提取）
popsicle doc summarize              # 当前 run 所有未摘要文档
popsicle doc summarize <doc-id>     # 指定文档

# Agent 驱动 LLM 生成
popsicle doc summarize <doc-id> --generate-prompt              # 输出 LLM prompt（text）
popsicle doc summarize <doc-id> --generate-prompt --format json # 输出 LLM prompt（JSON）
popsicle doc summarize <doc-id> --summary "..." --tags "a,b,c" # 写入 LLM 结果

# 跨 run 文档搜索
popsicle context search <query>                    # 全文搜索
popsicle context search <query> --status approved  # 按状态过滤
popsicle context search <query> --skill rfc-writer # 按 skill 过滤
popsicle context search <query> --limit 5          # 限制结果数

# prompt 注入历史相关文档
popsicle prompt <skill> --run <id> --related       # 启用跨 run 文档注入
```

#### 修改的 CLI 命令

`popsicle prompt` JSON 输出新增 `historical_refs` 字段：

```json
{
  "skill": "implementation",
  "historical_refs": [
    {
      "id": "doc-abc",
      "title": "JWT 用户认证方案",
      "doc_type": "rfc",
      "status": "approved",
      "summary": "用户认证模块的技术方案...",
      "file_path": ".popsicle/artifacts/run-1/auth-rfc.md",
      "bm25_score": 2.34
    }
  ]
}
```

#### SQLite Schema 变更

`documents` 表新增 `summary` 和 `doc_tags` 列（通过 ALTER TABLE 迁移，向后兼容）。新增 `documents_fts` FTS5 虚拟表和同步触发器。

## Rationale and Alternatives

### Why This Approach

1. **利用已有结构**：Popsicle 的 spec 文档从诞生之日起就是结构化的——有 skill_name、doc_type、status、DAG 依赖。这些结构化信号的信息量远超 embedding 向量。

2. **FTS5 对文档规模足够**：60-400 个文档，每个有摘要 + 标签，BM25 全文搜索的召回率已经足够。SQLite FTS5 是零外部依赖的成熟方案。

3. **保留升级路径**：如果未来文档量增长到 1000+，可以在 summary 列上加 embedding 向量列，而不需要重构数据模型。

4. **与 Popsicle 哲学一致**：纯 CLI、零外部依赖、确定性行为。不引入 embedding 模型、不需要网络调用。

### Alternative A：Embedding + 向量搜索

使用本地 embedding 模型（如 all-MiniLM-L6-v2）对文档摘要做向量化，余弦相似度搜索。

- **Pros**：语义检索精度最高
- **Cons**：增加 ~80MB 模型依赖；引入 ONNX Runtime 编译复杂度；不同模型向量不互认，换模型需全量重建索引；跨平台浮点精度可能有差异

不选择的原因：对当前文档规模过度设计。作为未来升级路径保留。

### Alternative B：LLM 驱动的推理式检索（PageIndex 方案）

构建文档层级树索引，每次检索时让 LLM 在树上推理导航。

- **Pros**：检索准确度极高（FinanceBench 98.7%）
- **Cons**：每次检索需要 LLM 调用，与 Popsicle 轻量 CLI 哲学冲突；面向单个长 PDF 而非多个短文档

不选择的原因：Popsicle 的文档天然有结构（pipeline/run/skill），不需要 LLM 来"发现"结构。

### Alternative C：纯关键词匹配（不引入 FTS5）

仅用 SQLite LIKE 查询 title + tags。

- **Pros**：实现最简单
- **Cons**：无法处理词形变化和多词匹配；无 BM25 排序；查询体验差

不选择的原因：FTS5 是 SQLite 内置功能，启用成本极低但体验提升显著。

### Cost of Inaction

不实现文档索引意味着：
- 新需求启动时无法自动发现历史相关设计，Agent 可能做出与历史决策矛盾的方案
- 变更需求时无法追踪影响范围，遗漏需要同步更新的文档
- 项目知识随 run 数量增长而碎片化，跨 run 的设计一致性无法保证

## Open Questions

- FTS5 对中文分词的支持有限（默认 unicode61 tokenizer 按空格/标点分词），是否需要引入 `simple` tokenizer 或自定义分词？当前 spec 文档以英文为主，中文支持作为后续优化。
- ~~`popsicle doc summarize` 的标签提取质量是否足够？~~ **已解决**：采用两级策略——规则提取作为 fallback，Agent 驱动 LLM 生成作为高质量覆盖。通过 `--generate-prompt` 和 `--summary/--tags` 接口实现依赖反转。
- `--related` 的默认行为：是否应该在有 `--run` 时默认启用，还是必须显式指定？

## Implementation Plan

- [x] Phase 1 — Schema 迁移：documents 表加 summary + doc_tags 列
- [x] Phase 2a — 规则摘要/标签生成：`popsicle doc summarize` 命令 + approve 自动触发
- [x] Phase 2b — Agent 驱动 LLM 生成：`--generate-prompt` + `--summary/--tags` 接口
- [x] Phase 3 — FTS5 索引：虚拟表 + 同步触发器 + `search_documents` 方法
- [x] Phase 4 — CLI 入口：`popsicle context search` 命令
- [x] Phase 5 — Prompt 集成：`--related` flag + Historical References 注入
- [x] Phase 6 — Agent 模板更新：CLAUDE.md / Cursor rules 添加 LLM summarize 工作流说明
