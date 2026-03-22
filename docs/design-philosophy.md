# Popsicle 设计哲学

本文档记录 Popsicle 的核心设计哲学与关键设计决策的思考过程。这些原则指导着功能设计和架构取舍。

## 1. Simulate, Don't Orchestrate — 多角色讨论的本质选择

### 问题

在 AI 辅助的多角色讨论（如 `arch-debate`、`product-debate`）中，存在两种根本不同的实现路径：

**路径 A：单上下文模拟（Single-Context Simulation）**
一个 AI 在单次对话中扮演多个角色，通过视角切换模拟多方辩论。所有角色共享同一个上下文窗口，用户作为唯一真实参与者介入讨论。

**路径 B：多 Agent 对抗辩论（Multi-Agent Adversarial Debate）**
多个独立 AI Agent 分别扮演不同角色，通过消息传递进行真实的对抗辩论。每个 Agent 拥有独立的上下文、立场和推理过程。

### 决策

Popsicle 选择 **路径 A：单上下文模拟**。

### 理由

**共享上下文的优势远大于对抗的价值。**

单上下文模拟中，所有角色发言共享同一个 attention 空间。这意味着：

- **架构师提出方案时，安全专家的知识已经在上下文中**——模型可以在生成架构方案的同时就考虑安全约束，产出质量更高
- **前序角色的论证被后续角色完整看到**——不需要通过摘要或消息传递损失信息
- **矛盾可以在同一个推理过程中被发现和调和**——而非在多轮异步交互中缓慢收敛

多 Agent 对抗辩论看似更"真实"，但实际上引入了不必要的信息损失：

- 每个 Agent 只能通过有限的消息了解其他角色的完整论述
- 上下文在传递过程中被压缩和扭曲
- 协调多个 Agent 的开销远大于模拟的开销
- 真正的"对抗"需要不同的训练目标，而非简单的 prompt 差异

**用户是唯一需要真实存在的角色。** AI 模拟的多角色目的是帮助用户覆盖盲点、拓展方案空间。关键不在于角色之间是否"真正"在辩论，而在于最终产出的方案是否充分考虑了多维视角。单上下文模拟通过共享 attention 恰好更好地实现了这个目标。

### 已知局限：单上下文角色趋同

学术研究（Du et al., 2023; Wang et al., 2024）和架构分析表明，单上下文模拟存在角色分化弱的固有限制，根因是三重机制叠加：

- **自回归锚定**——token 从左到右生成，后面的角色被前面角色的输出锚定，不存在真正的并发独立推理
- **自洽性压力**——模型训练目标优化文本连贯性，"A 说 X、B 反驳 X"天然违背训练目标，模型倾向让角色趋同
- **注意力交叉污染**——Self-attention 让所有 token 互相可见，角色间无信息屏障；而多 Agent 系统中各 Agent 上下文物理隔离

Popsicle 接受这个 trade-off：对于"帮助用户覆盖盲点"的目标，共享上下文带来的信息完整性收益大于角色对抗性的损失。但这个局限性需要通过评测来监控——第 8 节的"视角分化度"指标正是为此设计的退化检测信号。

### 开放问题

- 当评测持续观察到视角分化度低于阈值（角色退化为"换马甲说同样的话"），应触发向多 Agent 多 Session 架构的切换。具体阈值和切换机制待定
- 轻量级场景（如快速 sanity check）可保持单上下文并发；深度辩论场景（如安全审计）可考虑单 Session 顺序多角色作为中间方案——在预算和延迟之间取得平衡

## 2. CLI Executes, UI Observes — 执行与观察的分离

### 问题

AI Agent 和开发者如何与系统交互？UI 应该是操作入口还是观察窗口？

### 决策

CLI 是唯一的写路径，Desktop UI 只读。

### 理由

AI Agent 天然适合 CLI 交互——它们可以精确地构造命令、解析 JSON 输出、链式调用。把 UI 限制为只读，消除了两个写入路径之间的同步和冲突问题。开发者需要看清全局（Pipeline DAG、文档状态、Git 关联），但不需要通过 GUI 点击来推进工作流——这是 Agent 的工作。

这也意味着 UI 可以安全地被任何人在任何时候打开，不会产生副作用。

## 3. Skills as First-Class Citizens — Skill 是能力的原子单元

### 问题

如何组织和复用开发过程中的各种能力（需求分析、架构设计、测试规划等）？

### 决策

每个 Skill 是自包含的能力单元，携带自己的状态机、文档模板、AI 指导和生命周期钩子。

### 理由

将 Skill 设计为一等公民而非"Pipeline 的步骤"，使得：

- **同一个 Skill 可以在不同 Pipeline 中复用**——`arch-debate` 可以出现在 `full-sdlc`、`tech-only` 或独立使用
- **Skill 的内部工作流与 Pipeline 编排解耦**——Pipeline 只关心 Skill 的完成状态，不管内部经历了几轮 draft → review → revise
- **新能力可以独立开发和测试**——`popsicle skill create` 创建的 Skill 不需要立刻加入 Pipeline

**关注点分离原则：** `skill.yaml` 是纯编排配置（引擎读），`guide.md` 是纯写作指导（Agent 读），`template` 是文档骨架（Agent 填）。三个文件服务于三个不同的受众，不应混杂。

## 4. Guards Enforce Discipline — 信任但验证

### 问题

如何确保开发流程的质量而不增加人工审批的瓶颈？

### 决策

通过声明式 Guard 条件在工作流转换时自动验证。

### 理由

Guard 实现了"信任但验证"的理念：

- `upstream_approved` 确保上游文档已完成——不会在 PRD 未批准时开始架构设计
- `has_sections` 确保文档有实质内容——不会让模板占位符通过审批

Guard 是声明式的，不需要人工审批流程。它们定义的是"什么条件必须满足"，而非"谁来检查"。这让 AI Agent 可以自主推进工作流，同时维持质量底线。

## 5. Hybrid Storage — 对 Git 友好的混合存储

### 问题

文档和元数据如何存储？

### 决策

文档作为 YAML frontmatter + Markdown 文件存储（Git 友好），元数据和状态索引在 SQLite 中。

### 理由

Markdown 文件是开发者最自然的文档格式，可以直接在 IDE 中阅读和编辑，天然支持 Git diff 和 code review。但纯文件系统不适合做状态查询和关联查询，因此用 SQLite 做索引层。

这种混合方案让文档跟着代码走（同一个 Git 仓库），同时保持了查询的高效性。

## 6. Multi-Agent Native — 多 Agent 原生支持

### 问题

如何支持不同的 AI Agent（Claude Code、Cursor 等）？

### 决策

`popsicle init` 自动生成每个 Agent 原生格式的指令文件，遵循各自的 Agent Skills 规范。

### 理由

每个 AI Agent 有自己的指令格式和发现机制（Claude 的 `CLAUDE.md` + `skills/`，Cursor 的 `.mdc` + `skills/`）。与其设计一个抽象层让 Agent 适配，不如直接生成它们原生格式的文件。

生成的文件包含完整的 Skill 注册表——Agent 名称、产出物类型、输入依赖、工作流状态、转换、Guard 条件——Agent 不需要先调用 CLI 就能理解完整的开发工作流。

## 7. Selective Context Injection — 不是所有上下文都同等重要

### 问题

当 Skill 需要上游文档作为 AI 上下文时，应该如何注入？全量注入所有上游文档，还是有选择性地控制注入内容？

### 背景

一些框架（如 BMAD-METHOD 的 step-file 架构）通过将工作流拆分为多个步骤文件来控制上下文加载——每步只加载当前需要的上下文，解决 LLM 的 "lost in the middle" 问题。

但 Popsicle 的使用场景不同：

- **单窗口工作流**：典型场景下用户在一个 AI 会话中推进 pipeline，前序步骤的对话已经在上下文中，不需要分步重新加载
- **`popsicle prompt` 已有跨会话恢复能力**：换窗口时通过 `--run` 参数自动注入上游文档，已解决上下文丢失问题

因此 step-file 的"分步加载"在 Popsicle 中价值有限。但一个更本质的问题仍然存在：**LLM 对上下文的注意力分布不均匀。**

### 决策

通过声明式的 `relevance` 和 `sections` 控制上游文档的注入方式和优先级，而非全量平等注入。

### 理由

**LLM 的 attention 呈 U 型分布——开头和末尾的内容被记住最多，中间部分容易被忽略。** 当注入多个上游文档时，所有内容平等对待会导致：

- **attention 稀释**：对当前步骤最关键的文档可能被排在中间，反而被忽略
- **噪声干扰**：低相关度的上游文档（如 `domain-analysis` 对 `implementation`）占据上下文空间但贡献有限
- **信号丢失**：高相关度文档的关键信息被淹没在大量背景材料中

Skill 的 `inputs` 定义应扩展为支持选择性注入：

- **`relevance`**：声明输入的重要程度（`high` / `medium` / `low`），影响注入位置和方式
- **`sections`**：只注入指定章节而非全文，减少噪声

注入策略基于 attention 分布优化：

- `low` 放最前面（背景铺垫），可只注入标题和摘要
- `medium` 注入指定 sections
- `high` 全文注入，放在最靠近 prompt 的位置（attention 最强区域）

这个设计保持了 Popsicle 的声明式哲学——Skill 定义"需要什么、什么最重要"，引擎决定"如何最优地呈现"。不引入新的文件结构或工作流概念，仅扩展已有的 `SkillInput` 模型。

### 开放问题

- 是否需要支持 `sections` 的模式匹配（如正则或 glob），以应对文档结构不完全一致的情况？
- 是否引入动态 relevance——根据当前文档状态或内容自动判断上游文档的重要程度？
- 摘要生成（`low` relevance 时只注入摘要）应由引擎在文档完成时预生成，还是在注入时实时截取？

## 8. Testable by Design — Skill 与 Pipeline 的可评测性

### 问题

Skill 是 prompt + 工作流的组合，Pipeline 是 Skill 的 DAG 编排。当 Skill 的指令、模板或 Guard 发生变更时，如何知道变更是"改进"还是"回归"？仅靠主观感受（vibe check）无法支撑迭代。

### 背景

与传统的"单次执行、产出代码"的 Agent Skill 不同，Popsicle 的 Skill 有三个独特之处需要纳入评测设计：

- **多轮交互**：debate 类 Skill 有 4 个 Phase 和多个暂停点，不是一次调用就结束
- **结构化文档产出**：产出物是 Markdown 文档而非代码，质量无法用"编译通过"衡量
- **Pipeline 上下游依赖**：Skill 不是孤立运行的，上游产出的质量直接影响下游的表现

这意味着评测体系不能只看单个 Skill，必须覆盖三个层面。

### 决策

采用三层评测架构，从局部到全局逐层验证。

#### Layer 1：单 Skill 评测

对单个 Skill 的产出和过程做独立评价，分为确定性检查和模型辅助评分两类。

**确定性检查（自动化，基于结构化 trace）：**

Debate 类 Skill 通过 `popsicle discussion show --format json` 获取结构化 trace，检查：

- 是否经历了全部 Phase（检查 `phase` 字段覆盖度）
- 每个 Phase 是否有 `pause-point` 和 `phase-summary` 类型消息
- 注册角色数是否在 4-6 范围内
- 用户发言后是否有 >= 2 个角色回应

文档类 Skill 通过 `popsicle doc show --format json` 检查：

- 工作流转换序列是否合法（如 `gathering → scoring → approved`）
- Guard 条件是否全部满足
- 文档是否有非占位符内容

代码类 Skill 检查验证清单（Verification Checklist）勾选状态和构建/测试命令执行情况。

**模型辅助评分（Rubric-based，按 Skill 类型定义）：**

为每类 Skill 定义评分 rubric，由 AI 对产出做结构化打分。评分维度因 Skill 类型而异：

- PRD 关注问题清晰度、指标可量化性、用户故事可测试性、范围边界明确性
- Debate 关注视角分化度（角色是否真的在提供不同视角）、取舍显性化（决策矩阵是否体现真实权衡）、置信度适配（挑战程度是否匹配用户设定）
- RFC/ADR 关注方案与 debate 决策的一致性、接口定义完整性

视角分化度是验证"Simulate, Don't Orchestrate"设计决策的关键指标——如果持续观察到分化度低于阈值，说明单上下文模拟退化为"换马甲说同样的话"，是数据驱动地重新评估多 Agent 方案的信号。

#### Layer 2：Skill 衔接评测

验证 Pipeline DAG 中上游产出能否被下游正确消费，核心度量是**信息保真度**。

上游文档中的关键实体（功能名、API 名、架构组件名、决策选项名）应在下游文档中被引用或体现。如果某个关键实体在传递链中断裂，说明信息在 Skill 衔接处丢失。

关键衔接点：

| 上游 → 下游 | 检查内容 |
|------------|---------|
| `product-debate` → `prd` | debate 决策矩阵的选中方案是否在 PRD 中被保留 |
| `prd` → `arch-debate` | debate 是否以 PRD 功能需求为约束展开讨论 |
| `arch-debate` → `rfc` + `adr` | RFC 技术方案是否与 debate 最终选择一致 |
| `rfc` + `adr` → `implementation` | 实现是否覆盖了 RFC 描述的关键组件和接口 |
| `*-test-spec` → `*-test-codegen` | 测试代码是否覆盖了 test-spec 中的 P0/P1 用例 |

现有的 `upstream_approved` Guard 只检查"上游是否完成"，衔接评测进一步检查"上游的内容是否被下游正确理解和使用"。

#### Layer 3：Pipeline 端到端评测

对整条 Pipeline（如 `full-sdlc`）做全局度量：

| 指标 | 含义 |
|------|------|
| 通过率 | 所有 Skill 的 Guard 是否一次性通过，回退次数 |
| 文档一致性 | 最终 impl-record 与最初 product-debate 的目标是否一致 |
| Token 效率 | 整条 Pipeline 的 token 总量，各阶段分布 |
| 回归检测 | 同一 prompt，不同版本 Skill 的产出是否回归 |

### 理由

**Guard 是必要条件，Eval 是充分条件。**

Guard 回答"能不能过"——文档有没有必需的章节、上游是否批准。但 Guard 无法回答"过得好不好"——PRD 的问题陈述是否具体、debate 的角色是否真的在对抗、RFC 的接口设计是否覆盖了 PRD 的需求。

三层评测的设计让每一层都有明确的职责和独立的价值：

- **Layer 1** 保证单个 Skill 的产出质量，是最小可用的 eval
- **Layer 2** 保证 Skill 之间的信息传递不失真，这是 Pipeline 编排的核心价值
- **Layer 3** 保证端到端的一致性和效率，是对整个系统的信心度量

这三层与现有基础设施天然对齐：Discussion 持久化提供 debate trace，`--format json` 提供结构化数据，Guard 提供最小检查基线。

### 扩展方向

- **Skill Eval 分发包**：每个 Skill 目录下可携带自己的 eval 定义（prompt 测试集、rubric schema、黄金标准产出），使 eval 成为 Skill 的一部分随 `popsicle init` 分发
- **`popsicle eval` 命令族**：`eval skill`、`eval discussion`、`eval pipeline`、`eval report` 等子命令，提供统一的评测入口
- **Eval Dashboard**：Desktop UI 新增评测面板，展示 Skill 质量趋势、衔接保真度、Pipeline 通过率随时间的变化

## 9. Modules as Self-Contained Distribution — 模块是自包含的分发单元

### 问题

Skill 和 Pipeline 如何分发、组织和安装？随着远程拉取能力的引入，需要一个明确的分发边界。

### 决策

引入 Module 作为分发单元。一个 Module 包含一组完整的 Skill 和多条 Pipeline，Pipeline 只引用本 Module 内的 Skill。

### 三者关系

Module、Skill、Pipeline 各司其职，通过 Skill 名字松耦合：

- **Module 管"有什么"**——一组相关 Skill + 配套 Pipeline 的打包，是安装和卸载的原子单元
- **Skill 管"怎么做"**——自包含的能力单元，携带状态机、模板、AI 指导
- **Pipeline 管"怎么排"**——将 Skill 编排为 DAG，定义阶段顺序和依赖关系

```
Module（分发单元）
  ├── skills/           全部 Skill 在此
  │   ├── prd/
  │   ├── rfc/
  │   ├── implementation/
  │   └── ...
  └── pipelines/        多条 Pipeline，自由组合上面的 Skill
      ├── full-sdlc.pipeline.yaml
      ├── tech-sdlc.pipeline.yaml
      └── test-only.pipeline.yaml
```

Pipeline 通过 Skill 名字引用 Skill，不感知 Module 的存在。Module 是安装时概念，运行时透明——安装完成后，所有 Skill 加载到扁平的 `SkillRegistry`，Pipeline 按名字查找即可。

### 理由

**自包含消除了跨模块依赖的复杂性。**

如果允许 Pipeline 跨 Module 引用 Skill，就需要引入模块依赖声明、依赖解析、缺失 Skill 的降级策略——这些都是包管理器级别的复杂性。而 Popsicle 的核心价值在于工作流编排，不在于包管理。

一个 Module 就是一个自包含的世界：装进来就能用，删掉就干净。Pipeline 引用的所有 Skill 都保证存在于同一个 Module 内，不会出现"装了 Pipeline 但缺 Skill"的问题。

**分发与运行分离。** Module 只影响"哪些文件被安装到 `.popsicle/`"，不影响运行时的任何逻辑。`SkillLoader`、`PipelineLoader`、`Advisor`、`Guard` 等引擎组件无需感知 Module 的存在。这意味着 Module 系统的引入对现有代码的侵入性极小。

### 远程拉取

Module 的物理形态是一个包含 `module.yaml` + `skills/` + `pipelines/` 的目录，可以是 Git 仓库或仓库内的子目录。安装过程是文件复制：

```bash
popsicle module install github:popsicle-dev/official
```

1. 拉取到临时目录
2. 验证 `module.yaml`，检查 Skill/Pipeline 名字冲突
3. 复制到 `.popsicle/modules/<name>/`
4. 更新 `config.toml` 记录来源和版本

内置 Module 编译进二进制作为离线 fallback，`popsicle init` 时解压到 `.popsicle/modules/official/`。

### 决策：单活跃 Module

一个项目同一时刻只有一个活跃 Module，安装新 Module 即替换当前活跃 Module。`config.toml` 的 `[module]` 段记录当前活跃 Module 的名称、来源和版本。

**选择理由：**

- **跨 Module 的 Pipeline 组合不存在。** Pipeline 只引用本 Module 内的 Skill，多 Module 共存的唯一场景是"在不同 Pipeline Run 中使用不同 Module 的 Pipeline"，这完全可以通过切换活跃 Module 实现
- **Skill 级别的定制已有覆盖机制。** 用户可在 `.popsicle/skills/` 放置同名 Skill 覆盖 Module 中的默认行为，无需引入第二个 Module
- **多 Module 共存是包管理器级别的复杂度**——依赖声明、名字冲突解析、缺失 Skill 降级策略——不是 Popsicle 应承担的
- **如需组合不同 Module 的能力，正确做法是创建一个包含两套 Skill 的新 Module**，而非让引擎处理多 Module 共存

加载优先级（后加载覆盖先加载）：

1. `.popsicle/modules/<active>/skills/` — Module 提供（最低优先级）
2. `.popsicle/skills/` — 用户自定义覆盖
3. `workspace skills/` — 开发中的 Skill（最高优先级）

## 10. Domain Priming over Persona — 领域约束优于角色扮演

### 问题

是否应该为 Skill 引入代理人格系统——让 AI 在执行不同 Skill 时扮演不同角色（如"架构师 Winston"、"产品经理 John"），以提升产出质量？

### 背景

一些框架（如 BMAD-METHOD）为每个代理定义完整的人格：名字、身份、沟通风格、原则。其假设是角色扮演能让 LLM 更"专业"地执行任务。

### 决策

**不引入代理人格。** 继续使用 `guide.md`（领域写作指导）+ `prompts`（任务指令）+ Guard（结构化约束）的组合。

### 理由

多项 2025 年研究表明，**专家人格提示（persona prompting）对 LLM 任务性能的提升不稳定，而领域约束（domain priming）更可靠。**

**人格提示的问题：**

- 专家人格通常带来正面或**不显著**的性能变化，但模型对无关的人格细节高度敏感，**性能下降可达 30 个百分点** [1]
- 在事实准确性任务上，领域内专家人格对性能**无显著影响** [2]
- 人格提示的效果波动很大，在数学任务上 Gemini 下降 6.1%，GPT-4 下降 3.3% [3]

**领域约束（Domain Priming）更有效：**

- 非人格的领域提示一致性地提升了约 +2.5% 的性能 [3]
- 效果更稳定、可预测，不受随机人格细节的干扰

**唯一的正面证据来自代码生成：** 性格特质引导（如"谨慎、注重细节"）在代码生成任务中提升了 pass rate，28 组实验中 23 组有提升 [4]。但这更接近于行为约束而非角色扮演——与 Popsicle 已有的 `guide.md` 写作指导本质相同。

Popsicle 现有机制已经对应了研究中被证明有效的模式：

| Popsicle 机制 | 研究概念 | 研究结论 |
|---|---|---|
| `guide.md`（写作指导） | Domain priming | 有效，稳定提升 |
| `skill.yaml` prompts（任务指令） | Task-specific prompting | 有效 |
| Guard（结构化约束） | Constraint prompting | 有效 |
| 人格系统（名字、身份、风格） | Persona prompting | 不稳定，可能有害 |

### 开放问题

- 未来模型是否会对人格提示更鲁棒？如果出现可复现的正面证据，可重新评估。
- 是否在 `guide.md` 中增加行为约束（如"优先考虑安全性"、"质疑每个假设"）作为 domain priming 的强化？这不同于人格扮演，是对 LLM 行为的直接约束。

### 参考文献

1. Luz de Araujo, P.H., Röttger, P., Hovy, D., & Roth, B. (2025). *Principled Personas: Defining and Measuring the Intended Effects of Persona Prompting on Task Performance.* EMNLP 2025. https://aclanthology.org/2025.emnlp-main.1364/
2. *Prompting Science Report 4: Playing Pretend: Expert Personas Don't Improve Factual Accuracy.* arXiv:2512.05858. https://arxiv.org/abs/2512.05858
3. *Domain Priming vs Persona Prompting: Performance Comparison.* OpenReview. https://openreview.net/pdf?id=sVaRgmH8FE
4. *Personality-Guided Code Generation Using Large Language Models.* ACL 2025. https://aclanthology.org/2025.acl-long.54/

## 11. Project Context as Persistent Ground Truth — 项目技术画像的持久化

### 问题

`popsicle prompt` 和 `popsicle context` 提供的是 **pipeline 运行时的文档上下文**——PRD、RFC、test-spec 等产出物。但 LLM 在执行 Skill 时还需要另一层信息：**项目本身的技术约束**——用什么语言、什么框架、什么编码规范、什么测试模式。这类信息不属于任何 Skill 的产出物，却影响每个 Skill 的执行质量。

### 决策

引入 `popsicle context scan` 命令，自动分析项目文件生成技术画像，持久化为 `.popsicle/project-context.md`。此文件在 `popsicle prompt` 时自动注入，作为所有 Skill 的背景上下文。

### 理由

**domain priming 的效果取决于约束的具体性。**

`guide.md` 提供的是通用的写作指导（"分析领域边界"），但缺少项目特定的技术约束。一个 Rust 项目和一个 TypeScript 项目执行同一个 `rfc` Skill 时，架构决策空间完全不同。如果 LLM 不知道项目用 Rust，它可能建议一个依赖 Node.js 生态的方案。

自动生成的 project context 提供这一层缺失的约束：

- 从 `Cargo.toml` / `package.json` / `go.mod` 等推断技术栈和版本
- 从目录结构推断项目组织模式（monorepo、workspace、微服务等）
- 从 `.gitignore`、CI 配置推断开发实践
- 从已有代码推断编码风格和模式

**这是 domain priming 的强化，不是 persona。** 研究表明具体的领域约束比抽象的角色设定更能提升 LLM 性能（见第 10 节）。Project context 把这个原则从 Skill 级别提升到项目级别。

### 设计约束

- **可编辑**：自动生成后用户可以手动补充和修改，`scan` 命令不覆盖已有内容
- **注入位置**：作为 prompt 的最前部（背景铺垫，`low` relevance 位置），不抢占高权重上游文档的 attention
- **按需更新**：不自动重新扫描，用户显式运行 `popsicle context scan` 时才更新

### 开放问题

- 自动扫描应该提取哪些信息？过少则无价值，过多则成为噪声。需要实验确定最优信息密度
- 是否支持用户在 `config.toml` 中声明技术栈，作为 scan 的补充或替代？
- Project context 是否应该参与 Guard 检查——如 RFC 提议的技术方案是否与 project context 声明的技术栈一致？

## 12. Scale-Adaptive Pipeline Selection — 规模自适应的 Pipeline 推荐

### 问题

Popsicle 提供多条 Pipeline（`full-sdlc`、`tech-sdlc`、`test-only`），但用户需要自己判断哪条适合当前任务。一个小 bug 修复走 `full-sdlc` 是浪费，一个大型功能走 `test-only` 则覆盖不足。

### 决策

在 Advisor 中增加 Pipeline 推荐能力：根据任务描述和项目上下文，建议最合适的 Pipeline。

### 理由

**Pipeline 选择本质上是一个复杂度评估问题。** 不同规模的任务需要不同深度的流程：

| 任务规模 | 适合的 Pipeline | 信号 |
|---------|----------------|------|
| Bug 修复、配置变更 | `test-only` 或无 Pipeline | 单文件变更、无架构影响 |
| 小型功能、技术重构 | `tech-sdlc` | 不涉及产品需求变更、影响范围可控 |
| 新功能、跨模块变更 | `full-sdlc` | 涉及用户场景、需要产品讨论和架构决策 |

当前 `popsicle pipeline next` 已经能推荐"下一步做什么"，但这是在 Pipeline 选定之后。Pipeline 推荐是更上游的决策——**在用户开始之前，帮助选择正确的深度。**

### 实现方向

**近期（低成本）**：`popsicle pipeline recommend` 命令，接受任务描述作为输入，基于关键词和启发式规则推荐 Pipeline。规则可以编码为简单的匹配逻辑：

- 提及"bug"、"fix"、"typo"、"config" → 推荐 `test-only` 或跳过 Pipeline
- 提及"refactor"、"migrate"、"upgrade" → 推荐 `tech-sdlc`
- 提及"feature"、"user story"、"product" → 推荐 `full-sdlc`

**中期（与 Issue 集成）**：当用户通过 `popsicle issue start` 启动工作流时，根据 Issue 的标签、描述、关联文档自动推荐 Pipeline，用户确认后启动。

**长期（数据驱动）**：根据历史 Pipeline 运行数据（Token 消耗、回退次数、完成时间）学习推荐模型，自动调整推荐阈值。

### 开放问题

- 推荐是否可以覆盖到 Pipeline 内部——对于 `full-sdlc`，是否建议跳过某些阶段（如已有架构的项目跳过 `arch-debate`）？
- 是否需要支持用户定义推荐规则，而非硬编码启发式？
- 错误推荐的成本如何控制？推荐过轻（如大功能走 `test-only`）比推荐过重（小修复走 `full-sdlc`）危害更大

## 13. Auto-Memory — Bug 与决策点的跨会话持久记忆

### 问题

AI Agent 在不同会话中反复遇到相同的 bug，做出相互矛盾的决策，或忘记之前踩过的坑。每次新会话都是"失忆重启"，项目积累的经验无法跨会话传递。

### 背景

Claude 的 auto-memory 通过自动检测、持久化、跨会话注入解决通用记忆问题。但开发场景的记忆有两个特殊性：

1. **记忆的失效是事件驱动的**：代码重构、依赖升级会让记忆瞬间过时，而非随时间平滑衰减
2. **记忆的价值密度要求极高**：注入到 prompt 的每一行都消耗 attention，无效记忆是负优化

参考 [ai-school](https://github.com/curtiseng/ai-school) 的四层记忆架构（Sensory → ShortTerm → LongTerm → Semantic），开发场景不需要 Sensory（无瞬时感知）和 Semantic（无持续仿真的自我认知提炼），简化为两层即可。

### 决策

采用 **路径 C：Popsicle CLI 存储 + Agent Skill 触发** 的混合架构，两层记忆模型（短期 + 长期），单 Markdown 文件存储，200 行上限。

- **CLI 层**（`popsicle memory`）：负责存储、检索、注入、生命周期管理。跨 Agent 可用。
- **Agent 层**（Cursor Skill / Claude Code Skill）：负责触发时机——何时写入记忆、何时合并同类 bug 为 pattern。
- **注入层**：`popsicle prompt` 时按规则匹配注入 top-N 条相关记忆。

### 理由

**存储在 CLI，触发在 Agent，符合已有的架构分离原则。**

将记忆存储放在 Popsicle CLI 层而非 Agent 层（如 `.cursor/rules/`），确保 Claude Code、Cursor 等任何 Agent 都能读写同一份记忆——与第 6 节 "Multi-Agent Native" 一致。

将触发逻辑放在 Agent Skill 层而非 CLI 引擎层，避免在 CLI 中做自然语言理解。Agent 天然擅长判断"这次修复值得记住"，Skill 定义的是触发规则（如"修复 bug 后调用 `popsicle memory save --type bug`"），随 Agent 能力进化而更新。

**两层记忆 + 单文件 + 200 行上限，是刻意的约束。**

- 短期记忆：当前或最近 pipeline run 的临时洞察，3 个 run 未被引用则自动遗忘
- 长期记忆：经过验证的持久经验，仅在关联代码被大幅重构时标记 stale，不自动删除
- 单 Markdown 文件（`.popsicle/memories.md`）：Git 友好、人可读、Agent 可解析，与第 5 节 Hybrid Storage 一致
- 200 行上限迫使系统做有意义的压缩——同类 bug 合并为 pattern，而非无限堆积

**遗忘机制是事件驱动的，不是时间衰减的。**

ai-school 用 `exp(-decay × hours)` 做连续衰减，适合仿真场景。开发记忆的过时由离散事件触发：依赖升级、文件重构、架构变更。通过 git diff 检测关联文件变更来标记 stale，比时间窗口更准确。

### 开放问题

- 记忆的注入上限应该是多少条？top-5 还是 top-10？需要实验确定注入量与 attention 稀释的平衡点
- pattern 合并（多条 bug → 一条 pattern）应由 Agent 在会话中完成，还是由 CLI 提供 `popsicle memory consolidate` 命令？
- 短期 → 长期的提升除了引用次数，是否需要用户显式确认？

## 14. Work Items as Extracted Entities — 从文档中涌现的可追踪实体

### 问题

Popsicle 的 Skill 产出物是 Markdown 文档——PRD 中描述用户故事，test-spec 中罗列测试用例，bug-tracker 中记录缺陷。但这些信息埋在文档文本中，无法被独立查询、关联和追踪。一个 PRD 里的用户故事有多少被测试覆盖了？一个测试用例对应哪些 acceptance criteria？这些问题靠全文搜索回答不了。

### 决策

引入 Bug、UserStory、TestCase 作为一等实体，存储在 SQLite 中，与文档、Pipeline Run、Issue、Commit 建立关联。提供 `popsicle extract` 命令从已有 Markdown 文档中解析并创建这些实体。

### 理由

**文档是叙事，实体是结构。两者互补，不可替代。**

一份 PRD 文档用自然语言描述了需求的上下文、取舍和细节——这是 LLM 写作和人类理解的最佳载体。但当需要回答"哪些用户故事还没有被测试覆盖"时，叙事格式无能为力。结构化实体填补了这个空白：

- **UserStory** 携带 AcceptanceCriteria，每个 AC 可关联 TestCase——构建需求到测试的可追踪链
- **TestCase** 区分类型（Unit/API/E2E/UI）和优先级（P0/P1/P2），支持 `test coverage` 聚合查询
- **Bug** 关联触发它的 TestCase 和修复它的 Commit，形成"发现→修复→验证"的闭环

**提取而非手工创建。** `popsicle extract` 用正则从已有文档中解析实体（识别 `### Story N:` 标题、`**As a**` 句式、`- [ ]` 为 AC 等），而非要求用户重新录入。这保持了"文档优先"的工作流——先写文档，再从文档中涌现结构化数据。

**实体是文档的索引，不是文档的替代。** UserStory 记录的是"谁要什么"的结构化摘要，完整的业务上下文仍在 PRD 文档中。`story show` 展示结构化视图，`doc show` 展示完整叙事——两个视角服务于不同场景。

### 设计约束

- **双向关联**：每个实体记录 `source_doc_id`（从哪个文档提取）和 `pipeline_run_id`（属于哪个 Pipeline Run），支持正向和反向查询
- **自动去重**：`bug record --from-test` 在创建前检查是否已有同一 TestCase 的 Open bug，避免重复
- **渐进结构化**：实体可以从文档提取，也可以手动创建——不强制所有信息必须先写进文档
- **Key 而非 UUID**：Bug/Story/TestCase 使用人类可读的递增 key（`BUG-0001`、`STORY-0001`、`TC-0001`），便于在对话和 commit message 中引用

### 开放问题

- 提取的准确率依赖文档格式的规范程度。是否需要在 Skill 的 `guide.md` 中更严格地规定格式，以提升提取成功率？
- 实体的生命周期是否应该影响 Pipeline 进度——如所有 P0 TestCase 必须为 Automated 状态，Pipeline 才能完成？
- 是否需要实体级别的 Guard——如 Bug status 为 Open 时阻止 Pipeline 归档？

---

*本文档随项目演进持续更新。每个设计决策记录问题、决策和理由，为后续的重新评估提供上下文。*
