# Popsicle 能力参考

Agent 在创建 Skill 时按需查阅此文件。

## Skill 三件套

```
skills/my-skill/
├── skill.yaml          # 编排配置（引擎读）—— 状态机、依赖、guard、hooks
├── guide.md            # 写作指导（Agent 读）—— 怎么写好这类文档
└── templates/
    └── artifact.md     # 文档骨架（Agent 填）—— H2 章节结构
```

| 文件 | 受众 | 原则 |
|------|------|------|
| skill.yaml | Popsicle 引擎 | 纯配置，不放 prose |
| guide.md | AI Agent | 写作标准、好坏示例、思维框架 |
| template | Agent 填写 | H2 章节对应 has_sections guard |

## 现有 Skill 注册表（17 个）

### 需求与设计

| Skill | 产出物 (artifact_type) | 工作流 | 典型上游 |
|-------|----------------------|--------|---------|
| domain-analysis | domain-model | draft→review→approved | 无 |
| product-debate | product-debate-record | setup→debating→concluded | domain-model (optional) |
| prd | prd | draft→review→approved | product-debate-record |
| arch-debate | arch-debate-record | setup→debating→concluded | prd |
| rfc | rfc | draft→review→accepted/rejected | arch-debate-record, prd |
| adr | adr | draft→review→accepted/rejected | arch-debate-record, prd |

### 测试规划

| Skill | 产出物 | 工作流 | 典型上游 |
|-------|--------|--------|---------|
| priority-test-spec | test-gate-report | gathering→scoring→approved | rfc, adr |
| api-test-spec | api-test-spec | draft→review→approved | rfc |
| e2e-test-spec | e2e-test-spec | draft→review→approved | rfc |
| ui-test | ui-test-spec | draft→review→approved | prd |

### 实现与测试代码

| Skill | 产出物 | 工作流 | 典型上游 |
|-------|--------|--------|---------|
| implementation | impl-record | planning→coding→review→completed | rfc, adr, test specs |
| unit-test-codegen | unit-test-report | generating→running→done | impl-record, test specs |
| api-test-codegen | api-test-report | generating→running→done | impl-record, api-test-spec |
| e2e-test-codegen | e2e-test-report | generating→running→done | impl-record, e2e-test-spec |
| ui-test-codegen | ui-test-report | generating→running→done | impl-record, ui-test-spec |

### 质量

| Skill | 产出物 | 工作流 | 典型上游 |
|-------|--------|--------|---------|
| bug-tracker | bug-report | collecting→triaged→closed | test reports |
| test-report | test-summary | draft→review→approved | test reports |

## CLI 命令速查

### Skill 管理

```bash
popsicle skill list [--format json]
popsicle skill show <name> [--format json]
popsicle skill create <name> --description "..." [--artifact-type <type>] [--local]
```

### Pipeline 管理

```bash
popsicle pipeline list
popsicle pipeline create <name> --description "..." [--local]
popsicle pipeline run <name> --title "..."
popsicle pipeline status [--run <id>]
popsicle pipeline next [--run <id>] [--format json]
popsicle pipeline recommend "<task description>"
popsicle pipeline quick --title "..." [--skill <name>]
```

### 文档操作

```bash
popsicle doc create <skill> --title "..." --run <id>
popsicle doc list [--skill <name>] [--status <s>] [--run <id>]
popsicle doc show <doc-id> [--format json]
popsicle doc transition <doc-id> <action> [--confirm]
```

### 讨论系统

```bash
popsicle discussion create --skill <s> --topic "..." --run <id> [--confidence 1-5]
popsicle discussion role <disc-id> --role-id <id> --name "..." [--perspective "..."]
popsicle discussion message <disc-id> --role <id> --phase "..." --content "..."
  --type: role-statement | user-input | pause-point | phase-summary | decision | system-note
popsicle discussion conclude <disc-id>
popsicle discussion show <disc-id> [--format json]
```

### Issue / Bug / Story / Test

```bash
# Issue
popsicle issue create -t <product|technical|bug|idea> --title "..."
popsicle issue start <key>          # 自动选择 Pipeline 并启动

# Bug
popsicle bug create --title "..." [--severity critical|major|minor|trivial]
popsicle bug record --from-test <tc-key> --error "..."  # 测试失败自动建 Bug

# Story
popsicle story create --title "..." [--issue <key>]

# Test Case
popsicle test create --title "..." --type <unit|api|e2e|ui>
popsicle test run-result <tc-key> --passed/--failed [--error "..."]
```

### 实体提取

```bash
popsicle extract user-stories --from-doc <doc-id>
popsicle extract test-cases --from-doc <doc-id> -t <unit|api|e2e|ui>
popsicle extract bugs --from-doc <doc-id>
```

### Git 集成

```bash
popsicle git init                     # 安装 post-commit hook
popsicle git link [--doc <id>] [--run <id>] [--stage <s>] [--skill <s>]
popsicle git log [-n 20] [--run <id>]
popsicle git review <sha> passed|failed|skipped
```

### 上下文与记忆

```bash
popsicle context show [--run <id>] [--stage <s>]  # Pipeline 运行上下文
popsicle context scan [--force]                    # 项目技术画像

popsicle memory save -t <bug|decision|pattern|gotcha> -s "..."
popsicle memory list [--layer short-term|long-term] [--type <t>]
popsicle memory promote <id>     # 短期 → 长期
popsicle memory check-stale      # git 检测过期
popsicle memory gc               # 清理 stale

popsicle prompt <skill> [--state <s>] [--run <id>]  # 完整 prompt（含上下文注入）
```

## skill.yaml 字段参考

```yaml
name: my-skill                           # 小写+连字符，全局唯一
description: One-line summary            # 一行描述
version: "0.1.0"

inputs:                                  # 上游依赖
  - from_skill: upstream-skill           # 上游 Skill 名
    artifact_type: upstream-artifact     # 上游产出物类型
    required: true                       # false = 可选
    relevance: high                      # high|medium|low，影响 prompt 注入位置
    sections:                            # 只注入指定章节（减少噪声）
      - Section Name

artifacts:                               # 产出物
  - type: my-artifact                    # 全局唯一类型名
    template: templates/my-artifact.md
    file_pattern: "{slug}.my-artifact.md"

workflow:                                # 状态机
  initial: draft
  states:
    draft:
      transitions:
        - to: review
          action: submit
          guard: "upstream_approved"         # 前进转换放 guard
    review:
      transitions:
        - to: approved
          action: approve
          guard: "has_sections:A,B"          # 检查文档完整性
          requires_approval: true            # 需要人工 --confirm
        - to: draft
          action: revise                     # 回退不放 guard
    approved:
      final: true

hooks:
  on_artifact_created: null              # 文档创建后
  on_enter: null                         # 进入非 final 状态
  on_complete: null                      # 进入 final 状态
  # 示例: "popsicle extract bugs --from-doc $POPSICLE_DOC_ID"
```

## Guard 类型

| Guard | 检查内容 | 示例 |
|-------|---------|------|
| `upstream_approved` | 所有 required input 文档处于 final 状态 | `guard: "upstream_approved"` |
| `has_sections:A,B,C` | 文档有这些 H2 标题且内容非占位符 | `guard: "has_sections:Threat Model,Mitigations"` |
| `checklist_complete:Name` | 指定 checklist 全部勾选 | `guard: "checklist_complete:Verification Checklist"` |
| 组合 | 用分号组合多个 guard | `guard: "has_sections:A,B;checklist_complete:C"` |

占位符模式（guard 会拒绝）：`...`, `[Name]`, `[Title]`, `Describe `, `TODO`, `TBD`, `Add detailed content here`

## Pipeline 编排

Pipeline YAML 通过 `depends_on` 定义 DAG：

```yaml
name: my-pipeline
description: ...
stages:
  - name: design
    skill: my-design-skill
    description: Design phase
  - name: impl
    skill: implementation
    depends_on: [design]           # 必须与 Skill inputs 对齐
  - name: parallel-tests
    skills: [api-test-spec, e2e-test-spec]  # 同 stage 内并行
    depends_on: [design]
```

规则：
- `depends_on` 必须与 Skill 的 `inputs.from_skill` 对齐
- 同 stage 多 skill 用 `skills: [a, b]` 并行执行
- Pipeline 的 `scale` 字段影响 `popsicle pipeline recommend` 的推荐

## 现有 Pipeline 模板

| 名称 | 规模 | 阶段 |
|------|------|------|
| full-sdlc | full | product-debate → prd → arch-debate → rfc+adr → test-specs → ui-test → impl → test-codegen → bug+report |
| tech-sdlc | standard | arch-debate → rfc+adr → test-specs → impl → test-codegen → bug+report |
| design-only | planning | product-debate → prd → arch-debate → rfc+adr |
| impl-test | light | impl → test-codegen → bug+report |
| test-only | minimal | test-specs → test-codegen → bug+report |
