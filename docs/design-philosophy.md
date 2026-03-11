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

### 开放问题

- 随着多 Agent 框架的成熟，是否存在特定场景（如极深度的安全审计）适合切换到真正的多 Agent 模式？
- 置信度机制能否进一步影响角色模拟的深度——低置信度时模拟更多角色，高置信度时减少模拟、增加挑战？

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

### 未定决策：多 Module 共存时的 Skill 名字冲突

当用户安装多个 Module 时，不同 Module 可能包含同名 Skill（如两个 Module 都有 `prd`）。由于 `SkillRegistry` 是扁平的，同名 Skill 会冲突。

目前有两个候选策略，尚未做最终选择：

**策略 A：安装时冲突检测，拒绝安装**

```
$ popsicle module install github:someone/game-dev
Error: Skill name conflict — 'prd' already exists (from module 'official').
```

优点：简单、确定性强、不需要改运行时逻辑。
缺点：限制了 Module 生态的灵活性，要求所有 Module 作者全局协调命名。

**策略 B：一次只能激活一个 Module**

一个项目同一时刻只有一个活跃 Module，切换 Module 相当于切换整套工作流。

优点：彻底避免冲突，概念模型极简。
缺点：无法组合不同 Module 的能力（如同时使用"通用开发"和"安全审计"模块）。

在出现真实的多 Module 共存需求之前，暂不做最终决策。两种策略的实现成本都很低，可以根据实际使用模式再选择。

---

*本文档随项目演进持续更新。每个设计决策记录问题、决策和理由，为后续的重新评估提供上下文。*
