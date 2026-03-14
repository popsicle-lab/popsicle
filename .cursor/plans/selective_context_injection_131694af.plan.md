---
name: Selective Context Injection
overview: 扩展 SkillInput 模型，支持 relevance（重要程度）和 sections（章节过滤），实现基于 LLM attention 分布优化的上游文档选择性注入策略。
todos:
  - id: model-relevance
    content: 在 skill.rs 中添加 Relevance 枚举和 SkillInput 新字段（relevance, sections），更新 mod.rs 导出
    status: completed
  - id: markdown-utils
    content: 新建 engine/markdown.rs，提取 extract_section_content 为公共函数，新增 extract_sections 和 extract_summary，重构 guard.rs 引用
    status: completed
  - id: context-engine
    content: 新建 engine/context.rs，实现 assemble_input_context 排序+提取+组装逻辑
    status: completed
  - id: prompt-refactor
    content: 重构 prompt.rs 的 build_input_context 使用 core context engine，调整 prompt 组装顺序为上下文在前指令在后
    status: completed
  - id: dto-ui-update
    content: 更新 dto.rs 的 SkillInputInfo 和 ui/commands.rs 的映射
    status: completed
  - id: skill-yaml-update
    content: 为所有 17 个 skill.yaml 添加合理的 relevance 和 sections 配置
    status: completed
  - id: tests
    content: 为 Relevance 解析、markdown 工具、context 组装、向后兼容添加测试
    status: completed
isProject: false
---

# Selective Context Injection 实现计划

## 背景

根据 [design-philosophy.md](docs/design-philosophy.md) 第 7 节，当前 `popsicle prompt --run` 对所有上游文档做全量平等注入，导致 attention 稀释、噪声干扰、信号丢失。需要通过声明式的 `relevance` 和 `sections` 字段控制注入方式。

## 注入策略

基于 LLM attention U 型分布优化排序：

- **low**: 最先注入（背景铺垫），仅注入文档标题 + 摘要（首段或 H2 标题列表）
- **medium**: 中间位置，仅注入指定 `sections` 中的内容
- **high**（默认）: 最后注入（最靠近 prompt 指令），全文注入

同时将 prompt 组装顺序从 "指令 → 上下文" 调整为 "上下文 → 指令"，使 high relevance 内容和 prompt 指令共同处于 attention 最强区域（末尾）。

## 涉及的文件

### 1. 模型层 — `SkillInput` 扩展

**[crates/popsicle-core/src/model/skill.rs](crates/popsicle-core/src/model/skill.rs)**

- 新增 `Relevance` 枚举：`High | Medium | Low`，实现 `Serialize`/`Deserialize`/`Default`(=High)/`Ord`
- `SkillInput` 新增两个字段：
  - `relevance: Relevance`（默认 `High`，向后兼容）
  - `sections: Option<Vec<String>>`（可选，指定要提取的 H2 章节名）

```yaml
# skill.yaml 新格式示例
inputs:
  - from_skill: domain-analysis
    artifact_type: domain-model
    required: false
    relevance: low           # 新字段
  - from_skill: prd
    artifact_type: prd
    required: true
    relevance: medium         # 新字段
    sections:                 # 新字段
      - Problem Statement
      - User Stories & Acceptance Criteria
  - from_skill: arch-debate
    artifact_type: arch-debate-record
    required: true
    relevance: high           # 默认值，可省略
```

### 2. Markdown 工具模块

**新建 [crates/popsicle-core/src/engine/markdown.rs](crates/popsicle-core/src/engine/markdown.rs)**

将 `guard.rs` 中的 `extract_section_content` 提取为公共工具函数，并新增：

- `pub fn extract_sections(body: &str, section_names: &[&str]) -> String` — 提取指定 H2 章节
- `pub fn extract_summary(body: &str) -> String` — 提取摘要（第一个 H2 前的内容，或所有 H2 标题列表）

`guard.rs` 改为调用此模块的函数。

### 3. 上下文组装引擎

**新建 [crates/popsicle-core/src/engine/context.rs](crates/popsicle-core/src/engine/context.rs)**

将 `prompt.rs` 中的 `build_input_context()` 提取并增强：

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
    pub parts: Vec<ContextPart>,   // 已排序的各部分
    pub full_text: String,         // 拼接后的完整文本
}

pub fn assemble_input_context(inputs: Vec<ContextInput>) -> AssembledContext
```

核心逻辑：

1. 按 relevance 排序：Low → Medium → High
2. Low: 调用 `extract_summary()` 生成摘要
3. Medium: 调用 `extract_sections()` 提取指定章节
4. High: 使用完整 body
5. 为每部分添加 relevance 标注（如 `[Background]`、`[Primary]`）

### 4. Prompt 命令重构

**[crates/popsicle-cli/src/commands/prompt.rs](crates/popsicle-cli/src/commands/prompt.rs)**

- `build_input_context()` 改为调用 core 的 `assemble_input_context()`
- 调整 `full_prompt` 组装顺序：上下文在前，prompt 指令在后
- JSON 输出增加 `relevance` 信息

### 5. DTO 与 UI

**[crates/popsicle-core/src/dto.rs](crates/popsicle-core/src/dto.rs)**

- `SkillInputInfo` 新增 `relevance: String` 和 `sections: Option<Vec<String>>`

**[crates/popsicle-cli/src/ui/commands.rs](crates/popsicle-cli/src/ui/commands.rs)**

- 更新 `SkillInput` → `SkillInputInfo` 的映射

### 6. 模型导出

**[crates/popsicle-core/src/model/mod.rs](crates/popsicle-core/src/model/mod.rs)**

- 导出 `Relevance`

**[crates/popsicle-core/src/engine/mod.rs](crates/popsicle-core/src/engine/mod.rs)**

- 注册 `context` 和 `markdown` 模块

### 7. Skill YAML 更新

为所有现有 skills 添加合理的 `relevance` 和 `sections`：

- **rfc**: arch-debate=`medium`(sections: Decision Matrix), prd=`high`
- **adr**: arch-debate=`high`, prd=`medium`(sections: Problem Statement, Scope)
- **implementation**: rfc=`high`, adr=`high`, test-specs=`medium`(sections: P0/P1 Test Cases)
- **prd**: product-debate=`high`, domain-analysis=`low`
- **arch-debate**: prd=`high`
- 其余 skills 类似处理

### 8. 测试

- `model/skill.rs`: 测试带 relevance/sections 的 YAML 解析，测试默认值向后兼容
- `engine/markdown.rs`: 测试 `extract_sections` 和 `extract_summary`
- `engine/context.rs`: 测试排序逻辑、各 relevance 级别的内容提取、完整组装流程

