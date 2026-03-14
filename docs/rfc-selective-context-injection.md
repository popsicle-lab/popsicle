## Summary

本 RFC 提出对 `popsicle prompt` 的上游文档注入机制进行改造：通过在 `SkillInput` 上声明 `relevance`（重要程度）和 `sections`（章节过滤）两个字段，实现基于 LLM attention 分布特性的选择性上下文注入，替代当前的全量平等注入策略。

## Motivation

当 Skill 通过 `popsicle prompt --run <id>` 注入上游文档时，当前实现存在三个问题：

1. **Attention 稀释** — 所有上游文档以 `inputs` 声明顺序平等拼接。对当前 Skill 最关键的文档可能排在中间位置，而 LLM 的 attention 分布呈 U 型（开头和末尾的内容被关注最多，中间容易被忽略），关键信息恰好落入低注意力区域。

2. **噪声干扰** — 低相关度的上游文档（如 `domain-analysis` 对 `implementation`）全文注入，占据上下文窗口空间但贡献有限，反而稀释了高价值信息的信号强度。

3. **信号丢失** — 以 `implementation` Skill 为例，它依赖 5 个上游输入（rfc、adr、priority-test-spec、api-test-spec、e2e-test-spec）。全量注入 5 份完整文档后，RFC 的核心技术方案（最关键信息）被淹没在大量 test-spec 细节中。

以 `full-sdlc` pipeline 的 `implementation` 阶段为例：当前会注入约 5 份完整文档，但实际上 RFC 和 ADR 是实现阶段的核心参考，test-spec 只需要知道覆盖哪些维度即可，不需要看到每个测试用例的完整描述。

## Proposal

### Detailed Design

#### 1. 数据模型扩展

在 `SkillInput` 上新增两个可选字段：

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Relevance {
    Low,
    Medium,
    High,  // 默认值
}

pub struct SkillInput {
    pub from_skill: String,
    pub artifact_type: String,
    pub required: bool,
    pub relevance: Relevance,                // 新增，默认 High
    pub sections: Option<Vec<String>>,       // 新增，可选
}
```

`Relevance` 默认为 `High`，保证对现有 `skill.yaml` 的完全向后兼容 — 未声明 `relevance` 的输入自动按最高优先级全文注入，行为与改造前一致。

#### 2. 注入策略

基于 LLM attention U 型分布优化内容提取和排序：

| Relevance | 注入位置 | 内容提取 | 典型场景 |
|-----------|---------|---------|---------|
| `low` | 最先注入（上下文块开头） | 仅标题 + 摘要（首段 + H2 目录） | `domain-analysis` 对 `prd` |
| `medium` | 中间位置 | 仅指定 `sections` 的 H2 章节 | `prd` 对 `rfc`（只需 Problem Statement） |
| `high` | 最后注入（紧邻 prompt 指令） | 全文 | `rfc` 对 `implementation` |

注入排序：Low → Medium → High，high 内容最靠近 prompt 指令文本。

#### 3. Prompt 组装顺序调整

将 prompt 组装从 "指令 → 上下文" 调整为 "上下文 → 指令"：

```
## Input Context (from upstream skills)

### [Background] domain-model — Domain Analysis [approved]
{摘要}

---

### [Reference] prd — Product Requirements [approved]
{选中的章节}

---

### [Primary] rfc — Technical RFC [accepted]
{全文}

---

{base_prompt 指令}
```

这样 high relevance 内容和 prompt 指令共同处于文本末尾（attention 最强区域），最大化 LLM 对关键信息的注意力。

#### 4. Markdown 工具函数

从 `guard.rs` 提取共享的 Markdown 解析逻辑到独立模块 `engine/markdown.rs`：

- `extract_section_content(after_header) -> String` — 提取 H2 到下一个 H2 之间的内容
- `extract_sections(body, section_names) -> String` — 按名称提取多个 H2 章节
- `extract_summary(body) -> String` — 提取摘要（首段 + H2 目录列表）
- `is_template_placeholder(content) -> bool` — 判断内容是否为未填写的模板占位符

`guard.rs` 的 `check_has_sections` 和 `check_checklist_complete` 改为调用此模块。

#### 5. 上下文组装引擎

新建 `engine/context.rs`，提供核心组装逻辑：

```rust
pub struct ContextInput {
    pub artifact_type: String,
    pub title: String,
    pub status: String,
    pub body: String,
    pub relevance: Relevance,
    pub sections: Option<Vec<String>>,
}

pub struct AssembledContext {
    pub parts: Vec<ContextPart>,
    pub full_text: String,
}

pub fn assemble_input_context(inputs: Vec<ContextInput>) -> AssembledContext
```

处理流程：
1. 按 `relevance` 排序（`Ord` 实现：Low < Medium < High）
2. 对每个 input 按 relevance 级别应用内容提取
3. 为每部分添加 relevance 标注（`[Background]`、`[Reference]`、`[Primary]`）
4. 用 `---` 分隔符拼接为完整文本

### Interface Changes

#### skill.yaml 格式

新增可选字段 `relevance` 和 `sections`：

```yaml
inputs:
  - from_skill: domain-analysis
    artifact_type: domain-model
    required: false
    relevance: low           # 新字段，默认 high
  - from_skill: prd
    artifact_type: prd
    required: true
    relevance: medium         # 新字段
    sections:                 # 新字段，medium 时指定要提取的 H2
      - Problem Statement
      - User Stories & Acceptance Criteria
  - from_skill: arch-debate
    artifact_type: arch-debate-record
    required: true
    # relevance: high — 默认值，可省略
```

#### popsicle prompt JSON 输出

新增 `context_parts` 字段，包含每个注入部分的 relevance 和提取后的内容：

```json
{
  "skill": "rfc",
  "state": "draft",
  "prompt": "...",
  "full_prompt": "...",
  "input_context": "...",
  "context_parts": [
    {
      "artifact_type": "prd",
      "title": "Product Requirements",
      "status": "approved",
      "relevance": "medium",
      "content": "## Problem Statement\n\n..."
    }
  ],
  "available_states": ["draft"]
}
```

#### popsicle skill show JSON 输出

`inputs` 数组的每个元素新增 `relevance` 和 `sections` 字段。

#### 组装顺序变更

`full_prompt` 的结构从 `{prompt}\n---\n{context}` 变为 `{context}\n---\n{prompt}`。所有通过 `popsicle prompt --run` 获取上下文的 Agent 会自动获得新的排序行为。

## Rationale and Alternatives

### Why This Approach

声明式的 `relevance` + `sections` 方案有三个核心优势：

1. **最小侵入性** — 仅扩展已有的 `SkillInput` 模型，不引入新的文件结构、工作流概念或配置层。`relevance` 默认为 `High`，现有 `skill.yaml` 无需修改即可正常工作。

2. **与 Popsicle 哲学一致** — Skill 声明"需要什么、什么最重要"，引擎决定"如何最优地呈现"。这延续了 Guard（声明式验证）和 Pipeline（声明式编排）的设计模式。

3. **可验证** — 注入策略的效果可以通过 `popsicle prompt --run <id> --format json` 的 `context_parts` 输出直接观察，不需要黑盒测试。

### Alternative A: Step-File 分步加载

参考 BMAD-METHOD 的 step-file 架构，将工作流拆分为多个步骤文件，每步只加载当前需要的上下文。

- 优点：彻底控制每步的上下文大小
- 缺点：引入新的文件结构和工作流概念；与 Popsicle 的单窗口工作模式冲突 — 前序步骤的对话已在上下文中，分步重新加载是冗余的；`popsicle prompt --run` 已解决跨会话恢复问题

不选择此方案，因为它解决的是"分步加载"问题，而 Popsicle 的真正问题是"注入优先级"。

### Alternative B: 动态 Relevance（运行时自动判断）

不在 `skill.yaml` 中静态声明 relevance，而是在注入时根据上游文档的内容、长度、与当前 Skill 的语义相似度动态计算重要程度。

- 优点：无需 Skill 作者手动配置；能适应文档内容的变化
- 缺点：引入 LLM 调用或嵌入模型的额外开销；判断结果不可预测、难以调试；与 Popsicle 声明式哲学冲突

作为未来扩展方向保留，但初始实现采用静态声明以保证确定性。

### Cost of Inaction

不实现此方案，`popsicle prompt` 会继续全量平等注入所有上游文档。随着 pipeline 复杂度增长（如 `implementation` 有 5 个上游输入），attention 稀释问题会愈发严重，LLM 产出质量会因上下文噪声而下降。

## Open Questions

- Questions to resolve during the RFC process:
  - `sections` 是否需要支持模式匹配（正则或 glob）以应对文档结构不完全一致的情况？当前实现要求精确匹配 H2 标题名。
  - 摘要生成（`low` relevance）的内容策略是否合理？当前实现提取首段 + H2 标题列表，是否需要支持配置摘要长度或摘要模板？

- Questions to resolve during implementation:
  - `popsicle context` 命令是否也应支持按 relevance 过滤输出？当前实现未修改 `context` 命令。
  - Desktop UI 的 Prompt 面板是否需要展示 relevance 标注？当前 UI 的 `get_prompt` Tauri command 未使用 `build_input_context`。

## Stakeholders

| Role | Team/Person | Concern |
|------|-------------|---------|
| Skill 作者 | 开发者 | 需要为 inputs 选择合理的 relevance 和 sections |
| Agent 集成 | Claude Code / Cursor | 消费 `popsicle prompt` 输出，需适应新的组装顺序 |
| 引擎维护者 | Core 团队 | markdown 解析逻辑的准确性和边界情况处理 |

## Implementation Plan

- [x] Phase 1: 模型层 — `Relevance` 枚举和 `SkillInput` 字段扩展
- [x] Phase 2: Markdown 工具模块 — 提取共享函数，重构 `guard.rs`
- [x] Phase 3: 上下文组装引擎 — `engine/context.rs`
- [x] Phase 4: Prompt 命令重构 — 使用新引擎，调整组装顺序
- [x] Phase 5: DTO 与 UI 更新 — `SkillInputInfo` 新字段
- [x] Phase 6: Skill YAML 更新 — 为 12 个 Skill 配置 relevance/sections
- [x] Phase 7: 测试 — 模型解析、markdown 工具、上下文组装、向后兼容（共 18 个新测试）
