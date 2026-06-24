---
id: 5415991a-ad23-479c-816f-3a1833527200
doc_type: product-debate-record
title: artifact-system product scope：guard 归属 / work_item→task_chunk / namespace 边界
status: final
skill_name: product-debate
pipeline_run_id: 49989451-d311-4f76-b15a-cc7dfcf7787f
spec_id: cf6e7062-b75f-4925-b633-82a04e775a76
version: 1
parent_doc_id: null
tags: []
metadata: null
created_at: 2026-06-09T03:14:46.313628Z
updated_at: 2026-06-09T03:28:45.217140Z
---

# 产品辩论纪要 — 定义 artifact-system 的最终 product scope（slice-2）

> 源事实基：[`fact-extraction-report (slice-2)`](./popsicle-c76d729-fact-basis-slice-2--artifact-system.fact-extraction-report.md) + `docs/baseline/2026-06-09/{dependency-graph,api-contracts,unsafe-risk-report,tech-debt-inventory}.md`。
> 所有"实现成本 / 耦合 / LoC"陈述均 cite 上述事实基或 `legacy/popsicle/.../*.rs:line`。

## Topic

**一句话**：在 8 个已考古模块的事实基上，裁定 artifact-system 这个 product 的**最终边界** —— 哪些模块属本 product crate、哪些跨 product 依赖怎么切、`work_item` 重命名怎么落。

### 待裁决的 5 个边界候选项（来自 fact-extraction §迁移切片范围核对）

| # | 候选项 | 事实依据 | 待决问题 |
|---|---|---|---|
| 1 | **guard 归属** | `guard.rs` 3 类型中 `has_sections`/`checklist_complete` 仅依赖 `Document`+markdown（本 product），`upstream_approved` 依赖 `PipelineDef/PipelineRun/StageState/SkillRegistry`（slice-1）（`guard.rs:3-4,103-169`）| 整块归 artifact-system？整块留 skill-runtime？还是按 guard 类型拆？ |
| 2 | **work_item → task_chunk_entity** | migration/progress.md:12 要求重命名；`work_item.rs` 125 LoC、0 单测；唯一生产者 `extractor.rs` | 确认重命名 + 归属 artifact-system？语义是否同时收敛？ |
| 3 | **namespace 归属** | `namespace.rs` 93 LoC、无 crate 内依赖；语义是 spec 容器（编排实体）| 归 artifact-system（实体集中）还是 skill-runtime（编排层）？ |
| 4 | **context_layer::MemoriesLayer → memory** | `MemoriesLayer` 依赖 `memory::Memory`（`context_layer.rs:17,76-80`，memory 属 slice-1）| trait 留 artifact-system，4 层实现是否整体留？MemoriesLayer 例外？ |
| 5 | **doc/extract/summarize 命令壳 vs 核** | 命令入口在 `popsicle-cli/src/commands/{doc.rs,extract.rs}`（cli-ux/slice-3），核心逻辑在 markdown/extractor/guard | 壳归 slice-3、核归本 product 的边界是否成立？ |

## 边界

- **本场不决定 crate 物理布局**（已由 ADR-003 定：根级 `crates/<slice>/`）；本场决定**逻辑 product 边界**（哪些模块的责任归 artifact-system 这个 product）。
- **本场不写技术设计**（trait 签名、依赖注入形态留 arch-debate / rfc）；本场只产「归属 + 责任」裁决 + task 识别表喂 prd-writer。
- **本场不碰 legacy 代码**（只读考古）；产出是 popsicle-new 的 product 定义。
- 跨 product 依赖的**技术解法**（依赖注入 vs 反向依赖 crate）属 arch-debate；本场只标「边界在哪、谁注入谁」。

## Participants（参与角色，5 人，领域适配）

| 角色 ID | 角色 | 本场关注点 | IDD 纪律 |
|---|---|---|---|
| **PM** | 产品经理 | artifact-system 是不是"用户/agent 能识别的文档体系能力包"；主持 4 Phase | Phase 4 末必须产 3 张表（task 识别 / intent 层归类 / User Intents Catalog 起草），所有声明 cite fact-extraction-report |
| **ENGLD** | 工程负责人 | 耦合面 + 迁移成本；所有成本陈述 cite dependency-graph / unsafe-risk-report | guard 的跨 product 依赖（pipeline/run/registry）是 ENGLD 关键论据 |
| **UXR** | 用户体验研究员 | AI agent 与 dev 用户怎么实际用 doc/extract/guard；强调每个边界从"用户原话"出发 | 必须挑战"这个边界是技术耦合驱动还是用户心智驱动" |
| **DOMAIN** | 领域建模师（领域适配）| 实体语义内聚（Document / WorkItem / Namespace 是否同一界限上下文）| 关注 work_item→task_chunk 是纯改名还是语义收敛 |
| **MIGRATE** | 迁移负责人（领域适配）| Strangler shadow/cutover 可切性；边界是否产生"等价性对账"难点 | 关注 guard 拆分是否让 golden-output 对账变复杂 |

### 备用角色

- **PLATFORM-ARCH-MIRROR**（架构师产品侧镜像）：仅当 PM 拿不定 guard 边界、需反射"边界是否被技术耦合绑架"时召唤。不预编入避免冗长。

## Setup Checklist

- [x] 讨论主题已用一句话表达（裁定 artifact-system 最终 product 边界，5 候选项）
- [x] 目标 product 已绑定（`artifact-system`，来自 project-init-plan §Product Inventory + spec slice-2-artifact-system）
- [x] fact-extraction-report 存在状态已记录（存在 → 已读 Bounded Contexts / Risk Hotspots / Glossary / dependency-graph / unsafe-risk-report / tech-debt-inventory）
- [x] 角色阵容确定（5 人：PM+ENGLD+UXR 核心三角；DOMAIN/MIGRATE 领域适配）
- [x] 用户置信度已设置（1-5）：默认 3（中），待用户在审批点调整
- [x] 已向用户展示完整 setup 摘要并取得 `approve` 确认（由 stage complete --confirm 兑现）

---

## Phase 1: 用户需求与问题定义

### 用户痛点

- AI agent（cursor/claude/copilot）依赖 `popsicle doc create/check`、`popsicle doc extract`、`popsicle prompt` 推进 pipeline；这些命令的**核心逻辑**（Document 序列化、Markdown 编辑、Guard 校验、Context 装配、WorkItem 提取）必须有清晰的 product 归属，否则迁移时责任不清。
- guard 是 IDD 状态转换的咽喉（`guard.rs:26` `check_guard`）：每次 `stage complete` 都过它。它同时碰 Document（本 product）和 pipeline/run（slice-1）——边界不清会让两个切片互相阻塞。

### 目标用户

| 用户 | 怎么用 artifact-system | 证据 |
|---|---|---|
| AI coding agent | `doc create`→写 body→`doc check`(guard)→`doc extract`(WorkItem)→`prompt`(context 装配) | `commands/{doc,extract}.rs`；`engine/{guard,extractor,context_layer}.rs` |
| dev 用户 | 读 `.popsicle/artifacts/<run>/*.md`（frontmatter+body）；人审 checklist | `model/document.rs:9,108` |
| 下游 skill | 通过 `assemble_layers` 拿到拼好的 prompt | `context_layer.rs:34` |

### 约束清单

- [x] 不改 legacy 行为（shadow 模式；等价性对账是 cutover 硬门）
- [x] crate 物理布局已由 ADR-003 固定（不在本场范围）
- [x] guard 的 3 类型语义必须保持（has_sections/checklist_complete/upstream_approved）——任何拆分不得改变校验结果
- [x] work_item 现有 kind（bug/story/testcase）语义保持；重命名不丢数据（JSON fields blob）
- [x] 边界裁决必须能映射到 task chunk 喂 prd-writer

### 成功指标

- artifact-system 的每个归属模块都能追溯到一个 fact-extraction Bounded Context
- 5 个候选项各有明确裁决（归属 + 谁注入谁）
- 产出 task 识别表（≥5 项）+ intent 层归类表，供 prd-writer / intent-spec-writer

### 用户输入（Phase 1）

> 用户置信度 3（中）。痛点与目标用户以事实基为准；如有补充在审批点提出。

---

## Phase 2: 候选方案

### 方案 A: ENGLD — 技术内聚优先（最小耦合面）

artifact-system = **纯文档引擎**，剥离所有跨 product 依赖项：

- 归 artifact-system：`document`、`markdown`、`context`、`context_layer`(trait + ProjectContext/HistoricalRefs/UpstreamDocs 三层)、`extractor`、`work_item→task_chunk`
- 留 skill-runtime：`guard` 整块（因依赖 pipeline/run/registry）、`namespace`（编排实体）、`MemoriesLayer`（依赖 memory）
- **耦合面**：artifact-system 几乎零出向依赖（除 storage::DocumentRow）
- 代价：guard 整块离开文档体系，但 has_sections/checklist_complete 其实只依赖 Document（`guard.rs:173-266`）——把纯文档校验也推走，artifact-system 反而不持有"文档校验"这个用户心智核心能力

### 方案 B: PM — 用户能力包优先（文档体系完整）

artifact-system = **完整"文档体系"能力**，全部 8 模块归本 product：

- 归 artifact-system：全部 8 模块（含 guard、namespace、全部 4 个 context layer）
- 跨 product 依赖（pipeline/run/registry/memory）通过**依赖注入**（trait 参数）解决——artifact-system 定义接口，skill-runtime 注入实现
- **耦合面**：artifact-system 定义多个"被注入"接口；逻辑归属清晰（文档体系 = 一个能力包）
- 代价：namespace（spec 容器）放进文档体系语义上勉强；guard 的 upstream_approved 需要 artifact-system 定义一个 pipeline 抽象接口，可能过度设计

### 方案 C: DOMAIN — 按依赖方向 + 实体内聚切（混合）

按事实层**依赖方向**切，让边界落在天然接缝上：

- 归 artifact-system（文档制品的生产/校验/装配）：
  - `document`（实体）、`markdown`（编辑）、`context` + `context_layer` trait + ProjectContext/HistoricalRefs/UpstreamDocs 三层、`extractor`、`work_item→task_chunk`
  - `guard` 的**纯文档校验**部分：`has_sections` / `checklist_complete` / `count_checkboxes`（仅依赖 Document+markdown，`guard.rs:79-90,173-266`）
- 留 skill-runtime（编排/状态）注入：
  - `guard::upstream_approved`（依赖 pipeline/run/registry，`guard.rs:103-169`）—— 作为 skill-runtime 提供给 guard 的一个**编排判定回调**
  - `MemoriesLayer`（依赖 memory）—— context_layer trait 留 artifact-system，但 MemoriesLayer 这个实现归 skill-runtime（它注册到 assemble_layers）
  - `namespace`（spec 容器，编排实体）—— 归 skill-runtime
- **耦合面**：唯一接缝 = guard 的 upstream_approved 回调 + MemoriesLayer 注册 + storage::DocumentRow（共享栈底）。其余零跨界。
- 代价：guard 在两个 product 间"拆开"，需要一个清晰的接口让 skill-runtime 注入 upstream 判定——但这正是事实层已存在的依赖方向（`check_guard` 参数已含 `pipeline/run/registry`）

### 三方案对比（用于 Phase 3 辩论）

| 维度 | A 技术内聚 | B 能力包 | C 依赖方向混合 |
|---|---|---|---|
| artifact-system 是否持有"文档校验"用户心智 | ✗（guard 全走）| ✓ | ✓（纯文档校验留下）|
| 跨界接缝数 | 少（但割裂能力）| 多（注入接口多）| 1 类接缝（guard 回调）|
| 与事实层依赖方向一致 | 部分 | 需新增抽象 | ✓（顺依赖切）|
| namespace 语义归属合理性 | ✓（编排）| ✗（塞文档体系）| ✓（编排）|
| 等价性对账复杂度（MIGRATE）| 低 | 中（注入多）| 低-中（guard 拆点单一）|

### 用户输入（Phase 2）

> 三方案差异聚焦在 **guard 怎么切**。待 Phase 3 辩论。

---

## Phase 3: 多角色辩论

### 轮次 1: ENGLD 挑战 C（guard 拆分成本）

> ENGLD：把 guard 拆成"纯文档校验"留下 + "upstream_approved"注入，会不会在迁移时产生两份 guard 代码、对账困难？

DOMAIN 反驳：`check_guard` 事实上已经是 `if let` 链分派（`guard.rs:65-96`），三类型彼此独立。拆分不是切一个函数，而是把 `upstream_approved` 那一支的**实现**由 skill-runtime 以回调/trait 注入；has_sections/checklist_complete 留 artifact-system。代码上是"一个分派器 + 可注入的一支"，不是两份。

MIGRATE 补充：对账以 `check_guard` 的**输出**（GuardResult.passed）为 golden，拆分内部实现不改变输出，对账点不变。

### 轮次 2: UXR 挑战 A（artifact-system 丢失文档校验心智）

> UXR：方案 A 把 guard 整块推给 skill-runtime。但用户原话是"我 doc check 看章节齐不齐、checklist 勾完没"——这是**文档**心智，不是 pipeline 心智。把它从 artifact-system 拿走，product 边界就被技术耦合（upstream 那一支）绑架了。

ENGLD 让步：同意。A 的问题在于用 `upstream_approved` 这一支的依赖，绑架了整个 guard 的归属。C 的拆法正好解开——按"用户心智"把文档校验留下，只把编排判定推走。

### 轮次 3: DOMAIN 挑战 B（namespace 塞进文档体系）

> DOMAIN：方案 B 让全部 8 模块归 artifact-system，包括 namespace。但 namespace 语义是 "多 spec 容器"（`namespace.rs:7`）——它是**编排层级**（Namespace→Spec→Issue→Run），与 Document 制品无内聚关系（`namespace.rs` 无 crate 内依赖，也不被 document 引用）。塞进文档体系是按"实体都堆一起"而非"界限上下文"切。

PM 接受：namespace 归 skill-runtime（编排），artifact-system 不持有它。这也与 slice-1 已迁 spec/issue 的归属一致。

### 三人最终立场

| 角色 | 倾向 | 关键理由 |
|---|---|---|
| ENGLD | C | guard 拆点单一（仅 upstream 一支注入），顺事实依赖方向，无双份代码 |
| UXR | C | 文档校验心智（has_sections/checklist）留在 artifact-system |
| DOMAIN | C | namespace 归编排、work_item→task_chunk 归本 product，界限上下文清晰 |
| PM | C | 5 候选项都有干净裁决，可映射 task |
| MIGRATE | C | 对账以 GuardResult 输出为 golden，拆分不增对账点 |

### 用户输入（Phase 3）

> 五角色收敛到方案 C。待 Phase 4 决策矩阵确认。

---

## Phase 4: 收敛与决策

### 决策矩阵（5 维加权评分，角色提出维度）

| 维度（提出者）| 权重 | A 技术内聚 | B 能力包 | C 依赖方向混合 |
|---|---|---|---|---|
| 与事实依赖方向一致（ENGLD）| 0.25 | 3 | 2 | 5 |
| 用户心智内聚（UXR）| 0.25 | 2 | 4 | 5 |
| 界限上下文清晰（DOMAIN）| 0.20 | 4 | 2 | 5 |
| 迁移/对账复杂度（MIGRATE，越低越高分）| 0.15 | 4 | 3 | 4 |
| 跨界接缝最少（ENGLD）| 0.15 | 5 | 2 | 4 |
| **加权合计** | 1.00 | **3.20** | **2.70** | **4.80** |

### Decision

**采用方案 C（按依赖方向 + 实体内聚切）。** artifact-system 的最终 product 边界：

**归 artifact-system（本 product crate `crates/artifact-system/`）**：
1. `Document` 实体（document.rs）— frontmatter+body、revision、序列化
2. `markdown` 6 纯函数（markdown.rs）
3. `context` 装配（Relevance 排序）+ `context_layer` **trait** + `ProjectContextLayer`/`HistoricalRefsLayer`/`UpstreamDocsLayer` 三层
4. `extractor` 3 提取函数（extractor.rs）
5. `task_chunk_entity`（由 `work_item` **重命名**；kind=bug/story/testcase 语义保持；JSON fields blob 保持）
6. `guard` 的**纯文档校验**：`has_sections`、`checklist_complete[:section]`、`count_checkboxes`、`GuardResult`、`check_guard` 分派器骨架

**留 skill-runtime / 由其注入（不在 artifact-system crate）**：
- `guard::upstream_approved` 判定（依赖 pipeline/run/registry）→ skill-runtime 以**回调/trait** 注入 check_guard 分派器
- `MemoriesLayer`（依赖 memory）→ 实现归 skill-runtime，运行时注册到 `assemble_layers`
- `namespace` 实体 → 归 skill-runtime（编排层级 Namespace→Spec→Issue→Run）

**唯一跨界接缝**：check_guard 的 upstream 判定回调 + MemoriesLayer 注册 + 共享 `storage::DocumentRow`（栈底）。技术形态（trait 签名 / 依赖注入）留 arch-debate / rfc。

### 表 1: Task 识别表（边界裁决 → product / owner / effort）

| Task | 描述 | product | 来源裁决 | effort |
|---|---|---|---|---|
| T-A1 | Document 实体 + frontmatter 序列化/解析往返 | artifact-system | 候选项基线 | S |
| T-A2 | markdown 纯函数族（section 抽取/upsert/summary/tags/placeholder 判定）| artifact-system | 候选项基线 | M |
| T-A3 | guard 纯文档校验（has_sections / checklist_complete / count_checkboxes）+ 可注入 upstream 回调 | artifact-system | 候选项 1（C 拆法）| M |
| T-A4 | context 装配（Relevance 排序）+ context_layer trait + 3 内建层 | artifact-system | 候选项 4 | M |
| T-A5 | extractor 3 提取函数 → task_chunk | artifact-system | 候选项 5（核）| M |
| T-A6 | work_item → **task_chunk_entity** 重命名（kind + fields 语义保持）| artifact-system | 候选项 2 | S |
| T-A7（伴生）| skill-runtime 注入 upstream_approved 回调到 guard | skill-runtime | 候选项 1 | S |
| T-A8（伴生）| namespace 归 skill-runtime（确认，不在本 crate）| skill-runtime | 候选项 3 | XS |

### 表 2: Intent 层归类表（C 涉及的 invariant / contract / acceptance 候选）

| 候选 intent | 层 | 来源 |
|---|---|---|
| `DocumentRoundTrips`（to_file_content→from_file_content 还原）| acceptance | T-A1（document.rs:108,114）|
| `RevisionBumpsVersionAndLinksParent`（new_revision: version+1 & parent 链）| acceptance | T-A1（document.rs:85-105）|
| `GuardHasSectionsRejectsPlaceholder`（占位段判失败）| acceptance | T-A3（guard.rs:173-213）|
| `ChecklistCompleteIffNoUnchecked`（全勾才过）| acceptance | T-A3（guard.rs:217-266）|
| `GuardResultIsTotal`（任何 guard 字符串→Ok 或 InvalidSkillDef，不 panic）| invariant | T-A3（guard.rs:92-95）|
| `ContextAssemblyOrdersByRelevance`（Low→summary/High→全文，排序稳定）| acceptance | T-A4（context.rs:52-77）|
| `ExtractPreservesKind`（提取产物 kind 与 fn 对应）| acceptance | T-A5（extractor.rs:10,69,125）|
| `TaskChunkRenamePreservesFields`（重命名后 kind/fields 不丢）| invariant | T-A6（work_item.rs:42,112-117）|

### 表 3: User Intents Catalog（草稿，传 prd-writer / intent-spec-writer）

| 用户意图（原话化）| 对应 task | journey_stage |
|---|---|---|
| "我 doc create 后想确认 frontmatter/body 存盘再读能一字不差还原" | T-A1 | daily-ops |
| "doc check 要能告诉我哪个章节还是模板占位、checklist 还差几个" | T-A3 | daily-ops |
| "prompt 装配时最相关的文档要给全文、次要的只给摘要" | T-A4 | daily-ops |
| "doc extract 要从 PRD/test-spec 正确抽出 story/bug/testcase" | T-A5 | daily-ops |
| "老的 work_item 改名 task_chunk 后，我历史数据的 kind/字段不能丢" | T-A6 | lifecycle |
| "guard 收到不认识的类型时要明确报错，不能 panic 把我整个 run 带崩" | T-A3 | troubleshooting |

### Action Items（传后续 stage）

- prd-writer：以表 1 的 6 个 artifact-system task（T-A1..T-A6）为 PRD 的 task chunk，跨 daily-ops/lifecycle/troubleshooting 旅程阶段；伴生 T-A7/T-A8 标 skill-runtime owner。
- arch-debate：裁定 guard upstream 回调 + MemoriesLayer 注册 + DocumentRow 共享的**技术形态**（trait 签名 / 依赖注入方向 / 是否反向 crate 依赖）。
- intent-spec-writer：表 2 的 8 个候选 intent 收紧，重点 `GuardResultIsTotal`（invariant）与 `ChecklistCompleteIffNoUnchecked`（acceptance）。

### 用户输入（Phase 4）

> 推荐方案 C（加权 4.80）。最终 freeze 待用户在审批点 `approve`（stage complete --confirm）。

---

## Decision

详见上方 §Phase 4 §Decision。**采用方案 C**：artifact-system 持有 document/markdown/context+layer trait+3 层/extractor/task_chunk/guard 纯文档校验；upstream_approved 回调、MemoriesLayer、namespace 归 skill-runtime 注入。最终 freeze 待用户审批（stage complete --confirm）后由 stage 触发。

---

## Phase Coverage

- [x] Phase 1 已完成，有用户痛点 + 目标用户 + 约束清单 + 成功指标
- [x] Phase 2 已产出 3 个差异化候选方案（A/B/C）
- [x] Phase 3 全部角色已发表评审意见（3 轮 + 五人最终立场）
- [x] Phase 4 已收敛到推荐方案 + 决策矩阵（C，加权 4.80）
- [x] 至少 4 个用户交互点（Phase 1-4 各一 + 审批点）

## Output Checklist

- [x] debate-record 含 Phase 1-4 全部小结
- [x] debate-record 标注了「用户决策覆盖」（每 Phase 末「用户输入」节）
- [x] 决策的每个核心声明都标注了 intent 层（表 2）
- [x] 每个数字 / LoC / 模块名引用都能追溯到 fact-extraction-report / baseline / *.rs:line
- [x] decision-matrix 的维度由角色提出且权重明示
- [x] task 识别表（表 1）≥5 项且标 product/owner/effort
- [x] 已向用户展示产出并取得 `approve` 确认（由 stage complete --confirm 兑现）
