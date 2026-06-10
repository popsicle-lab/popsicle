# Bootstrap 指南 — intent-coder 模块

你正在把一份遗留代码库迁移进 Intent-Driven Development 工作流，或把一个全新产品 / 模块从 product brief 推进到可验证 spec。本模块提供**考古 + 闸门 + greenfield spec 链**的能力，把「裸 legacy 代码」或「自然语言产品想法」接到 IDD 管线里。

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
┌──────────────────┐
│ slice-delivery   │  ← intent-coder（v0.4）
│  shadow-         │     按 ADR 实现 crates/<slice>/
│  implementer     │
│       ↓          │
│  equivalence-    │     legacy vs new golden
│  baseline        │
│       ↓          │
│  cutover-author  │     切流 ADR + progress.md
│       ↓          │
│  living-doc-     │     实现态保活（implementation-status 等）
│  author          │
└──────────────────┘
```

> **本模块的边界**（v0.4）：产品侧 + 技术侧 spec 链**全部内置**；**delivery 链**
>（in-shadow 实现编排、golden 对账、切流 ADR）**也已内置**，由 `slice-delivery` pipeline
> 串起。仍留在边界**之外**的：① 无约束的「一键 codegen」——shadow-implementer 只按
> ADR/intent 范围实现，不发明 scope 外逻辑；② charter 修订（CADR）。

---

## Skill 选用指南

| Skill | 何时跑 |
|---|---|
| **project-init** | 在任何一次 IDD 迁移**最开头跑一次**。决定 product 命名、挑首个迁移切片、把 legacy pin 成 git submodule、铺出每个 product 的 4 件套目录、落地 doc-architecture charter。之后所有 skill 都写进这一步铺好的目录里。|
| **fact-extractor** | 紧跟 project-init，跑在 pinned 的 legacy submodule 上。产出 dependency-graph、public-API contracts、unsafe/risk 清单、tech-debt 清单。**所有下游 writer 都消费它的输出。** |
| **product-debate** | 对**每个**要走 IDD 流程的 product slice 跑一次（每个新功能 / 重要决策也跑一次）。迁移场景消费 fact-extraction-report；greenfield 场景消费 product brief + 显式 target_product。产出 PRD 草稿、辩论纪要、决策矩阵。|
| **prd-writer** | 紧跟 product-debate。把辩论草稿打磨成可落地的 IDD **任务图五件套**（PRD overview + N 份 task 文件按 5 个旅程阶段归类 + tasks/README + acceptance.intent 种子 + PDR 骨架）。质量评分（含 AI 可消化度维度）≥ 90 才放行。支持 legacy Product Inventory，也支持 greenfield target_product 声明并在 File Manifest 中创建 / 声明目录骨架。|
| **arch-debate** | 当 PRD 的 Intent Mapping 标了 `contracts.intent` / [ADR 候选]、需要定跨模块契约或架构选型时跑。多角色技术辩论（ARCH/SEC/PERF/OPS/DATA/DEV），产 RFC 草稿 + 技术决策矩阵 + 辩论纪要。**无跨模块契约的 PRD 可整条跳过技术侧支线**。|
| **rfc-writer** | 紧跟 arch-debate。把 RFC 草稿打磨成正式 RFC + ARCHITECTURE.md 增量 + contracts.intent 种子（Awaiting ADR）+ ADR 骨架（Proposed），质量评分 ≥ 90 才放行。|
| **adr-writer** | 紧跟 rfc-writer。把 ADR 骨架固化为 Accepted（此后不可变），解锁 contracts 种子的 Awaiting 状态、列出收紧工单交 intent-spec-writer。技术决策的审批闸。|
| **intent-spec-writer** | 在 acceptance 种子（prd-writer）或 contracts 解锁（adr-writer）产出后跑。把种子收紧成合法 `.intent` 并合并到 `intents/*.intent`，自验 `intent check` 通过。|
| **intent-consistency-check** | 任何一次 pipeline 阶段产生或修改了 `.intent` 文件之后。它是 Z3 闸——observe 出报告；CI 用 `intent-validate` tool 的非 0 退出码阻塞下游阶段。|
| **living-doc-author** | spec 完成后或 slice-delivery 末尾。代码或 intent 变化时重跑。slice-delivery 建议 `--target implementation-status,architecture-manifest,product-header`。|
| **shadow-implementer** | slice spec（intent-check）通过后，`slice-delivery` 第一棒。按 ADR File Manifest 写 `crates/<slice>/`。|
| **equivalence-baseline** | shadow-implementer 之后。建 golden、写 traceability 草稿、登记 divergence。|
| **cutover-author** | equivalence 门禁通过后。切流 ADR + `migration/progress.md`。|

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

迁移模式强烈要求 fact-extraction-report 可用（否则辩论会基于自然语言推断，质量评分扣分）。
greenfield 模式的前提是 product brief 足够明确，至少包含目标用户、范围、硬约束和成功信号。

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
skill 即可。

当用户开始一个新产品 / 新模块，没有 legacy fact baseline 时，用
**`greenfield-product-spec`**：

```
debate (product-debate, greenfield-product-brief)
  → prd (prd-writer, declares/creates target_product skeleton as needed)
  → arch-debate → rfc → adr   （有 contracts / 架构选型时跑）
  → intent-spec → intent-check → living-docs
```

这条链用于外部项目复用：它不依赖 popsicle 自迁移，也不要求先跑 project-init /
fact-extractor。product-debate 的依据是 Product Brief；prd-writer 负责把新
target_product 的目录与文件清单显式落进 File Manifest。

### slice-spec + slice-delivery（v0.4，每个 slice）

仓库已由 `migration-bootstrap` 铺好后：

```
slice-spec（无 init，6～8 stage）
  facts → debate → prd → [arch-debate → rfc → adr] → intent-spec → intent-check

slice-delivery（4 stage，同一 issue 链式跑）
  implement (shadow-implementer)
  → equivalence (equivalence-baseline)
  → cutover (cutover-author)          ← requires_approval
  → living-docs (living-doc-author)   ← requires_approval
```

**spec 已完成的 slice**（如 popsicle-new 的 skill-runtime / artifact-system）可
**跳过 slice-spec，直接 `popsicle pipeline run slice-delivery`**。
