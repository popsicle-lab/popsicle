# RFC: Popsicle 支持 IDD 大型项目迁移所需的能力补全

Status: 丢弃
Created: 2026-05-06
Context: 来自一系列关于"用 popsicle + intent-lang 落地 IDD 并迁移大型遗留项目"的方法论讨论
Related: docs/rfc-entity-redesign.md, docs/rfc-multi-agent-discussion-patterns.md

---

## 1. Motivation

Popsicle 当前已具备 Skill / Pipeline / 状态机 / Artifact / Context Layer 等"操作系统"级能力，并通过 `popsicle-spec-development`、`popsicle-intent-development` 两个技能包覆盖了文档与意图建模的大部分基础。

但当目标场景是 **"一个真实的多产品大型遗留项目（数据库 / 网络 / 仿真 / 可视化）以 IDD 方式向新仓库渐进式迁移"** 时，整套体系还缺若干关键能力。本 RFC 列出这些 Gap，并按对"启动迁移"的阻塞度排优先级，作为后续迭代的指引。

本 RFC **不立即提议实现方案**——遵循"先 dogfood，再设计"的纪律，待第一个真实迁移场景跑通后再为高优 Gap 提交独立实现 RFC。

---

## 2. 目标态回顾

完整的 IDD 迁移工作流由以下 Pipeline 组成（详见相关讨论档案）：

- P1 `bootstrap-new-repo`：新仓库 Day-1 骨架
- P2 `legacy-archaeology`：老仓库考古
- P3 `new-product-decision`：PRFC → PDR
- P4 `new-tech-decision`：RFC → ADR
- P5 `scenario-migration`：场景迁移（核心）
- P6 `decision-supersession`：推翻历史决策
- P7 `weekly-health-check`：定时巡检
- P8 `intent-to-code`：从 intent 到可运行代码
- P9 `migration-with-tdd`：迁移场景的 TDD 实现
- P10 `change-implement`：小变更轻量实现
- P11 `legacy-test-augment`：给老代码补 as-is 测试

每条 Pipeline 由若干 Skill 组成，按职责分七组：A 考古 / B 文档建模 / C 形式化 / D 决策评审 / E 迁移执行 / F 测试验证 / G 上下文维护 / H 实现层。

---

## 3. Gap 清单

### 🔴 阻塞级（不补无法启动迁移）

#### Gap 1：老仓库考古能力缺失（A 组）
- 缺：`fact-extractor`、`invariant-miner`、`scenario-slicer`
- 影响 Pipeline：P2 全部 / P5 Step 1
- 没有这组 Skill，新仓库的 PRODUCT.md / ARCHITECTURE.md 无法基于事实建立基线
- 最小可行版本：仅做静态结构提取（API 签名、模块依赖），不做流量录制

#### Gap 2：迁移流程编排缺失（E 组）
- 缺：`migration-planner`、`shadow-runner`、`traffic-cutover`、`deprecation-tagger`、`sunset-enforcer`
- 缺一等公民概念：`MIGRATION.md` 活文档 + `Sunset Date` 时间触发
- 影响 Pipeline：P5 大部分阶段
- 部分能力（如 traffic-cutover）可能用现成基础设施（service mesh、nginx）顶住，但 `migration-planner` 和 Sunset Date 强制机制是 popsicle 必须有的

#### Gap 3：intent ↔ 代码追溯链（H7）
- 缺：`intent-trace-annotator`
- 改 intent 时无法精确定位受影响代码；改代码时无法验证是否仍满足 intent
- 长期维护中 intent 与实现脱钩，IDD 价值会被持续稀释

---

### 🟡 高优级（不补会很痛但能用人肉顶）

#### Gap 4：测试规格 / 红测试生成（H1-H3）
- 缺：`intent-to-testspec`、`testspec-review`、`testspec-to-redtest`
- 当前最多到"intent 写得对"，没有从 intent 派生测试的链路
- 一旦进入实现期，会重新陷入 "LLM 一次性吐测试 + 代码" 的合谋陷阱
- 这也是与 obra/superpowers 方法论对齐的关键环节（true red/green TDD）

#### Gap 5：突变测试验证（H5）
- 缺：`mutation-verifier`
- 没有它等于没有"测试质量裁判"——LLM 生成的测试可能全绿但漏掉关键路径
- 迁移场景的 cutover 必须以 mutation 通过为硬门禁，否则无法证明"新代码真的等价于老代码"

#### Gap 6：adversarial-reviewer（D 组）
- 缺：`adversarial-reviewer`
- 防止"作者+审查者"自循环——尤其当 PRFC/RFC 也由 LLM 起草时
- 当前 popsicle 的 multi-agent-discussion 已涉及，需要与决策 Skill 集成为"必经环节"

---

### 🟢 中长期（半年内补，可先用人肉顶）

#### Gap 7：跨仓库引用
- 当前 popsicle 假设单仓库
- 迁移期需要：新仓库 Skill 能读老仓库的 facts.json、git log、issue/PR
- 建议方案：在 PROJECT_CONTEXT.md 中允许声明 `external_repos`，由 context layer 透明加载

#### Gap 8：Skill 间不同 LLM 模型隔离
- `testspec-to-redtest` 用模型 A，`red-to-green` 用模型 B，反 LLM 自我合谋
- 当前 popsicle 模型选择应该已是 Skill 级配置（待确认），需要文档化为反合谋实践
- 可能需要新增 Skill 元数据字段：`adversarial_to: <skill_id>`，引擎拒绝同模型同时执行

#### Gap 9：subagent 并行编排
- P5 Step 3 的"多模块 red-to-green"需要多 subagent 并行 + 汇总 stage
- 需要确认 popsicle Pipeline DAG 是否原生支持 fan-out/fan-in，以及失败传播策略

#### Gap 10：定时触发 / 时间感知
- 当前 popsicle 是事件驱动
- P7 `weekly-health-check` 和 Sunset Date 都需要定时机制
- 不一定要在 popsicle 内做调度，但需要明确定义"外部 cron 调用 popsicle 命令"的标准接口

#### Gap 11：as-is 与 to-be intent 的标签区分
- 与 intent-lang 协作的 Skill 元数据需求
- 老代码考古产生的 intent 是 as-is（描述现状），不应进入主 invariants 库
- popsicle 侧的 Skill 需要识别 intent 类型并选择存储位置

---

## 4. 行动建议（优先级排序）

遵循"刚好够启动迁移 + 后续按需补"的纪律，下面是建议的实施顺序：

### Phase 0（本周）：建立诚实基线，不写代码
1. 选定要迁移的真实项目
2. 用现有工具链跑 P1 `bootstrap-new-repo`：写 ADR-0001、PDR-0001、MIGRATION.md
3. 选定第一个迁移场景（最薄的端到端切片），写一份 PDR
4. 开 issue tracker 记录"工具不支持"的痛点

### Phase 1（2-3 周）：补 Gap 1 的最小可用版本
- 实现 `fact-extractor` v0：仅静态分析，输出稳定 schema 的 facts.json
- 不做：流量录制、运行时分析、复杂依赖图

### Phase 2（4-6 周）：跑通第一个场景的迁移
- 用现有工具 + Phase 1 的 fact-extractor + 人肉顶其他 Gap
- 目标：一个完整的端到端样本，包含 PDR/ADR/intent/红绿测试/cutover
- 产物：基于真实痛感的 Gap 优先级排序（替代本 RFC 的预测）

### Phase 3（按 Phase 2 反馈排）：开发 2-3 个最痛的新 Skill
- 预测最痛的可能是 `intent-to-testspec` + `mutation-verifier`，但以实际为准
- `intent-trace-annotator` 通常在第 3-5 个迁移场景时才真正需要

### Phase 4（持续）：根据迁移反馈反推 popsicle 引擎改进
- Gap 7-11 的需求会在多个场景的迁移中逐步清晰
- 拒绝在书房里设计——所有引擎级改动必须有真实场景的 forcing function

---

## 5. 反模式 / 克制清单

为避免常见失误，明确"不做"的事：

1. **不要先把 popsicle 改造完美再开始迁移**——会改一年没真实反馈
2. **不要追求第一个迁移场景"完美"**——它的目的是暴露问题，不是树立标杆
3. **不要同时开多个迁移场景**——共性问题会被并行掩盖
4. **不要把 superpowers 模块照搬进 popsicle**——借鉴原则（TDD 红绿、subagent、spec-first 分块），不要硬抄实现
5. **不要为了凑齐七组 Skill 而硬填 Gap**——让 Gap 由真实痛点驱动

---

## 6. 与现有 RFC 的关系

- `rfc-entity-redesign.md`：本 RFC 中的"as-is intent"标签（Gap 11）可能需要 Document 实体扩展
- `rfc-multi-agent-discussion-patterns.md`：Gap 6（adversarial-reviewer）应在此 RFC 框架下实现
- `rfc-project-context.md`：Gap 7（跨仓库引用）属于 ProjectContext 的扩展
- `rfc-scale-adaptive-pipeline.md`：Gap 9（subagent 并行编排）与之相关
- `rfc-selective-context-injection.md`：Gap 1 的 facts.json 应作为 context layer 的输入源

---

## 7. Open Questions

1. `MIGRATION.md` 应该是 popsicle 的一等公民概念（新增 MigrationPlan 实体），还是用现有 Document + 特殊 kind 表达？
2. Sunset Date 的强制机制由谁实现：popsicle CLI 内置 / 外部 cron / Git Action？
3. `adversarial_to` 这种反合谋约束是 Skill 级元数据还是 Pipeline 级配置？
4. as-is intent 是否需要 intent-lang 语言层标记（如 `@asis` annotation），还是仅靠目录约定（`legacy-asis/`）？

这些问题留待第一个迁移场景跑通后回答。

---

## 8. 历史背景

本 RFC 来自一系列关于以下主题的讨论：
- 意图驱动开发（IDD）方法论
- intent-lang 的定位（需求建模 vs 契约语言 vs 程序语言）
- PRD/RFC/ADR 与活文档/决策档案的分离
- 大型项目向 IDD 框架的 Strangler Fig 迁移策略
- 与 obra/superpowers 方法论的整合

完整讨论档案保存在用户本地：
- `idd-intent-lang-discussion.md`
- `idd-doc-migration-discussion.md`
- `idd-new-repo-migration-discussion.md`
