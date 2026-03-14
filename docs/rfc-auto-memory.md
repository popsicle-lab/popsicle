## Summary

本 RFC 提出为 Popsicle 引入 Auto-Memory 系统：通过 `popsicle memory` CLI 命令族管理项目级记忆（bug、决策点、pattern），Agent Skill 负责触发写入时机，`popsicle prompt` 在生成 prompt 时按相关性自动注入记忆。采用两层记忆模型（短期 + 长期），单 Markdown 文件存储，200 行硬上限。

## Motivation

### 当前痛点

1. **跨会话失忆**：Agent 在新会话中无法访问前序会话的 bug 修复经验和设计决策，导致重复犯错或做出矛盾决策
2. **经验不可传递**：一个 Agent（如 Cursor）积累的经验无法被另一个 Agent（如 Claude Code）复用
3. **现有机制的空白**：`project-context.md` 提供技术画像，上游文档提供 pipeline 上下文，但"操作性经验"（bug pattern、gotcha、trade-off 决策）没有持久化载体

### 与现有机制的关系

| 机制 | 职责 | 粒度 | 生命周期 |
|------|------|------|---------|
| ADR | 正式架构决策记录，给人看 | 完整文档 | 永久 |
| Bug Tracker Skill | 测试发现的 bug 追踪 | 完整文档，属于 pipeline run | Pipeline run 内 |
| project-context.md | 项目技术画像 | 项目级 | 手动更新 |
| **Auto-Memory** | **Agent 操作性经验，给 AI 看** | **1-5 行/条** | **自动生命周期** |

Auto-Memory 不替代 ADR 或 Bug Tracker，而是填补"轻量、跨会话、自动生命周期"的空白。一个 ADR 的核心决策可被提炼为一条 decision memory；一次 bug 修复的 root cause 可被提炼为一条 bug memory。

## Proposal

### Detailed Design

#### 1. 记忆数据模型

单条记忆由 Markdown 的 H3 块表示，结构化元数据嵌入行内：

```markdown
### [BUG] Context injection 排序不确定性
- **Created**: 2026-03-10 | **Layer**: long-term | **Refs**: 3
- **Tags**: context-injection, sorting | **Files**: engine/context.rs
- assemble_input_context 使用 HashMap 迭代导致排序不确定，
  high relevance 文档可能被排在中间。修复：改用 BTreeMap。
```

字段说明：

| 字段 | 必填 | 说明 |
|------|------|------|
| type | 是 | `BUG` / `DECISION` / `PATTERN` / `GOTCHA` |
| summary | 是 | H3 标题中的一行描述 |
| Created | 是 | 创建日期 |
| Layer | 是 | `short-term` / `long-term` |
| Refs | 是 | 被注入且被 Agent 引用的累计次数 |
| Tags | 否 | 用于检索匹配的标签 |
| Files | 否 | 关联的代码文件路径（相对路径） |
| Run | 否 | 关联的 pipeline run ID |
| detail | 是 | 1-5 行自然语言描述，包含 root cause、fix、trade-off 等 |

#### 2. 存储格式

单文件 `.popsicle/memories.md`，200 行硬上限：

```markdown
# Project Memories

## Long-term

### [PATTERN] Rust serde 默认值陷阱
- **Created**: 2026-03-01 | **Layer**: long-term | **Refs**: 4
- **Tags**: serde, yaml, backward-compat | **Files**: model/skill.rs
- 新增 YAML 字段必须用 #[serde(default)]，否则旧文件反序列化失败。
  已在 SkillInput.relevance 和 SkillInput.sections 中踩过。

### [DECISION] prompt 组装顺序：上下文在前指令在后
- **Created**: 2026-03-13 | **Layer**: long-term | **Refs**: 5
- **Tags**: prompt, attention, context-injection
- 基于 LLM attention U 型分布，high relevance 上下文紧邻 prompt 指令，
  共同占据 attention 最强的末尾区域。

---

## Short-term

### [BUG] extract_summary 对无 H2 文档返回空
- **Created**: 2026-03-14 | **Layer**: short-term | **Run**: ghi789
- **Tags**: markdown, summary | **Files**: engine/markdown.rs
- extract_summary 在文档没有 H2 时返回空字符串，应 fallback 到前 N 行。
```

选择单文件而非目录（每条记忆一个文件）的理由：
- Git diff 更直观——一次 PR 能看到所有记忆变更
- Agent 一次 read 就能获取全部记忆上下文
- 200 行上限天然约束了文件大小

#### 3. 生命周期机制

```
Agent 触发写入 → 短期记忆 → 评估 → 提升为长期 / 遗忘
                                ↓
                        同类合并为 pattern
```

**写入**：Agent 在会话中通过 CLI 命令写入短期记忆。

**提升条件**（短期 → 长期）：
- Refs >= 2（被注入且被 Agent 在产出中引用 2 次以上）
- 或 `popsicle memory promote <id>` 手动提升

**遗忘条件**：
- 短期记忆：超过 3 个 pipeline run 未被引用 → 自动删除
- 长期记忆：关联 Files 的 git diff 行数超过 50% → 标记 `[STALE]`，下次注入时提示 Agent 确认有效性；连续 2 次被确认无效 → 删除
- 长期记忆不自动删除，仅标记 stale

**合并**：当同 type + 相似 tags 的记忆超过 3 条时，Agent 应合并为一条 PATTERN 记忆，释放行数配额。

**容量管理**：当文件超过 200 行时，`popsicle memory save` 拒绝写入并提示：
- 清理 stale 记忆（`popsicle memory gc`）
- 合并同类记忆为 pattern
- 手动删除低价值记忆

#### 4. 检索与注入

`popsicle prompt --run <id>` 时自动注入相关记忆：

```
注入优先级 = type_weight × recency_weight × match_score
```

- **type_weight**：PATTERN=1.0, BUG=0.8, DECISION=0.6, GOTCHA=0.5
- **recency_weight**：最近 3 个 run 的记忆 1.0，3-10 个 run 0.5，更早 0.2
- **match_score**：基于 tags 交集和 Files 与当前变更文件的重叠度

注入上限：top-10 条，仅注入 summary 行（非 detail），以 `[Project Memories]` 块形式放在 prompt 最前部（low relevance 位置）：

```
[Project Memories — 以下是项目积累的经验，请在工作中注意避免已知问题]
- [BUG] Context injection 排序不确定性 → 改用 BTreeMap
- [PATTERN] Rust serde 默认值：新增 YAML 字段必须 #[serde(default)]
- [DECISION] prompt 组装顺序：上下文在前，指令在后
```

当 Agent 需要某条记忆的完整 detail 时，可调用 `popsicle memory show <id>`。

### Interface Changes

#### CLI 命令族

```bash
# 写入记忆
popsicle memory save --type bug --tags "serde,yaml" --files "model/skill.rs" \
  --summary "新增字段缺少 serde(default) 导致旧文件解析失败" \
  --detail "SkillInput 新增 relevance 字段时未加默认值标注..."

# 列出记忆
popsicle memory list                          # 全部
popsicle memory list --layer long-term        # 仅长期
popsicle memory list --type bug               # 仅 bug 类型
popsicle memory list --format json            # JSON 输出

# 查看单条
popsicle memory show <id>

# 生命周期管理
popsicle memory promote <id>                  # 短期 → 长期
popsicle memory stale <id>                    # 标记为 stale
popsicle memory gc                            # 清理已确认无效的 stale 记忆
popsicle memory delete <id>                   # 手动删除

# 诊断
popsicle memory stats                         # 行数使用情况、各类型分布
```

所有命令支持 `--format json`，供 Agent 程序化消费。

#### Agent Skill 触发规则

Agent Skill（Cursor `.mdc` / Claude Code `CLAUDE.md`）中定义触发指导：

```markdown
## 记忆写入时机

在以下场景结束时，调用 popsicle memory save：

1. **修复了一个 bug**：记录 root cause 和 fix（type=bug）
2. **做了技术选型决策**：记录 alternatives 和 rationale（type=decision）
3. **发现了一个 gotcha/陷阱**：记录触发条件和规避方式（type=gotcha）
4. **同类记忆 >= 3 条**：合并为 pattern（type=pattern）

## 记忆写入原则

- summary 必须是一行可独立理解的描述
- detail 不超过 5 行
- tags 使用小写、连字符分隔
- files 使用相对于项目根目录的路径
```

#### prompt 注入集成

在 `assemble_input_context` 中新增记忆源，作为最低 relevance 的上下文注入：

```rust
// engine/context.rs 中扩展
pub struct ContextInput {
    // ... 现有字段
}

pub struct MemoryContext {
    pub memories: Vec<MemorySummary>,  // top-N 条匹配的记忆摘要
}

pub fn assemble_input_context(
    inputs: Vec<ContextInput>,
    memories: Option<MemoryContext>,  // 新增参数
) -> AssembledContext
```

## Rationale and Alternatives

### Why This Approach

路径 C（CLI 存储 + Agent 触发）结合了两方面优势：

1. **CLI 存储**保证跨 Agent 一致性——Cursor 写入的记忆 Claude Code 也能读
2. **Agent 触发**避免在 CLI 中做自然语言理解——Agent 天然擅长判断"这值得记住"
3. **单文件 Markdown**与 Popsicle 的 Hybrid Storage 哲学一致，零外部依赖
4. **200 行上限**是主动的设计约束，防止记忆系统退化为"另一个日志"

### Alternative A：纯 Agent 层（Cursor Rules / CLAUDE.md）

将记忆存储为 `.cursor/rules/memories.mdc` 或追加到 `CLAUDE.md`。

- **Pros**：零实现成本，Agent 原生支持自动加载
- **Cons**：记忆被锁定在单个 Agent 生态中；无结构化生命周期管理；无法通过 `popsicle prompt` 统一注入；与 Popsicle 的 context injection 系统脱节

不选择的原因：违反第 6 节 "Multi-Agent Native" 原则。

### Alternative B：Popsicle 原生全自动

在 CLI 引擎中内置自然语言分类器，自动从 Agent 对话/产出中提取记忆。

- **Pros**：零用户干预，完全自动化
- **Cons**：需要在 CLI 中引入 LLM 调用依赖；分类准确性难以保证——一条错误的 bug 记忆可能误导后续所有会话；增加 CLI 的运行时复杂度

不选择的原因：CLI 应保持轻量确定性，"理解语义"的工作应留给 Agent。

### Alternative C：向量检索 + 数据库存储

使用 SQLite/Qdrant 存储记忆，embedding 做语义检索。

- **Pros**：检索精度更高，可支持大量记忆
- **Cons**：引入 embedding 模型依赖；200 行上限下规则匹配已足够；破坏 Popsicle 纯 CLI 的轻量性

不选择的原因：过度设计。当记忆量增长到规则匹配不够用时再考虑。

### Cost of Inaction

不实现记忆系统意味着：
- Agent 在每次新会话中从零开始，重复犯相同的 bug
- 项目经验只存在于人的记忆中，无法被 AI 复用
- 多 Agent 之间无法共享操作性经验

## Open Questions

- Questions to resolve during the RFC process:
  - 记忆的 ID 应该用什么格式？自增数字（简单）还是基于内容的 slug（可读）？
  - `popsicle memory gc` 是否应该在 `popsicle prompt` 时自动触发，还是必须手动运行？
  - 记忆注入应该在 `assemble_input_context` 内部处理，还是作为独立的 context 源在 `prompt.rs` 中组装？

- Questions to resolve during implementation:
  - Refs 计数如何追踪——Agent 如何报告"我引用了这条记忆"？需要 `popsicle memory ref <id>` 命令吗？
  - stale 检测的 git diff 阈值（50%）是否合理，需要实验调优
  - 200 行上限是否足够？需要在实际使用中验证

## Implementation Plan

- [ ] Phase 1 — 存储层：`memories.md` 文件格式解析/序列化，`Memory` 数据模型
- [ ] Phase 2 — CLI 命令：`save`、`list`、`show`、`delete`、`promote`、`stale`、`gc`、`stats`
- [ ] Phase 3 — 注入集成：在 `popsicle prompt` 中注入匹配的记忆摘要
- [ ] Phase 4 — Agent Skill：生成 Cursor `.mdc` 和 Claude Code 的记忆触发指导
- [ ] Phase 5 — 生命周期自动化：短期遗忘、stale 检测、容量告警
