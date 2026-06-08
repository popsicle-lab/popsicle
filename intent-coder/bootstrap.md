# Bootstrap 指南 — intent-coder 模块

你正在把一份遗留代码库迁移进 Intent-Driven Development 工作流。本模块提供**考古 + 闸门**的能力，把「裸 legacy 代码」和「spec 驱动的 IDD 管线」缝合起来。

IDD 适配旅程（intent-coder 负责实心方框、外部 writer 负责虚线方框）：

```
空仓库 + legacy submodule
    │
    ▼
┌──────────────────┐
│ project-init     │  ← intent-coder
│  铺 4 件套目录    │     按 product 铺出
│  pin legacy      │     落地 doc-architecture charter
│  落地 charter    │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ fact-extractor   │  ← intent-coder
│  从 legacy 抽事实 │     输出 5 个结构化事实文件
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ product-debate   │  ← intent-coder
│  多角色辩论       │     消费 fact-extraction-report 的
│                  │     Bounded Contexts / Risk Hotspots
└────────┬─────────┘     产出 prd-draft + debate-record + decision-matrix
         │
         ▼
┌──────────────────┐
│ prd-writer       │  ← intent-coder (v0.2 任务图范式)
│  打任务图五件套   │     产出 PRD overview + N 份 task 文件
│                  │     （按 5 个旅程阶段归类）+ tasks/README +
│                  │     acceptance.intent 种子 + PDR 骨架
└────────┬─────────┘     质量评分 ≥ 90 才放行
         │
         ▼
┌──────────────────┐
│ arch-debate      │  ← intent-coder（Phase 3 内置）
│ → rfc-writer     │     消费 PRD Intent Mapping 标 contracts / [ADR 候选] 的条目
│ → adr-writer     │     辩论 → RFC → 固化 ADR(Accepted) → 解锁 contracts.intent
└────────┬─────────┘     （无跨模块契约 / ADR 候选时整段可跳过）
         │
         ▼
┌──────────────────┐
│ intent-spec-     │  ← intent-coder（Phase 1 内置）
│ writer           │     把 acceptance 种子 + 解锁的 contracts 收紧成
│                  │     合法 .intent，合并到 products/.../intents/*.intent
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ intent-          │  ← intent-coder（自带 Z3）
│ consistency-     │     在 .intent 上跑 SMT 判决
│ check            │     observe 出报告；CI 用 tool exit code 做硬闸
└────────┬─────────┘
         │
         ▼
   实现 + 单测 codegen   ← 不属本模块
         │
         ▼
┌──────────────────┐
│ living-doc-      │  ← intent-coder
│ author           │     上游变了就刷活文档对应章节
└──────────────────┘
```

> **本模块的边界**（截至 Phase 3）：产品侧（产品讨论 → PRD → PDR + acceptance.intent 种子）
> 与技术侧（架构辩论 → RFC → ADR + contracts.intent 收紧）**现已全部内置**。
> 留在模块边界**之外**的只有：① 代码实现 / 单测生成（见下方「明确不做」）；
> ② charter 本身的修订（走 CADR 流程，不在任何 writer skill 内）。任何符合 charter 的
> 第三方 writer 仍可替换内置 skill 接进来，但不再是「缺位必须自带」。

---

## Skill 选用指南

| Skill | 何时跑 |
|---|---|
| **project-init** | 在任何一次 IDD 迁移**最开头跑一次**。决定 product 命名、挑首个迁移切片、把 legacy pin 成 git submodule、铺出每个 product 的 4 件套目录、落地 doc-architecture charter。之后所有 skill 都写进这一步铺好的目录里。|
| **fact-extractor** | 紧跟 project-init，跑在 pinned 的 legacy submodule 上。产出 dependency-graph、public-API contracts、unsafe/risk 清单、tech-debt 清单。**所有下游 writer 都消费它的输出。** |
| **product-debate** | 对**每个**要走 IDD 流程的 product slice 跑一次（每个新功能 / 重要决策也跑一次）。需要 fact-extraction-report 作为辩论的事实基。产出 PRD 草稿、辩论纪要、决策矩阵。|
| **prd-writer** | 紧跟 product-debate。把辩论草稿打磨成可落地的 IDD **任务图五件套**（PRD overview + N 份 task 文件按 5 个旅程阶段归类 + tasks/README + acceptance.intent 种子 + PDR 骨架）。质量评分（含 AI 可消化度维度）≥ 90 才放行。**也支持直接调用**（绕过辩论），但 PDR 的 Decision Context 章节会单薄，且需要本 skill 替 PM 做 task 识别（质量风险高）。|
| **arch-debate** | 当 PRD 的 Intent Mapping 标了 `contracts.intent` / [ADR 候选]、需要定跨模块契约或架构选型时跑。多角色技术辩论（ARCH/SEC/PERF/OPS/DATA/DEV），产 RFC 草稿 + 技术决策矩阵 + 辩论纪要。**无跨模块契约的 PRD 可整条跳过技术侧支线**。|
| **rfc-writer** | 紧跟 arch-debate。把 RFC 草稿打磨成正式 RFC + ARCHITECTURE.md 增量 + contracts.intent 种子（Awaiting ADR）+ ADR 骨架（Proposed），质量评分 ≥ 90 才放行。|
| **adr-writer** | 紧跟 rfc-writer。把 ADR 骨架固化为 Accepted（此后不可变），解锁 contracts 种子的 Awaiting 状态、列出收紧工单交 intent-spec-writer。技术决策的审批闸。|
| **intent-spec-writer** | 在 acceptance 种子（prd-writer）或 contracts 解锁（adr-writer）产出后跑。把种子收紧成合法 `.intent` 并合并到 `intents/*.intent`，自验 `intent check` 通过。|
| **intent-consistency-check** | 任何一次 pipeline 阶段产生或修改了 `.intent` 文件之后。它是 Z3 闸——observe 出报告；CI 用 `intent-validate` tool 的非 0 退出码阻塞下游阶段。|
| **living-doc-author** | 当首个迁移切片完成、已经有 PRD/RFC 之后。代码或 intent 变化时重跑，让活文档的「现在状态」章节不腐烂。|

---

## 何时跑 project-init

**每个仓库出生时跑一次。** 支持重跑但场景少：

- product 命名错了 → 重跑（便宜；趁还没有太多下游引用这些名字时）
- legacy submodule 的 pin 需要换 → 重跑（罕见）
- **不要**为了「刷新 charter」而重跑——charter 改动走 CADR（Charter Amendment Decision Record）

## 何时跑 fact-extractor

**项目 bootstrap 时跑一次，之后这些情况再跑**：

- 代码库经历大规模重构（新模块、大删除）
- 引入新领域（新的 bounded context）
- 任何大规模重构之前（留一份「重构前」基线）

**不要每次 PR 都跑**——它是基线工具，不是 CI 闸门。

## 何时跑 product-debate

**对每个 product slice / 每个重大产品决策跑一次**：

- 首切片：fact-extractor 完成后立刻跑
- 后续 product：当 PM 想做实质性产品决策时跑（不是每个小调整都跑）
- 跨 product 议题：通常拆成多场单 product 辩论；真要跨时显式标注

**强制前提**：fact-extraction-report 可用（否则辩论会基于自然语言推断，质量评分扣分）。

## 何时跑 prd-writer

**通常紧跟 product-debate**：

- 辩论结束 → prd-draft → prd-writer 升级为三联体
- 也支持**绕过辩论**直接调用（例如紧急 bugfix 类的小 PRD），但 PDR Decision Context 会单薄

**注意**：prd-writer 不产架构内容。涉及模块间契约的部分会标注「[ADR 候选：技术方案待 arch-debate 确认]」，留给外部 RFC/ADR writer。

---

## Pipeline 选用指南

当用户开始迁移一份遗留代码库时，用 **`migration-bootstrap`**（`pipelines/migration-bootstrap.pipeline.yaml`，10 stage DAG）：

```
init (project-init)                （一次性、交互式）
  → facts (fact-extractor)         （基线）
  → debate (product-debate)        （首切片产品辩论）
  → prd (prd-writer)               （PRD + acceptance.intent 种子 + PDR 骨架）
  → arch-debate                    （技术侧支线起点，无 contracts 候选可 skip 本段）
  → rfc (rfc-writer)               （RFC + contracts 种子 + ADR 骨架）
  → adr (adr-writer)               （固化 ADR，解锁 contracts）
  → intent-spec (intent-spec-writer)（收紧 acceptance + contracts → 合并）
  → intent-check (intent-consistency-check)（Z3 闸，observe）
  → living-docs (living-doc-author)（活文档保活）
```

全部 10 个 stage 都是 intent-coder 自带 skill——一键 `popsicle pipeline run migration-bootstrap`
即可，不必逐个 `skill start`。技术侧支线（arch-debate → rfc → adr）在 PRD 不含跨模块契约
时可整段 skip（popsicle 把 skipped 视为依赖已满足，下游 intent-spec 照常 ready）。

如果用户手上已有 PRD/ADR/intent 三件套、只想跑 Z3 闸，单独调用 `intent-consistency-check`
skill 即可（它能作为 stage skill 接入任何 pipeline）。
