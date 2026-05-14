# RFC: Workflow-Only Core（把 popsicle 砍成纯工作流引擎）

**Status**: Proposed
**Author**: discussion 2026-05-14
**Scope**: 把 `popsicle-core` 从「工作流 + 文档」双职责拆成「纯工作流」+ 可选 `popsicle-doc-helpers` 辅助 crate

## Motivation

当前 popsicle 同时承担两件事：

1. **工作流引擎** —— Skill 状态机、Run/Spec/Issue/PipelineRun ledger、Hook 总线、CLI 编排
2. **文档管理** —— `Document` 实体、artifact 文件存储、Markdown 智能编辑、Guard 系统、上下文装配、WorkItem 提取

这导致两个问题：

- **耦合点过多**：任何模块（如 intent-coder）想自定义文档落点、自定义 Guard、自定义上下文规则，就要绕开或扩展 popsicle 核心
- **概念面积大**：上层工具仅想要"跑 Skill 状态机"时，被迫继承一整套文档语义

本 RFC 的核心主张：

> **popsicle 只管工作流，不管文档。**

文档相关能力**不删除**，而是搬到独立的可选 crate `popsicle-doc-helpers`，由调用方按需引入。

---

## 新边界

| 留在 `popsicle-core` | 搬到 `popsicle-doc-helpers` |
|---|---|
| Skill YAML 状态机解析 (`model/skill.rs`) | `Document` 实体 (`model/document.rs`) |
| Run / Spec / Namespace / PipelineRun ledger | artifact 文件存储 (`storage/file.rs::artifact_path`) |
| Hook 总线 (`engine/hooks.rs`) | Markdown 工具 (`engine/markdown.rs`) |
| Pipeline 编排 (`model/pipeline.rs`) | Guard 系统 (`engine/guard.rs`) |
| Tool / Memory 注册表 (`registry/`, `memory/`) | 上下文装配 (`engine/context.rs`, `context_layer.rs`) |
| Git 集成 (`commands/git.rs`) | WorkItem 提取 (`engine/extractor.rs`, `model/work_item.rs`) |
| CLI 主框架 | `popsicle doc/checklist/extract/migrate/prompt` 命令族 |

### popsicle-core 暴露给模块的 API（3 个原语）

```rust
// 1. 工作流事件总线
trait Hook {
    fn on_event(&self, event: HookEvent) -> Result<HookOutput>;
}
// HookEvent { run_id, skill, transition, env: HashMap<String,String> }

// 2. Run ledger 查询
fn list_runs(filter: RunFilter) -> Vec<Run>;
fn get_run(run_id: &str) -> Option<Run>;

// 3. 内容解析（替代直接读 artifact 文件）
trait ContentProvider {
    fn resolve(&self, ref_: &ContentRef) -> Result<String>;
}
// ContentRef 是模块定义的字符串，popsicle-core 不解释
```

**关键 trick**：上下文装配从「读 artifact 文件」改为「调用 ContentProvider」。popsicle-core 拿到的永远是 `String`，**不关心它是文件、git blob、数据库行、还是 LLM 生成**。

---

## 砍后会丢失的能力（必须显式承载）

下表是审计 `engine/` + `model/` + `commands/` 之后**真的会丢失**的能力，按"丢失代价"排序。

### 🟥 A. 高价值能力 —— 砍了立刻就疼

#### A1. 内置 Guard 系统 — `engine/guard.rs` 858 行

工作流闸门，**只对 Document 内容生效**：

| Guard | 作用 |
|---|---|
| `upstream_approved` | 上游 skill 的产出文档必须 Final |
| `has_sections:Summary,Decision` | 文档 body 必须含这些 H2 段、且段内非模板占位 |
| `checklist_complete[:Section]` | Markdown checkbox 全勾选 |
| 多守卫合取 `g1;g2;g3` | 全部通过才放行 |

**砍后代价**：每个模块自己写 checkbox 解析、section 提取、上游状态查询。等价 guard 至少要在 intent-coder 重写一遍。回归测试覆盖也得自己写。

#### A2. Markdown 智能编辑 — `engine/markdown.rs` 382 行

`extract_sections` / `extract_summary` / `extract_tags` / `upsert_section` / `count_checkboxes` / `is_template_placeholder` / `extract_section_content`。

**砍后代价**：模块自己写 markdown parser，或引入 `pulldown-cmark` 自己包一层。每个新模块重复一次。

#### A3. 上下文装配 — `engine/context.rs` 244 行 + `context_layer.rs` 236 行 ⭐ 最痛

`assemble_input_context` 按 Relevance（Low/Med/High）排序上游文档，截取 summary 或全文，注入 LLM prompt。`ProjectContextLayer` / `MemoriesLayer` / `HistoricalRefsLayer` / `UpstreamDocsLayer` 可插拔。

**砍后代价**：这是 IDD 流水线的**核心 LLM 友好能力**。如果纯工作流引擎不读文档，"上游 PDR 自动注入下游 prompt"就没了。下游每个模块都要重新发明一套 ContentProvider + ranking。

#### A4. WorkItem 提取 — `engine/extractor.rs` 365 行

`extract_user_stories` / `extract_test_cases` / `extract_bugs` —— 从 PRD/RFC 文档自动析出工作项写进 SQLite，让 `popsicle item list` 能查。

**砍后代价**：bug/story/testcase 索引消失。intent-coder 想列出所有验收标准要自己扫文件。

#### A5. `popsicle prompt` 命令 — `commands/prompt.rs` 655 行

为 AI agent 装配「完整上下文 + skill guide + 模板」的单一 prompt 字符串。是 Claude Code / Cursor 调 popsicle 时**唯一的入口**。

**砍后代价**：AI agent 接入面要重设计——要么每个模块各写一套 prompt 组装，要么社区有人写共享层。

---

### 🟧 B. 中等价值 —— 砍了能补，但要花工

| 能力 | 行数 | 备注 |
|---|---|---|
| `popsicle doc` 命令族 | 596 | doc list/show/transition/revise/approve/replay |
| `popsicle checklist` | 447 | 跨文档汇总待办事项 |
| Spec 锁 / Issue / PipelineRun 联动 | —— | PipelineRun 仍跑，但产物去向归模块；Spec 锁仍可用（锁的是 run） |
| `commands/extract.rs` | 143 | WorkItem 提取的 CLI 触发器（依赖 A4） |
| `popsicle migrate` | 143 | 文档 schema 迁移工具，砍 doc 后自然消失 |

---

### 🟨 C. 低价值 —— 顺手就能复制

- Frontmatter 解析、artifact 路径计算、模板渲染（~50 行小工具）
- `engine/advisor.rs` next_steps 推荐（基于文档状态推下一步，砍后改为「基于 Run 状态推」）

---

### 🟩 D. 不丢 —— 与文档无关

Skill YAML 状态机、Pipeline 编排、Run/Spec/Namespace ledger、Hook 总线、Tool/Memory 注册表、Git 集成。

---

## 三种现实的折中

| 方案 | 描述 | 代价 |
|---|---|---|
| **D1 全砍** | popsicle 只剩 workflow + run + hook，A1–A5 全部丢弃 | 上下文装配等能力散落各模块，长期 N 套不兼容实现 |
| **D2 半砍（推荐）** | 砍 Document 实体、artifact 文件存储；A1–A5 搬到可选 crate `popsicle-doc-helpers`（feature flag） | 模块按需引入；popsicle-core 保持纯净；零能力丢失 |
| **D3 不砍** | 现状 | popsicle 与文档紧耦合 |

---

## 推荐：方案 D2 落地步骤

| 顺序 | 任务 | 破坏性 |
|---|---|---|
| 1 | 在 `popsicle-core` 加 `ContentProvider` trait + 默认文件实现，让现状不变 | 否 |
| 2 | 新建 crate `popsicle-doc-helpers`，把 `engine/guard.rs` / `markdown.rs` / `context.rs` / `extractor.rs` / `model/document.rs` 搬过去 | 否（重导出） |
| 3 | `engine::context` 改为通过 ContentProvider 取内容 | 否（旧 provider 兼容） |
| 4 | intent-coder 模块实现自己的 `ContentProvider`，pdr/adr 从 `decisions/` 读 | 否 |
| 5 | 标记 `popsicle-core` 重导出的 `Document` / `artifact_path` / doc 命令为 `@deprecated` | 否 |
| 6 | 一个发布周期之后，从 `popsicle-core` 删除重导出，只保留 `popsicle-doc-helpers` | 是 |

**步骤 1–5 非破坏性**，旧模块仍能用默认 provider 工作。**步骤 6 才是真切割**。可分两个 PR：API surface 加 → 半年宽限 → 切除旧实现。

---

## 与 intent-coder dogfood 的关系

intent-coder（[github.com/narwal/intent-coder](https://github.com/)）正在 dogfood popsicle。其文档体系按 L0–L6 分层：

- **L0** `docs/CHARTER.md` —— 四条铁律
- **L1** invariants（机器可验证）
- **L2** `PRODUCT.md` + `tasks/` + `acceptance.intent`
- **L3** `decisions/{pdr,adr}/` —— append-only
- **L4** `ARCHITECTURE.md` + `contracts.intent`
- **L5** `migration/slices/`
- **L6** 代码

popsicle 核心**不应该知道** L0–L6、PDR、ADR、proposals 这些概念。所有「artifact → 仓库哪一层」的映射，由 intent-coder 模块自己实现的 promote 逻辑完成，通过读 `popsicle-core` 暴露的 `Run` ledger + Hook event 完成。

见 `intent-coder/example/saas-demo/docs/CHARTER.md` 的 "Layer Map" 段落两张图（写入流 + Skill ↔ 文档层映射），它们是本 RFC 对模块侧的可视化对照。

---

## 决策点

需要确认：

1. **走方案 D2 还是 D1？** D2 推荐（不丢能力）
2. **是否同意把 doc helpers 拆成独立 crate？**（vs feature flag in `popsicle-core`）
3. **deprecation 宽限期长度** —— 建议一个 minor release 周期

---

## 不在本 RFC 范围

- 具体的 `ContentProvider` trait API 详细签名（细节随实现迭代）
- intent-coder 的 promote 命令实现细节（属于 intent-coder 模块）
- 现有 `.popsicle/popsicle.db` schema 是否需要迁移（依赖最终决策）
