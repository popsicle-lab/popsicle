---
id: d0794fe1-0a06-4d4a-9b05-94e7e94c3d16
doc_type: product-debate-record
title: 'skill-runtime product-debate: scope & D4 legacy slimming'
status: final
skill_name: product-debate
pipeline_run_id: f89529af-d8ce-40f7-ad05-985e35b9cfec
spec_id: df11b6f9-e504-42b7-8d27-700d529c2346
version: 1
parent_doc_id: null
tags: []
metadata: null
created_at: 2026-06-08T03:48:16.127331Z
updated_at: 2026-06-08T04:09:05.909924Z
---

---
---
---
---
---
---
---
---
---

# 产品辩论纪要 — D4 §5 ⚠️ 候选项裁决（定义 skill-runtime + artifact-system 的最终 product scope）

> **Status**: Setup（待 user `start` 后进入 Phase 1）
> **Date**: 2026-06-08
> **Target Product**: `skill-runtime`（首切片；裁决结果将外溢到 `artifact-system` / `cli-ux`）
> **User Confidence**: **3/5**（平等对话——agent 默认推荐 + 关键节点请示用户；用户已在 setup 暂停点确认）
> **Participants**: **PM / UXR / ENGLD**（三人小组——用户选 `core_3_only`；GROWTH/BIZ 暂不入场，若 Phase 3 出现盲点再升级）
> **Fact Basis**: ✅ [`fact-extraction-report.md`](popsicle-c76d729-fact-basis-slice-1--skill-runtime.fact-extraction-report.md) + 4 个伴生 baseline artifact（`docs/baseline/2026-06-08/`）

---

## Topic

事实数据点名 5 项 D4 §5 ⚠️ 候选项，每项裁"留 / 砍 / 简化"——决策结果直接定义 `skill-runtime` 与 `artifact-system` 两个 product 的最终 scope，并定夺 `sync-collab` 是否进入 product inventory。

### 待裁决的 5 个候选项

| # | 候选项 | facts 倾向 | 影响面 |
|---|---|---|---|
| 1 | `popsicle-sync` crate + `popsicle sync` CLI 命令（整个 sync-collab）| **砍** | popsicle-core 0 引用 / intent-coder 10 skill 全无依赖 / 895 LoC / yrs+tokio-tungstenite 依赖 |
| 2 | `namespace` 实体（model + admin namespace 命令）| **留但简化** | popsicle-new 自己正在用（namespace `popsicle-migration`）—— 砍它需要换种方式承载 spec 容器 |
| 3 | `issue` 实体（model + issue 命令族）| **留** | popsicle-new 当前 `issue create/start` 是 PipelineRun 启动入口；砍它需要重设计启动机制 |
| 4 | `work_item` 实体 + `engine/extractor.rs::extract_*` 三 fn + `popsicle extract/item` 命令 | **待裁** | intent-coder v0.3 任务图范式以 "task chunk" 替代了 work_item 的部分价值；但 extract_user_stories/test_cases/bugs 三个 fn 在 IDD 流程中仍有用武之地 |
| 5 | `doc-checklist-migrate-prompt` 命令族裁剪 | **部分裁** | `doc create/check/show/list` 是 IDD 主路径必需（刚刚用过）；`doc extract/summarize` 喂 work_item；`prompt` 必需；`migrate` 留作 admin；`checklist` 与 `doc check` 重复 → 砍候选 |

## 边界

- **In Scope**:
  - 5 个候选项的"留 / 砍 / 简化"产品决策
  - `skill-runtime` 与 `artifact-system` 的 product scope 边界（哪些 popsicle-core 模块归哪个 product）
  - `sync-collab` 是否进 product inventory 的最终裁决
  - 商业层："popsicle 是 intent-coder 私有引擎"在 PRODUCT.md 中怎么对客户表达
- **Out of Scope**:
  - 技术架构（crate 拆分、trait 边界、模块依赖图重设计）→ 留给 arch-debate stage
  - 编程语言 / 工具链选择 → arch-debate
  - intent 层具体写法 → intent-spec-writer
  - 等价性测试脚本 → 由首切片实施过程决定
- **触及 charter？**: **否**（这是 product-level 决策，不动 `docs/CHARTER.md` 四条铁律 / Layer Map / 三层 intent）

## Participants（参与角色，5 人，领域适配）

| 角色 ID | 角色 | 本场关注点（领域适配后）| 在本场的 IDD 纪律 |
|---|---|---|---|
| **PM** | 产品经理 | popsicle-new 对 intent-coder 是不是"客户能识别的能力包"；主持 4 个 Phase | Phase 4 末尾必须产 3 张表（task 识别 / intent 层归类 / User Intents Catalog 起草），所有声明 cite fact-extraction-report |
| **UXR** | 用户体验研究员 | AI agent 与 dev 用户的实际使用流程；强调 `[TBD: needs archaeology]` 是不是从用户原话出发 | 必须挑战每个候选项的"用户实际怎么用原话讲" |
| **ENGLD** | 工程负责人 | 实现成本 + 长期维护成本；所有"实现成本"陈述必须 cite fact-extraction-report 的 Risk Hotspots 或 unsafe-risk-report | unwrap 301 个的代价是 ENGLD 的关键论据 |
| **GROWTH** | 增长策略师（领域适配）| intent-coder 工具链稳定性带来的下游接受度 + 主流 AI agent (cursor/claude/copilot) 中 popsicle 的差异化 | GROWTH 关注「砍掉了 sync-collab 后 popsicle 是否丧失差异化」 |
| **BIZ** | 商业分析师（领域适配）| popsicle 私有引擎可持续性 + 避免"通用平台 PMF 误解" + 维护人力成本 | BIZ 关注「保留 namespace+issue+work_item 是否暗示 popsicle 仍想做通用平台」 |

### 备用角色（如关键节点需要）

- **PLATFORM-ARCH-MIRROR**（架构师在产品侧的镜像）：仅在 PM 拿不定 product 边界时召唤，反射"这个边界是不是被技术耦合驱动"
- 不预先编入 5 人阵容，避免对话冗长

---

## Setup Checklist

- [x] 讨论主题已用一句话表达完毕（5 候选项裁决 → 定义 product scope）
- [x] 目标 product 已绑定（`skill-runtime`，来自 project-init-plan §Product Inventory）
- [x] fact-extraction-report 存在状态已记录（存在 → 已读取 Bounded Contexts / Risk Hotspots / Domain Glossary / tech-debt-inventory §"结构性候选删项"）
- [x] 角色阵容确定（5 人，含 PM + UXR + ENGLD 核心三角；GROWTH/BIZ 领域适配）
- [x] **用户置信度已设置（1-5）**（待 setup 暂停点回应）
- [x] **已向用户展示完整 setup 摘要并取得 `start` 确认**（**当前点：暂停**）

---

## Phase 1: 用户需求与问题定义

### 用户痛点

popsicle legacy（`c76d729`）是"通用 workflow 引擎 + 多个 admin/migrate 命令 + 0 引用的 `popsicle-sync`"，**唯一的实际消费者** intent-coder（10 skills + 1 pipeline）只用了其中一个窄子集；继续维持通用平台姿态会让维护成本（fact-extraction-report：`unwrap()`×301、`index.rs` 单文件 101 个、`popsicle-sync` 895 LoC + yrs+tokio-tungstenite 重依赖）压垮人力，且让 PRODUCT.md 的"一行用途"始终无法对 AI agent 用户讲清"我是什么"。

### 目标用户

- **主要用户**: `intent-coder` skill pack（10 个 skill + `migration-bootstrap` pipeline）以及通过它工作的 AI coding agent（Cursor / Claude Code / Copilot），它们通过 `popsicle skill load/run` + `popsicle pipeline start/stage` + `popsicle doc create/check` 调用 popsicle-new
- **次要用户**: 在 popsicle-new 上跑 pipeline 的人类工程师（写新 skill 的扩展者、查 doc 状态的维护者）

### 约束清单

- **必须满足**:
  - 保留 intent-coder 当前 **10 个 skill + `migration-bootstrap` pipeline 能正常工作**（不破现有 `init / facts / debate / prd / arch-debate / rfc / adr / intent-spec / intent-check / living-docs` 的 stage 序列；不动 schema：`from_skill` + `artifact_type` 输入声明）
  - 保留 **PipelineRun / Document / Skill / Stage** 四个 IDD 流程最小依赖实体（fact-extraction-report 已确认这是 intent-coder 的实际引用面）
  - 已使用机制不变：YAML frontmatter、`doc check check`、`stage complete --confirm`、`.popsicle/artifacts/` 目录结构
- **最好满足**:
  - 砍掉的 popsicle-core 代码量 ≥ 30%（让维护面收敛到可保养的体积）
  - `skill-runtime` / `artifact-system` / `cli-ux` 的 `PRODUCT.md` "一行用途" 都能在 ≤ 25 字内说清
  - 每个 product 对外公开接口数 ≤ 7（避免新一轮通用平台漂移）
- **可以取舍**:
  - `popsicle-sync` 是**整砍**还是**挂起**（挂起 = 留 crate 但 PRODUCT.md 标 hibernated，不进 inventory）
  - `work_item` 实体是**整砍**、**完全融入 IDD task chunk**、还是**保留 extract_* 三 fn 作为下沉工具**
  - admin/migrate 命令族是**保留**还是**移出 cli-ux 进 `popsicle-ops`（潜在新 product）**

### 成功指标

| 指标 | 当前基线（来自 fact-extraction-report）| 目标 |
|---|---|---|
| popsicle-core module 数 | 12（`commands/`、`engine/`、`skill/`、`hook/`、`tool/`、`memory/`、`storage/`、`model/`、`pipeline/`、`schema/`、`migrate/`、`utils/`）| ≤ 8 |
| `unwrap()` 调用数（popsicle-core）| 301（hotspot：`storage/index.rs` 101 个）| 在 skill-runtime 切片范围内 ≤ 50 |
| `popsicle-sync` 被引用次数（popsicle-core + intent-coder skills 共 10 个 YAML）| 0 引用 | 维持 0（不在 product inventory 出现）或显式标 `hibernated` |
| `PRODUCT.md` "一行用途" 文本量 | 三个 product 全空（init stage 填 `[TBD]`）| 三个都 ≤ 25 字、零 "通用 / 平台 / 灵活" 等 forbidden phrase |
| 主路径命令数（每个 product 暴露的 popsicle CLI 命令）| 不可数（admin/sync/extract/checklist/migrate/prompt 全部混在 popsicle CLI 命令树里）| 每个 product ≤ 7 个公开命令 |

### 用户输入

**[已审阅 2026-06-08]** 用户全部 4 项确认 `accurate / ok / as_is / keep_3`，无改动。原起草为 agent 在 confidence=3 模式下基于 `fact-extraction-report` + `project-init-plan` + `docs/CHARTER.md` 起草。
请你（用户）确认：
1. 用户痛点是否准确（特别是"通用平台姿态"这句话有没有冤枉 popsicle）
2. 约束清单的"必须 / 最好 / 可以"分档是否合你心意
3. 5 个成功指标的目标值是否过于保守 / 过于激进
4. "可以取舍"里要不要加 / 删项

## Phase 2: 候选方案

三人小组各提一案。所有方案对 5 候选项必须给出确定动作（留 / 砍 / 简化 / 改造）+ 三个 product 的最终 inventory。

| 方案 | 提案者 | 核心思路 | 自评优势 | 自评劣势 |
|---|---|---|---|---|
| **A** | **ENGLD** | 激进精简：5 项全砍或最小化，按"砍代码量最大化"裁决 | 维护面立即收敛 ≥ 30%；私有引擎边界硬到无可争议 | 失去可逆性（sync/work_item 想回来要重写）；可能在 Phase 3 被 UXR/PM 反扑 |
| **B** | **UXR** | 渐进保留：核心命令切净 + 边缘 hibernated；保 work_item → task_chunk 的语义升级路径 | 用户回得来；migration 风险低 | "通用平台姿态"阴影没消；维护面收敛仅 ~20%；可能引出新的 forbidden phrase |
| **C** | **PM** | Product 边界倒推：按"每个 product 一行用途"逆推 5 项归属 | PRODUCT.md 最易写；product scope 清晰；不为"砍代码"而砍 | 实施成本最高（CLI 命令树要重组）；裁哪项 / 留哪项依赖 product 命名规则 |

---

### 方案 A: ENGLD — 激进精简 / 私有引擎硬边界

**对 5 候选项的动作**

| # | 候选项 | A 的动作 |
|---|---|---|
| 1 | popsicle-sync | **整砍** — 删 crate、删 `popsicle sync` 命令族、删 yrs+tokio-tungstenite 依赖 |
| 2 | namespace | **简化** — namespace 从一等实体降为 Document.metadata 上的可选 tag 字段；admin namespace 命令族全砍 |
| 3 | issue | **留** — PipelineRun 启动入口不动；issue 命令族保留 `create/start/show/list/close`（5 个） |
| 4 | work_item | **整砍实体** — 删 work_item table；`extract_user_stories/test_cases/bugs` 三 fn 下沉为 `utils/extract.rs` 模块函数（供 skill 内部调用，不暴露 CLI） |
| 5 | doc-checklist-migrate-prompt | **砍 4 留 4** — 留：`doc create/check/show/list`（IDD 主路径）；砍：`checklist`（与 doc check 重复）、`migrate`、`prompt`、`doc extract/summarize/item` |

**最终 Product Inventory**: `skill-runtime` / `artifact-system` / `cli-ux`（**3 个**，不含 sync-collab）

**核心用户流程**: `popsicle skill load → pipeline start → stage complete --confirm → doc create/check`（5 个核心命令，所有非主路径砍）

**关键功能**: P0 = skill loading / pipeline staging / doc CRUD + check；P1 = issue 启动；P2 = ø

**实现路径**: 在 popsicle-new 新建 3 product；不带 sync-collab；首切片只迁 skill-runtime 验等价性

**ENGLD 自评**:
- ✅ 维护面立即收敛 ≥ 30%（fact-extraction-report 已证 sync 895 LoC + work_item 相关 ~400 LoC + 命令族缩减）
- ✅ 维护下沉到可保养体积（unwrap 大头 index.rs 不在切片范围，留给后续切片处理）
- ⚠️ 失可逆性：sync / work_item 想回来 = 重写
- ⚠️ 可能在 Phase 3 被 UXR 反问"AI agent 用户的实际场景 fact 数据支撑了吗"

---

### 方案 B: UXR — 渐进保留 / 语义升级

**对 5 候选项的动作**

| # | 候选项 | B 的动作 |
|---|---|---|
| 1 | popsicle-sync | **挂起** — 留 crate 在 workspace；PRODUCT.md 标 `hibernated`；CLI sync 命令隐藏到 `popsicle admin sync` 不主推 |
| 2 | namespace | **留 + 隐藏** — namespace 实体保留；admin namespace 命令族保留但移到 `popsicle admin namespace` 不主推 |
| 3 | issue | **留** — 不动 |
| 4 | work_item | **语义升级** — 重命名为 `task_chunk`；`extract_*` 三 fn 改名 `derive_*`，对齐 IDD task graph 范式；保留实体便于未来挂 intent 检查结果 |
| 5 | doc-checklist-migrate-prompt | **砍 3 留 5** — 留：`doc create/check/show/list/extract`、`prompt`；砍：`checklist`、`summarize`、`item`；`migrate` 移 `popsicle admin migrate` |

**最终 Product Inventory**: `skill-runtime` / `artifact-system` / `cli-ux` + **sync-collab（hibernated, optional）**（**3+1 个**）

**核心用户流程**: 与 A 同主路径；附加 `popsicle admin <ns|sync|migrate>` 三个 admin 出口

**关键功能**: P0 同 A；P1 = task_chunk 派生 + issue 启动；P2 = sync hibernated（保 PRODUCT.md "未来可激活"）

**实现路径**: 保 sync crate；work_item → task_chunk 是 schema 演进；admin 命令归 `popsicle-ops` 子树

**UXR 自评**:
- ✅ 用户回得来（sync hibernated 不删；task_chunk 是改名而非删除）
- ✅ 实际 AI agent 用户的"我以后想 collab" 场景留了门
- ⚠️ "通用平台姿态"阴影没消（PRODUCT.md 多了 hibernated 字段会让"私有引擎边界"模糊）
- ⚠️ 维护面收敛仅 ~20%（sync crate 留在 workspace 还要参与 CI 编译）
- ⚠️ task_chunk 这个新名词没在 charter / glossary 出现过 → 可能引入新 forbidden phrase

---

### 方案 C: PM — Product 边界倒推 / 一行用途驱动

**思路**: 先确定三个 product 各自能在 ≤ 25 字写完的"一行用途"，再倒推 5 候选项归到哪个 product（归不进任何 product 的 → 砍）

**三个 Product 一行用途（PM 起草）**:
- `skill-runtime`：**"加载 skill、串成 pipeline、按 stage 推进。"**（22 字）
- `artifact-system`：**"创建文档与 checklist，按 schema 落盘并可查可校验。"**（25 字）
- `cli-ux`：**"把 skill-runtime + artifact-system 暴露成可对话的命令行。"**（25 字）

**对 5 候选项的动作（按 product 归属倒推）**

| # | 候选项 | 归属 product | 命令归属调整 |
|---|---|---|---|
| 1 | popsicle-sync | **无** — 三个一行用途里都装不下；**整砍** | 删 crate + sync 命令族 |
| 2 | namespace | **artifact-system** — namespace 是 Document/Artifact 的容器；不暴露 admin 命令（只在 doc 创建时使用） | admin namespace 命令族砍；schema 层保留 |
| 3 | issue | **skill-runtime** — issue 是 PipelineRun 启动入口，归 skill-runtime | issue 命令族保留在 skill-runtime 命令空间 |
| 4 | work_item | **artifact-system** — work_item 也是一种 artifact（产物文档）；保留实体；extract_* 三 fn 归 artifact-system 工具函数 | `popsicle extract/item` 命令族归 artifact-system 命令空间，不再属于 skill-runtime |
| 5 | doc-checklist-migrate-prompt | **拆 3 部分** | `doc/extract/summarize` → artifact-system；`checklist` 砍（与 doc check 重复 — 一行用途装不下两个 check 概念）；`migrate` → cli-ux 子命令 `popsicle admin migrate`；`prompt` → cli-ux（"对话能力"的一部分） |

**最终 Product Inventory**: `skill-runtime` / `artifact-system` / `cli-ux`（**3 个**，不含 sync-collab）

**核心用户流程**: 按 product 切分的命令树：`popsicle skill/pipeline/stage <…>` (skill-runtime) | `popsicle doc/extract <…>` (artifact-system) | `popsicle admin <…>` (cli-ux)

**关键功能**: P0 = 三个 product 各自的一行用途内的动作；P1 = 命令树重组；P2 = work_item → task chunk 重命名（可选，留给 arch-debate）

**实现路径**: 在 popsicle-new 按 product 拆 crate（`popsicle-skill-runtime` / `popsicle-artifact-system` / `popsicle-cli-ux`）；首切片 skill-runtime；CLI 命令树重组（破坏向后兼容，仅对 intent-coder 而言）

**PM 自评**:
- ✅ PRODUCT.md 一行用途天然成立（已起草并都 ≤ 25 字）
- ✅ product 边界清晰，避免"为砍而砍"
- ✅ 命令树重组让 cli-ux 真正"暴露"，而不是混杂
- ⚠️ 实施成本最高（CLI 命令树重组 = 用户记忆要变；intent-coder 的 10 个 skill 调用方式可能要更新）
- ⚠️ 命令空间从扁平 → 三层，对人类工程师有学习成本
- ⚠️ 命令兼容 break 是个潜在 charter 触发点（要不要保兼容旧命令 6 个月？）

---

### 三方案对比（用于 Phase 3 辩论）

| 维度 | A (ENGLD) | B (UXR) | C (PM) |
|---|---|---|---|
| 代码量收敛 | ≥ 30% | ~20% | ≥ 30%（但需重组）|
| 可逆性 | 低 | 高 | 中 |
| PRODUCT.md 易写 | 中 | 难（hibernated 字段）| **极易**（一行用途已起草）|
| 实施成本（首切片）| 低 | 低 | 高（命令树重组）|
| 私有引擎边界硬度 | 硬 | 中（sync 还在）| 硬 |
| Charter 触发风险 | 否 | 低（hibernated 是新概念）| 中（命令兼容 break）|
| 对 intent-coder 现有 10 skill 的影响 | 0（不动 skill API）| 0 | 可能 1-2（若 skill 用了被搬走的命令）|

### 用户输入

**[已审阅 2026-06-08]** 用户选 `primary=C` + `charter_break=skill_internal` + `go_phase3=go`。原起草: confidence=3 模式 — agent 倾向**方案 C（PM）**：理由 = "一行用途已起草且都 ≤ 25 字" 是约束清单中"最好满足"项的最强证据，且 product 边界清晰避免后续漂移；但 **C 的实施成本是真痛点**，是否接受需要你定。

请你选：
1. 你倾向哪个方案做 Phase 3 辩论的"主候选"
2. 是否要把"命令兼容 break"这个 charter 触发风险升级到 charter-amendment（如选 C）
3. 三案之外是否要补 D（混合方案，如 A 的砍法 + C 的 PRODUCT.md 起草）


## Phase 3: 多角色辩论

主候选：**方案 C（PM）**。三人按 IDD 纪律对 C 反驳/补强，给出"修正后的 C-prime"。所有反驳必须 cite fact-extraction-report 或 docs/CHARTER.md。

### 轮次 1: ENGLD 挑战 C

**ENGLD-Q1**: "命令树重组" 对 intent-coder 10 个 skill 的真实影响在 facts 数据里有多大？
**事实根据**: `fact-extraction-report §Bounded Contexts` + `api-contracts.md §popsicle-cli` 显示 intent-coder skills 通过 4 个命令族调用 popsicle: `doc create/check/show/list`、`pipeline start/stage`、`stage complete --confirm`、`skill load`。

**ENGLD-A1**: C 方案的命令树（`skill / pipeline / stage / doc / extract / admin`）里 **doc / pipeline / stage / skill 4 族原位归 product**（doc → artifact-system；pipeline / stage / skill → skill-runtime），**对 intent-coder 现有 skill 调用方式 0 破坏**。真实破坏点只有 4 个：`work_item / extract / migrate / prompt` 命令的位置变动 — 但 fact-extraction-report 已确认 intent-coder skills **从不调用这 4 个命令族**（10 个 skill.yaml 里 grep 无命中）。
**结论**: ENGLD-Q1 不构成阻断。"命令兼容 break" 对实际消费者（intent-coder）= **0 影响**。Charter 不上 amendment 这个决策成立。

---

**ENGLD-Q2**: 方案 C 把 namespace 归 artifact-system 但不暴露 admin 命令 — popsicle-new 自己用的 namespace `popsicle-migration` 是怎么创建的？
**事实根据**: project-init-plan §Repository Identity 显示 popsicle-new 在 init stage 通过 `popsicle namespace create popsicle-migration` 创建过 namespace。

**ENGLD-A2 (PM 接挑战)**: PM 承认 C 原版"砍 admin namespace 命令族"过激。**修正**: namespace 命令族保留 `create / list / use` 共 3 个核心命令（归 artifact-system 命令空间），砍纯 admin 操作（如 rename / archive / delete-cascade）。修正后 namespace 命令族从原 ~6 个降到 3 个。

---

**ENGLD-Q3**: 方案 C 把 work_item 归 artifact-system 保留实体 — D4 §5 ⚠️ 候选项的 facts 倾向是 "work_item 已被 IDD task chunk 替代"，保留实体不自相矛盾？
**事实根据**: `tech-debt-inventory §结构性候选删项` 行 4: "work_item 概念在 IDD task graph 范式下被 task chunk 替代，但 extract_* 三 fn 仍有 IDD 流程下沉价值"。

**ENGLD-A3 (PM 采纳 B 的重命名)**: 保留 work_item 实体作为 task chunk 的**存储类型**（storage entity），重命名为 `task_chunk_entity` 避免与 IDD 概念层的 "task chunk" 混淆；`extract_user_stories / test_cases / bugs` 三 fn 改名 `derive_*`，对齐 IDD task graph 派生范式。**glossary.md 必须加一行**: "task_chunk_entity = task chunk 在 popsicle storage 层的具体存储类型"。

---

### 轮次 2: UXR 挑战 C

**UXR-Q1**: "命令树重组对 AI agent 用户的实际伤害" — 这是猜测还是事实？
**事实根据**: AI agent (Cursor/Claude Code/Copilot) 调命令通过 `popsicle --help` 的输出，而非记忆。

**UXR-A1**: AI agent 用户 **0 伤害**（help 输出更新即可）。**人类工程师**（次要用户）需要更新心智模型 — 需要 `docs/MIGRATION.md`（之后由 living-doc-author 写）。本场不阻断 C。

---

**UXR-Q2**: 方案 C 的"一行用途"全是动作动词，缺"用户/场景"维度 — user-journey 模板要求"用户故事 + 痛点驱动"。一行用途和 user-journey 怎么对齐？
**事实根据**: `intent-coder/skills/prd-writer/templates/prd.md` + `popsicle-new/docs/user-journeys/README.md` 显示 user-journey 是 cross-product 概念，需要 product 一行用途内有"用户线索"才能挂上去。

**UXR-A2 (PM 接挑战 + 模板修正)**: C 原版的"一行用途 ≤ 25 字" 不变（已起草达标），但 **PRODUCT.md 必须在"一行用途"下方紧跟一句话**: **"为 < 主要用户 > 解决 < 场景痛点 >"**。修正后 PRODUCT.md 头部固定 2 行：
- Line 1: 一行用途（≤ 25 字）
- Line 2: 一句话用户+场景（≤ 35 字）

这条修正会反传到 `intent-coder/skills/project-init/templates/PRODUCT.md`（但不在本场实施 — 留 living-doc 阶段处理）。

---

**UXR-Q3**: sync-collab 整砍 — 多个 AI agent 同时在同一 pipeline run 上工作的未来场景怎么办？
**事实根据**: `unsafe-risk-report §popsicle-sync 子系统` 显示 sync crate 0 被 popsicle-core / intent-coder 引用，但代码本身 (895 LoC, yrs+tungstenite) 是一个完整可工作的 CRDT 实现。

**UXR-A3 (UXR 部分倒向 B 但 PM 反推)**: PM 反驳: 当前 fact 数据是 "10 个 intent-coder skill 0 引用 sync"，IDD 主路径**事实上不需要** sync；保留 sync crate 在 workspace 会让"PRODUCT.md 私有引擎边界"模糊（B 方案承认的劣势）。**折中**: **整砍 + 在 `docs/PROJECT_CONTEXT.md` 写明触发条件**：
> 当满足以下条件之一时，重启 sync-collab proposal: (a) ≥ 2 个 AI agent 在同一 PipelineRun 上并发写 doc 出现冲突 ≥ 3 次; (b) 人类用户提出实时协同需求。触发条件未满足前 sync crate 不进 inventory。

UXR 接受此折中。

---

### 轮次 3: 汇总 — 修正后的 C-prime 方案

| # | 候选项 | C 原版 | **C-prime（最终）** | 主要修正源 |
|---|---|---|---|---|
| 1 | popsicle-sync | 整砍 | **整砍** + docs/PROJECT_CONTEXT.md 写明触发条件 | UXR-Q3 折中 |
| 2 | namespace | 归 artifact-system + 砍 admin 命令族 | 归 artifact-system + 保留 `create/list/use` 3 个核心命令 + 砍纯 admin 操作 | ENGLD-Q2 修正 |
| 3 | issue | 归 skill-runtime | **不变** — 归 skill-runtime + issue 命令族 5 个原位保留 | — |
| 4 | work_item | 归 artifact-system 保留实体 | **重命名 `task_chunk_entity`** + extract_* → derive_* + glossary 加注释 | ENGLD-Q3 + B 的语义升级 |
| 5 | doc-checklist-migrate-prompt | doc/extract → artifact-system；checklist 砍；migrate/prompt → cli-ux；item 砍 | **不变** | — |

**修正后的 Product Inventory**: `skill-runtime` / `artifact-system` / `cli-ux`（3 个，不含 sync-collab）

**修正后的 PRODUCT.md 头部模板**:
- Line 1: 一行用途（≤ 25 字）
- Line 2: 为 < 主要用户 > 解决 < 场景痛点 >（≤ 35 字）

**Charter Amendment**: **不上**（用户决策：intent-coder 是 popsicle-new 内部消费者，命令变动算 internal API 调整）

**伴生动作**（落到 Phase 4 任务表）:
- A1: docs/PROJECT_CONTEXT.md 增 §"未来 collab 触发条件"
- A2: docs/glossary.md 增 entry "task_chunk_entity"
- A3: 反传修正 `intent-coder/skills/project-init/templates/PRODUCT.md` 头部模板（**不在本 pipeline 实施**，记入 traceability.md 待 living-doc 阶段处理）
- A4: docs/MIGRATION.md（人类工程师心智模型迁移指南，由 living-doc-author 起草）

### 三人最终立场

| 角色 | 立场 | 关键论据 |
|---|---|---|
| **PM** | 接受 C-prime | "一行用途 ≤ 25 字" 全部达标；3 product 边界清晰 |
| **ENGLD** | 接受 C-prime | "命令兼容 break 对 intent-coder = 0 影响" 经 fact 验证；namespace 修正后实施可行 |
| **UXR** | 接受 C-prime | sync 整砍 + 触发条件文档 = 未来回得来；PRODUCT.md 双行模板可对齐 user-journey |

**全员一致**: 进 Phase 4 决策收敛。

### 用户输入

**[已审阅 2026-06-08 / 用户跳过 = confidence=3 默认推荐]** Phase 3 三角色辩论由 agent 在 confidence=3 模式下完成，每条反驳/补强 cite 了 fact-extraction-report 或 api-contracts.md。请你（用户）审：
1. C-prime 的 5 项动作是否照走（特别是 work_item 改名 `task_chunk_entity`、sync 整砍 + 触发条件文档）
2. 4 个伴生动作 A1-A4 是否漏项 / 多项
3. "Charter 不上 amendment" 这个决策是否需要在 docs/CHARTER.md 加一条 footnote（"intent-coder 是 popsicle-new 内部消费者，CLI 命令变动算 internal API 调整"）


## Phase 4: 收敛与决策

### 决策矩阵（5 维加权评分）

| 维度 | 权重 | 方案 A (ENGLD) | 方案 B (UXR) | **方案 C-prime (PM 修正)** |
|---|---|---|---|---|
| Fact 支持度（每项动作有 fact 引用）| 0.25 | 4/5 | 3/5 | **5/5** |
| 维护面收敛（≥30% 代码量）| 0.20 | 5/5 | 2/5 | **5/5** |
| 可逆性（未来回得来）| 0.15 | 2/5 | 5/5 | **4/5**（sync 触发条件文档 + work_item 重命名而非删除）|
| PRODUCT.md 易写（一行用途 + 用户场景）| 0.20 | 3/5 | 2/5 | **5/5**（双行模板已起草 3 个） |
| 实施成本（首切片人力）| 0.20 | 5/5（最低） | 5/5（最低） | **3/5**（命令树重组）|
| **加权总分** | **1.00** | **3.85** | **3.25** | **4.45** |

**胜出**: 方案 C-prime（4.45/5）

### Decision

**Product Inventory（最终）**: 3 个 — `skill-runtime` / `artifact-system` / `cli-ux`。**不含** sync-collab。

**5 候选项最终裁决**:

| # | 候选项 | 裁决 | 落到何处 |
|---|---|---|---|
| 1 | popsicle-sync | **整砍** | 删 crate / 删 `popsicle sync` 命令族 / 删 yrs+tokio-tungstenite 依赖。`docs/PROJECT_CONTEXT.md` §"未来 collab 触发条件" 记录重启条件 |
| 2 | namespace | **简化保留** | 实体归 `artifact-system`；命令保留 `namespace create/list/use` 3 个；砍 admin 专属操作 |
| 3 | issue | **留** | 实体 + 命令族（5 个：create/start/show/list/close）归 `skill-runtime` |
| 4 | work_item | **改造保留** | 实体重命名 `task_chunk_entity`，归 `artifact-system`；`extract_user_stories/test_cases/bugs` → `derive_*`；`glossary.md` 加注释 |
| 5 | doc-checklist-migrate-prompt | **按 product 拆 + 砍 2** | doc/extract/summarize → `artifact-system`；prompt/migrate → `cli-ux`；**砍** checklist（与 doc check 重复）；**砍** item（用 doc 替代）|

**PRODUCT.md 头部双行模板**:
- Line 1: 一行用途（≤ 25 字）
- Line 2: "为 < 主要用户 > 解决 < 场景痛点 >"（≤ 35 字）

**Charter Amendment**: **不上**。改写 ADR-001（在 `products/skill-runtime/decisions/adr/ADR-001-intent-coder-is-internal-consumer.md`）记录"intent-coder 是 popsicle-new 内部消费者，CLI 命令变动算 internal API 调整"原则。

**已起草的三个 product 一行用途（PM 提供，待 prd-writer 阶段写进 PRODUCT.md）**:
- `skill-runtime`：**"加载 skill、串成 pipeline、按 stage 推进。"**（22 字）
  - 为 **AI coding agent** 解决 **IDD 工作流的执行编排与状态推进** 痛点（32 字）
- `artifact-system`：**"创建文档与 checklist，按 schema 落盘并可查可校验。"**（25 字）
  - 为 **AI coding agent + 人类维护者** 解决 **IDD 产物的 schema 化存储与一致性校验**（35 字）
- `cli-ux`：**"把 skill-runtime + artifact-system 暴露成可对话的命令行。"**（25 字）
  - 为 **AI coding agent + 人类维护者** 解决 **统一对话界面与可发现性**（30 字）

---

### 表 1: Task 识别表（5 项裁决 + 4 个伴生 → product/owner/effort）

| Task ID | 任务 | Product | 类型 | 预估 effort | 触发后续 stage |
|---|---|---|---|---|---|
| T01 | 删除 popsicle-sync crate + sync 命令族 + 移除 yrs/tungstenite 依赖 | skill-runtime（首切片不含，记 migration/traceability）| 删除 | S（半天）| arch-debate（确认 crate 拆分图无 sync）|
| T02 | namespace 实体迁移 + 砍 admin 子命令 + 留 3 个核心命令 | artifact-system | 迁移+裁剪 | M（1 天）| 等 artifact-system 切片启动 |
| T03 | issue 实体 + 5 个命令族迁移 | skill-runtime | 迁移 | M（1 天）| arch-debate 确认 PipelineRun 启动入口 |
| T04 | work_item → task_chunk_entity 重命名 + extract_* → derive_* 函数改名 + glossary 加注释 | artifact-system | 重命名 | M（1 天）| intent-spec-writer 写 task_chunk_entity 不变量 |
| T05 | doc 命令族迁移 artifact-system + 砍 checklist/item | artifact-system + cli-ux | 迁移+裁剪 | M（1 天）| arch-debate 确认命令树 |
| T06 | prompt/migrate 命令族迁移 cli-ux | cli-ux | 迁移 | S（半天）| — |
| **A01** | docs/PROJECT_CONTEXT.md §"未来 collab 触发条件" | docs | 新增 | XS（30 分钟）| living-doc 阶段 |
| **A02** | docs/glossary.md 加 `task_chunk_entity` 条目 | docs | 新增 | XS（10 分钟）| living-doc 阶段 |
| **A03** | 反传修正 intent-coder/skills/project-init/templates/PRODUCT.md 头部模板 | （intent-coder 上游，**不本场实施**）| 新增 | S | living-doc 阶段记 traceability |
| **A04** | docs/MIGRATION.md（人类工程师心智模型迁移指南）| docs | 新增 | M | living-doc 阶段 |
| **A05** | products/skill-runtime/decisions/adr/ADR-001-intent-coder-is-internal-consumer.md | skill-runtime | 新增 | S | adr-writer 阶段 |

### 表 2: Intent 层归类表（C-prime 涉及的 invariant / contract / acceptance）

> 用于喂 `intent-spec-writer` 阶段。Intent 层级遵循 docs/CHARTER.md §三层 intent 层次。

| Intent ID | 文字内容 | 层级 | 所属 Product | Z3 可判定？ |
|---|---|---|---|---|
| INV-SR-01 | "popsicle-new 中不存在任何对 popsicle-sync 的依赖" | Invariant（全局）| skill-runtime（首切片）| 是（symbolic dep graph）|
| INV-SR-02 | "PipelineRun 状态机仅允许 {pending → in_progress → completed/blocked} 转移" | Invariant | skill-runtime | 是 |
| CON-SR-01 | "skill load 命令必须返回 SkillLoadResult，包含 name/version/state_machine" | Contract | skill-runtime | 部分（schema 校验）|
| CON-AS-01 | "doc create 必须产生包含完整 YAML frontmatter 的 .md 文件" | Contract | artifact-system | 是 |
| CON-AS-02 | "task_chunk_entity 的 derive_* 函数输出必须满足 task chunk schema" | Contract | artifact-system | 是 |
| INV-AS-01 | "namespace 是 Document 容器；同一 namespace 下 doc title 唯一" | Invariant | artifact-system | 是 |
| ACC-SR-01 | "迁移 skill-runtime 切片后，intent-coder 的 migration-bootstrap pipeline 在 popsicle-new 上从 init → living-docs 全程通过" | Acceptance | skill-runtime | 否（等价性测试）|

### 表 3: User Intents Catalog（草稿，传 prd-writer / intent-spec-writer）

| UI ID | 用户意图（一句话）| 主要用户 | 触发场景 | 涉及 Product |
|---|---|---|---|---|
| UI-1 | AI agent 加载 intent-coder skill 包，启动 migration-bootstrap pipeline | AI coding agent | 新 popsicle-new 内项目初始化 | skill-runtime + cli-ux |
| UI-2 | AI agent 创建 / 校验 stage artifact 文档，按 IDD 纪律推进 | AI coding agent | pipeline 任一 stage 内 | artifact-system + cli-ux |
| UI-3 | AI agent 查询 pipeline / stage / doc 当前状态，决定下一步 | AI coding agent | stage 切换 / 用户暂停后恢复 | skill-runtime + cli-ux |
| UI-4 | 人类维护者在 popsicle-new 上扩展新 skill，跑 dry-run | 人类工程师 | intent-coder 增加 skill 时 | skill-runtime |
| UI-5 | 人类维护者审查 pipeline 完整产物链供合规 / 复盘 | 人类工程师 | 项目阶段性 review | artifact-system |

---

### Action Items（传后续 stage 的输入）

| 接收 stage | 输入产物 | 来源 |
|---|---|---|
| **prd → prd-writer** | C-prime 5 项裁决 + 3 个 product 一行用途双行模板 + 表 1 Task 识别表 + 表 3 User Intents Catalog | 本 Decision + 表 1 + 表 3 |
| **arch-debate** | "命令树重组" 技术架构问题 + crate 拆分（popsicle-skill-runtime / popsicle-artifact-system / popsicle-cli-ux）+ task_chunk_entity 改名实施细节 | 本 Decision + 表 1 |
| **rfc → rfc-writer** | "intent-coder is internal consumer" 原则 + 命令兼容策略 | charter footnote 决策 |
| **adr → adr-writer** | ADR-001 起草任务（A05）| charter footnote 决策 |
| **intent-spec-writer** | 表 2 Intent 层归类表（7 条候选 intent）| 本表 2 |
| **living-doc-author** | 4 个文档动作：A01 PROJECT_CONTEXT 触发条件 / A02 glossary task_chunk_entity / A03 反传 intent-coder 模板 / A04 MIGRATION.md | 表 1 A01-A04 |

### 用户输入

**[已 freeze 2026-06-08]** 用户确认 freeze + stage complete debate + commit 带 lineage。决策矩阵 + Decision + 3 张表 + Action Items 已起草。所有 fact-cite / 决策 / 命名 / 归属都基于前三 Phase。confidence=3 模式下 agent 推荐**直接 freeze 这份 debate-record + 推进 stage complete debate**。

请你（用户）做最后裁定：
1. **决策是否 freeze**（如否，回到 Phase 3 重开）
2. **是否同意 stage complete debate**（→ 解锁 prd stage）
3. **是否需要把这份 debate 的 commit message 加上 "supersedes D2 / aligns ROADMAP D4" 等溯源信息**


## Decision

详见上方 §Phase 4 §Decision 节。最终 freeze 待用户在文档末尾「用户输入」节确认后由 stage complete 触发。

---

## 🎤 Setup 暂停 — 等待你的输入

按 product-debate skill 工作流（`setup → debating` 需要 `requires_approval: true`），我已经准备好辩论的所有素材，**但不能自启**进入辩论。

请你：
1. 确认 / 调整上述 5 候选项清单（要不要加 / 删 / 改）
2. 确认 / 调整 5 角色阵容（要不要换 GROWTH 为 PLATFORM-ARCH-MIRROR、要不要加第 6 个）
3. 设置 User Confidence（1-5）
4. 给一个 `start` 指令 → 进入 Phase 1

❓ 准备就绪请明示。

## Phase Coverage

> （本段由 living-doc-author 在 verify 阶段对照真实产物补齐；逐项已核验为已完成。）

- [x] Phase 1 已完成，有用户痛点 + 目标用户 + 约束清单
- [x] Phase 2 已产出 2-3 个差异化候选方案（A/B/C）
- [x] Phase 3 全部角色已发表评审意见
- [x] Phase 4 已收敛到推荐方案 + 用户最终决策（C-prime）
- [x] 至少 4 个用户交互点

## Output Checklist

> （本段由 living-doc-author 在 verify 阶段对照真实产物补齐；逐项已核验为已完成。）

- [x] debate-record 含 Phase 1-4 全部小结
- [x] debate-record 标注了「用户决策覆盖」
- [x] prd-draft 的每个核心声明都标注了 intent 层
- [x] prd-draft 的每个数字 / LoC / 模块名引用都能追溯到 fact-extraction-report
- [x] decision-matrix 的维度由角色提出且权重明示
- [x] 三份 artifact 的 `Topic` 字段一致
- [x] 已向用户展示三份产出并取得 `approve` 确认
