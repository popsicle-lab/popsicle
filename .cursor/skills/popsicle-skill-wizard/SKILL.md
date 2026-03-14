---
name: popsicle-skill-wizard
description: 交互式引导创建 Popsicle Skill（skill.yaml + guide.md + template）。当用户想创建新 Skill、定制开发流程、或问到 Popsicle 技能体系时使用。
---

# Popsicle Skill 创建向导

帮助用户交互式地创建 Popsicle Skill。通过结构化问答收集需求，然后调用 CLI 生成脚手架并补全三件套文件。

> 详细的能力参考见 [reference.md](reference.md)

## Phase 1：需求收集

使用 AskQuestion 收集以下信息。如果用户在对话中已经描述了意图，直接从上下文推断，跳过已知项。

### Q1：Skill 基本信息

```
AskQuestion:
  id: skill-name
  prompt: "Skill 名称（小写字母+连字符，如 security-review）"
  options: [自由输入]

  id: skill-purpose
  prompt: "这个 Skill 做什么？"
  options: [自由输入]
```

### Q2：Skill 类型（决定工作流模板）

```
AskQuestion:
  id: skill-type
  prompt: "选择最接近的 Skill 类型"
  options:
    - id: document
      label: "📄 文档型 — 撰写设计文档、规格说明（draft → review → approved）"
    - id: debate
      label: "💬 讨论型 — 多角色模拟辩论（setup → debating → concluded）"
    - id: codegen
      label: "🔧 代码生成型 — 生成并运行代码（generating → running → done）"
    - id: implementation
      label: "⚙️ 实现型 — 编码+验证（planning → coding → review → completed）"
    - id: triage
      label: "🔍 收集型 — 收集→分类→关闭（collecting → triaged → closed）"
    - id: custom
      label: "✏️ 自定义工作流"
```

### Q3：上游依赖

```
AskQuestion:
  id: upstream-inputs
  prompt: "需要哪些上游 Skill 的产出物作为输入？（可多选，不需要则选"无"）"
  allow_multiple: true
  options:
    - id: none
      label: "无上游依赖（独立 Skill）"
    - id: domain-model
      label: "domain-analysis → domain-model"
    - id: product-debate-record
      label: "product-debate → product-debate-record"
    - id: prd
      label: "prd → prd"
    - id: arch-debate-record
      label: "arch-debate → arch-debate-record"
    - id: rfc
      label: "rfc → rfc"
    - id: adr
      label: "adr → adr"
    - id: test-gate-report
      label: "priority-test-spec → test-gate-report"
    - id: api-test-spec
      label: "api-test-spec → api-test-spec"
    - id: e2e-test-spec
      label: "e2e-test-spec → e2e-test-spec"
    - id: ui-test-spec
      label: "ui-test → ui-test-spec"
    - id: impl-record
      label: "implementation → impl-record"
    - id: unit-test-report
      label: "unit-test-codegen → unit-test-report"
    - id: api-test-report
      label: "api-test-codegen → api-test-report"
    - id: e2e-test-report
      label: "e2e-test-codegen → e2e-test-report"
    - id: ui-test-report
      label: "ui-test-codegen → ui-test-report"
    - id: bug-report
      label: "bug-tracker → bug-report"
    - id: test-summary
      label: "test-report → test-summary"
```

### Q4：功能特性

```
AskQuestion:
  id: features
  prompt: "需要哪些附加功能？"
  allow_multiple: true
  options:
    - id: approval
      label: "需要人工审批（requires_approval: true）"
    - id: guard-sections
      label: "章节完整性检查（has_sections guard）"
    - id: guard-upstream
      label: "上游批准检查（upstream_approved guard）"
    - id: guard-checklist
      label: "检查清单验证（checklist_complete guard）"
    - id: hook-extract
      label: "完成后自动提取实体（extract bugs/stories/tests）"
    - id: hook-custom
      label: "自定义 hook（shell 命令）"
    - id: add-to-pipeline
      label: "创建后加入现有 Pipeline"
```

### Q5：存储位置

```
AskQuestion:
  id: location
  prompt: "Skill 存放位置"
  options:
    - id: workspace
      label: "skills/ — 项目级（随项目分发）"
    - id: local
      label: ".popsicle/skills/ — 本地自定义（不随项目分发）"
```

## Phase 2：生成脚手架

根据收集到的信息，执行以下步骤：

### Step 1：调用 CLI 创建脚手架

```bash
popsicle skill create <name> \
  --description "<purpose>" \
  --artifact-type <artifact-type> \
  [--local]  # 如果选了 .popsicle/skills/
```

### Step 2：编写 skill.yaml

根据用户选择生成完整的 `skill.yaml`。以下是各类型的工作流模板：

**文档型：**
```yaml
workflow:
  initial: draft
  states:
    draft:
      transitions:
        - to: review
          action: submit
          guard: "upstream_approved"  # 按需
    review:
      transitions:
        - to: approved
          action: approve
          guard: "has_sections:Section1,Section2"
          requires_approval: true  # 按需
        - to: draft
          action: revise
    approved:
      final: true
```

**讨论型：**
```yaml
workflow:
  initial: setup
  states:
    setup:
      transitions:
        - to: debating
          action: start
    debating:
      transitions:
        - to: concluded
          action: conclude
          requires_approval: true
        - to: setup
          action: restart
    concluded:
      final: true
```

**代码生成型：**
```yaml
workflow:
  initial: generating
  states:
    generating:
      transitions:
        - to: running
          action: run
    running:
      transitions:
        - to: done
          action: complete
        - to: generating
          action: regenerate
    done:
      final: true
```

**实现型：**
```yaml
workflow:
  initial: planning
  states:
    planning:
      transitions:
        - to: coding
          action: start
          guard: "upstream_approved"
    coding:
      transitions:
        - to: review
          action: submit
          guard: "has_sections:Change Summary,Files Changed;checklist_complete:Verification Checklist"
    review:
      transitions:
        - to: completed
          action: approve
          requires_approval: true
        - to: coding
          action: revise
    completed:
      final: true
```

**收集型：**
```yaml
workflow:
  initial: collecting
  states:
    collecting:
      transitions:
        - to: triaged
          action: triage
    triaged:
      transitions:
        - to: closed
          action: close
        - to: collecting
          action: reopen
    closed:
      final: true
```

### Step 3：编写 Input 依赖

根据 Q3 的选择，生成 inputs 配置。artifact_type 到 from_skill 的映射：

| artifact_type | from_skill |
|---------------|------------|
| domain-model | domain-analysis |
| product-debate-record | product-debate |
| prd | prd |
| arch-debate-record | arch-debate |
| rfc | rfc |
| adr | adr |
| test-gate-report | priority-test-spec |
| api-test-spec | api-test-spec |
| e2e-test-spec | e2e-test-spec |
| ui-test-spec | ui-test |
| impl-record | implementation |
| unit-test-report | unit-test-codegen |
| api-test-report | api-test-codegen |
| e2e-test-report | e2e-test-codegen |
| ui-test-report | ui-test-codegen |
| bug-report | bug-tracker |
| test-summary | test-report |

对于每个选择的 input，询问 relevance（high/medium/low）和是否 required。

### Step 4：编写 guide.md

生成 guide.md 模板，包含以下结构：

```markdown
# <Skill Name> Writing Guide

## Purpose
<从用户描述推断>

## Section Standards
<对 template 中每个 H2 章节给出写作标准>

### <Section 1>
- 应包含什么
- 好的示例
- 差的示例

## Thinking Framework
<写作时应思考的问题>

## Common Mistakes
<常见错误>
```

### Step 5：编写 template

根据用户需求生成模板，H2 章节名需与 `has_sections` guard 参数一致：

```markdown
## <Section 1>

<Placeholder text — guard 会检测并拒绝>

## <Section 2>

<Placeholder text>
```

占位符模式（guard 会拒绝）：`...`, `[Name]`, `Describe `, `TODO`, `TBD`, `Add detailed content here`

## Phase 3：Pipeline 集成（可选）

如果用户选择了"加入现有 Pipeline"：

1. 列出可用 Pipeline：`popsicle pipeline list`
2. 用 AskQuestion 让用户选择目标 Pipeline
3. 确定 `depends_on` 关系（基于 inputs 推断）
4. 编辑对应的 pipeline YAML 文件，添加 stage

## Phase 4：验证

创建完成后执行验证：

```bash
popsicle skill show <name> --format json
```

验证清单：
- [ ] skill.yaml 格式正确，CLI 能解析
- [ ] artifact_type 全局唯一
- [ ] inputs 的 from_skill 对应的 Skill 存在
- [ ] workflow 有 initial 状态和至少一个 final 状态
- [ ] guard 仅在前进转换上
- [ ] template H2 章节名与 has_sections 参数匹配
- [ ] guide.md 对每个 H2 章节有写作标准
- [ ] 如加入 Pipeline，depends_on 与 inputs 对齐

## 关键规则

1. **skill.yaml 是纯配置** — 不放 prose，写作指导放 guide.md
2. **guard 只放在前进转换** — submit/approve 放 guard，revise/reopen 不放
3. **artifact_type 全局唯一** — 用 `popsicle skill list --format json` 检查冲突
4. **template H2 = guard 参数** — `has_sections:A,B` 要求 template 有 `## A` 和 `## B`
5. **requires_approval** — 需要用户 `--confirm` 才能执行的转换
6. **hooks 接收环境变量** — `$POPSICLE_DOC_ID`, `$POPSICLE_SKILL`, `$POPSICLE_RUN_ID`
7. **所有命令支持 `--format json`** — 用于结构化输出
