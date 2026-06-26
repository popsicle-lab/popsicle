# intent-coder

一个 popsicle 模块，帮你把**遗留代码库迁移到 IDD（Intent-Driven Development，意图驱动开发）工作流**里，也能把全新的产品 / 模块从 product brief 推进到可验证 spec：PRD 任务图、intent 验证、活文档持续更新。

本模块站在 IDD 工作流的最前端，连接「我有一坨老代码」和「我准备好按 spec-and-verify 工作了」之间的鸿沟——先**抽取事实**（这份代码今天到底在干什么）、再**用形式化 intent 把闸**（拦住不一致的下游 spec）、再**持续保活产品/技术文档**（让它们随代码演进而不腐烂）。

本模块**自带** `.intent` DSL 与 Z3 验证能力，**自带**从产品讨论 → PRD 起草 → 架构辩论 → RFC/ADR → intent 收紧 → 一致性验证 → 活文档保活的完整主链（产品侧 + 技术侧 10 个 skill 全内置，含 PDR/ADR + acceptance/contracts.intent 种子产出与正式化）。留在模块边界外的只有代码实现 / 单测生成，以及 charter 本身的修订（走 CADR）。任何符合 charter 的第三方 writer 仍可替换内置 skill。

## v0.3 范式升级：从功能树到任务图

`prd-writer` 起草产物从「单一 PRD 文件 + Feature 列表」升级为**任务图**——PRD 拆解为按 **5 个固定用户旅程阶段**（`onboarding` / `daily-ops` / `troubleshooting` / `admin` / `lifecycle`）组织的 task chunk，每个 task 一份独立 `.md` 文件，自带 YAML frontmatter（audience / journey_stage / prerequisites / query_anchors 等），可被 AI Copilot 独立召回精准回答用户问题。

这是 AI 时代产品文档的「极致结构化 + 以任务和意图为中心 + Chunking-friendly」范式落地。详见 [`skills/prd-writer/references/task-organization.md`](skills/prd-writer/references/task-organization.md)。

---

## Skills

| Skill | 状态 | 用途 |
|-------|------|------|
| **project-init** | ✅ 已交付 | 给新仓库铺骨架：把 legacy pin 成 git submodule、为每个 product 铺 4 件套目录（`PRODUCT.md` / `ARCHITECTURE.md` / `intents/` / `decisions/` + `proposals/`）、把 doc-architecture charter 落地到 `docs/CHARTER.md` |
| **fact-extractor** | ✅ 已交付 | 读遗留代码，输出结构化事实基（dependency graph、public-API contracts、unsafe/risk 清单、tech-debt 清单）——下游所有 writer 都消费这份事实 |
| **product-debate** | ✅ 已交付 | 多角色产品辩论模拟器：用 4-6 个角色（PM / UXR / GROWTH / ENGLD / BIZ）就一个 product slice 充分辩论方案空间。支持 `legacy-fact-baseline`（fact-extractor + project-init）和 `greenfield-product-brief`（产品简报 + 显式 target_product）。**v0.3：Phase 4 PM 强制做 task 识别 + intent 层归类 + User Intents Catalog 起草**。产出 task-centric PRD 草稿 + 决策矩阵 + 辩论纪要，喂给 prd-writer |
| **prd-writer** | ✅ 已交付（v0.2 任务图） | 把辩论产出（或直接需求）打磨成 IDD 任务图五件套：**PRD overview**（PRODUCT.md 顶层增量）+ **N 份 task 文件**（按 5 个旅程阶段归类）+ **tasks/README.md**（索引）+ **acceptance.intent 种子**（单文件多 block，与 task_id 双射）+ **PDR 骨架**（Consequences 精确到文件级）。强制贯彻 charter 四条铁律 + AI 时代任务图范式，质量评分（5 维度 100 分）≥ 90 才放行 |
| **arch-debate** | ✅ 已交付 | 多角色**技术架构**辩论模拟器（ARCH / SEC / PERF / OPS / DATA / DEV），是 product-debate 的技术侧对称体。消费 PRD 里标了 `contracts.intent` / [ADR 候选] 的条目 + 事实基，产出 RFC 草稿 + 技术决策矩阵 + 辩论纪要。**Phase 3 起内置**。无跨模块契约的 PRD 可整段跳过技术侧支线 |
| **rfc-writer** | ✅ 已交付 | prd-writer 的技术侧对称体：把 RFC 草稿打磨成正式 RFC + ARCHITECTURE.md 增量 + `contracts.intent` 种子（Awaiting ADR）+ ADR 骨架（Proposed），质量评分（4 维度 100 分）≥ 90 才放行 |
| **adr-writer** | ✅ 已交付 | 技术决策审批闸：把 ADR 骨架固化为 Accepted（此后不可变），并**解锁** `contracts.intent` 种子的 `[Awaiting ADR]`、列出收紧工单交 intent-spec-writer。保持薄——不发明 ADR 内容，只做固化门 + 解锁 |
| **intent-spec-writer** | ✅ 已交付 | 把 `prd-writer` 的 acceptance 种子 + `adr-writer` 解锁的 contracts 收紧成合法 intent-lang：分层归位（acceptance/invariants/contracts）、剥离时间/性能约束（D2）、四规则审查、去重查冲突，`intent check` 自验后合并到 `intents/*.intent`。**Phase 1 起内置** |
| **intent-consistency-check** | ✅ 已交付（observe） | intent-coder 自带的 Z3 闸：对全量 `.intent` 跑 `intent check`，汇总 verified/failed/skipped 出报告。skill 始终 observe（不阻断）；硬闸由 CI 用 `intent-validate` tool 的 exit code 实现，附量化的 observe→gate 退出判据 |
| **living-doc-author** | ✅ 已交付 | 活文档保活/对账：扫 doc-code drift（过期/断链/孤儿/未验证），刷新 `tasks/README.md` 健康度、task 反向引用、frontmatter `last_verified`。v0.4 增 `implementation-status` / `architecture-manifest` / `product-header` target。只对账不创作正文（正文改动走 prd-writer + PDR）|
| **shadow-implementer** | ✅ 已交付（v0.4） | 按 intents + ADR File Manifest 实现/补齐 `crates/<slice>/` in-shadow 代码；产出 intent→fn/test 覆盖表。`migration-slice-delivery` / `feature-delivery` 第一棒 |
| **equivalence-baseline** | ✅ 已交付（v0.4） | legacy vs new golden 对账、`docs/baseline/`、traceability 草稿、divergence 登记。切流门禁 ≥5 golden pass |
| **cutover-author** | ✅ 已交付（v0.4） | 核验 intent/equivalence/cargo 三门禁；切流 ADR(Accepted)；更新 `migration/progress.md` |

**Spec 侧** 10 个 skill 的依赖顺序（产品侧 1-4 + 技术侧支线 5-7 + 收口 8-10）：

1. `project-init` —— 仓库出生时跑一次，铺出后续所有 skill 写入的目录舞台。
2. `fact-extractor` —— 在 pinned 的 legacy submodule 上跑，产出证据基。
3. `product-debate` —— 在一个 product slice 上辩论，输出 task-centric PRD 草稿。
4. `prd-writer` —— 输出五件套（PRD + acceptance 种子 + PDR 骨架）。
5. `arch-debate` —— **（技术侧支线，无 contracts 候选可跳过）** 就 PRD 标的契约/选型做架构辩论。
6. `rfc-writer` —— 把架构辩论打磨成 RFC + contracts 种子 + ADR 骨架。
7. `adr-writer` —— 固化 ADR（Accepted）+ 解锁 contracts 种子。
8. `intent-spec-writer` —— 把 acceptance 种子 + 解锁的 contracts 收紧成合法 `.intent`，合并。
9. `intent-consistency-check` —— 在 intent 合并后跑 Z3 一致性验证（observe）。
10. `living-doc-author` —— spec 完成后或 delivery pipeline 末尾保活文档。

**Delivery 侧**（v0.4，`migration-slice-delivery` / `feature-delivery` pipeline）：

11. `shadow-implementer` —— 按 ADR 实现 `crates/<slice>/` + property tests。
12. `equivalence-baseline` —— golden 对账 + traceability 草稿。
13. `cutover-author` —— 切流 ADR + progress 看板。

## Pipeline 怎么选

**必读**：[`skills/issue-author/guide.md`](skills/issue-author/guide.md) —— Issue 创建（pipeline 决策树 + delivery 门禁）。

要点：`migration-slice-delivery` / `feature-delivery` **不是**新产品 feature 的 spec 流程；它们要求 spec 链已完成。`--type technical` 默认是 `arch-decision`，不是 delivery。

## Pipelines（ADR-029 canonical 名）

| Pipeline | 用途 |
|----------|------|
| **migration-bootstrap** ✅ | **仓库级 Day-1 一次**（10 stage）。见 [`pipelines/migration-bootstrap.pipeline.yaml`](pipelines/migration-bootstrap.pipeline.yaml)。 |
| **migration-slice-spec** ✅ | **迁移 slice spec**（无 init）。见 [`pipelines/migration-slice-spec.pipeline.yaml`](pipelines/migration-slice-spec.pipeline.yaml)。 |
| **migration-slice-delivery** ✅ | **迁移 slice 交付**：implement → equivalence → cutover → living-docs。见 [`pipelines/migration-slice-delivery.pipeline.yaml`](pipelines/migration-slice-delivery.pipeline.yaml)。 |
| **product-greenfield-spec** ✅ | **新产品 / 新模块 spec**。见 [`pipelines/product-greenfield-spec.pipeline.yaml`](pipelines/product-greenfield-spec.pipeline.yaml)。 |
| **feature-spec** ✅ | **已有 product 增量能力 spec**（无 legacy facts）。见 [`pipelines/feature-spec.pipeline.yaml`](pipelines/feature-spec.pipeline.yaml)。 |
| **feature-arch-spec** ✅ | **已有 product 大增量 full spec**（PDR+ADR+task+intent；无 product-debate）。见 [`pipelines/feature-arch-spec.pipeline.yaml`](pipelines/feature-arch-spec.pipeline.yaml)。ADR-030。 |
| **feature-delivery** ✅ | **日常能力交付**（implement → verify）。见 [`pipelines/feature-delivery.pipeline.yaml`](pipelines/feature-delivery.pipeline.yaml)。 |
| **doc-retro-spec** ✅ | **代码已合并后补 PDR/task/intent**。见 [`pipelines/doc-retro-spec.pipeline.yaml`](pipelines/doc-retro-spec.pipeline.yaml)。 |
| **doc-sync-weekly** ✅ | **周期性巡检**：tasks-index + product-context。见 [`pipelines/doc-sync-weekly.pipeline.yaml`](pipelines/doc-sync-weekly.pipeline.yaml)。 |
| **arch-decision** ✅ | **架构 ADR 链**。见 [`pipelines/arch-decision.pipeline.yaml`](pipelines/arch-decision.pipeline.yaml)。 |
| **fix-regression** ✅ | **单点回归**（`--type bug` 默认）。见 [`pipelines/fix-regression.pipeline.yaml`](pipelines/fix-regression.pipeline.yaml)。 |
| **platform-refactor** ✅ | **内部重构 / infra**。见 [`pipelines/platform-refactor.pipeline.yaml`](pipelines/platform-refactor.pipeline.yaml)。 |

旧名（`slice-delivery`、`bugfix` 等）仍可作为 CLI alias；新 Issue 应使用 canonical 名。

## 使用

在新项目里：

```bash
mkdir new-project && cd new-project && git init
popsicle init   # installs pipelines + intent-coder module when intent-coder/ exists at repo root
# Or refresh module after upstream updates:
popsicle admin sync-intent-coder
# Legacy / external checkout:
popsicle module add /path/to/intent-coder   # deferred in self-host MVP; use sync-intent-coder in-repo

# 1. 铺出文档骨架（交互式 —— 命名 products、挑首个迁移切片）
popsicle skill start project-init

# 2. 通过审批后，扫遗留 submodule 抽事实
popsicle skill start fact-extractor --source legacy/<your-legacy-name>

# 3. 对首切片 product 开一场多角色产品辩论
popsicle skill start product-debate

# 4. 把辩论产出打磨成 PRD 五件套（PRD + acceptance.intent 种子 + PDR）
popsicle skill start prd-writer

# 5-7.（技术侧支线，PRD 含跨模块契约 / [ADR 候选] 时才跑）
popsicle skill start arch-debate      # 多角色技术辩论 → RFC 草稿
popsicle skill start rfc-writer       # RFC + contracts 种子 + ADR 骨架
popsicle skill start adr-writer       # 固化 ADR(Accepted) + 解锁 contracts

# 8. 把 acceptance 种子 + 解锁的 contracts 收紧成合法 .intent 并合并
popsicle skill start intent-spec-writer

# 9. 过 intent 一致性闸（Z3 / observe）
popsicle skill start intent-consistency-check

# 10. 保活活文档（刷新 tasks 索引 / 健康度 / 反向引用）
popsicle skill start living-doc-author --target all

# —— 仓库级：一键 migration-bootstrap（10 stage，仅 Day-1 一次）——
# popsicle pipeline run migration-bootstrap

# —— 后续 slice：spec + delivery（同一 issue 可链式跑）——
# popsicle issue create ... --pipeline migration-slice-spec
# popsicle issue create ... --pipeline migration-slice-delivery   # 或 feature-delivery

# —— 新产品 / 新模块：从 product brief 直接进入 spec 链 ——
# popsicle issue create ... --pipeline product-greenfield-spec

# delivery 末尾建议：
# popsicle skill start living-doc-author \
#   --target implementation-status,architecture-manifest,product-header
```

## 文档 / intent 各文件的归属

截至 Phase 3，产品侧与技术侧 writer **全部内置**。各文件的产出者：

- `PRODUCT.md` / `ARCHITECTURE.md` 的**骨架**由 `project-init` 铺出（含 `[TBD: needs archaeology]` 占位符）。
- `PRODUCT.md` 的**内容**由 `prd-writer` 产出（基于 `product-debate` 辩论摘要）。
- `ARCHITECTURE.md` 的**内容**由内置的 `rfc-writer` 产出（基于 `arch-debate` 辩论摘要）。
- `decisions/pdr/*.md` 的**内容**由 `prd-writer` 产出（PDR 骨架），用户审批后落地。
- `decisions/adr/*.md` 的**内容**由 `rfc-writer` 产骨架、`adr-writer` 固化为 Accepted。
- `intents/acceptance.intent` 的**种子**由 `prd-writer` 产出，由 `intent-spec-writer` 收紧合并。
- `intents/contracts.intent` 的**种子**由 `rfc-writer` 产出（Awaiting ADR），`adr-writer` 解锁后由 `intent-spec-writer` 收紧。
- `intents/invariants.intent` 由 `intent-spec-writer` 分层填入（safety + primed）。
- 所有 intent 的**一致性校验**由 `intent-consistency-check`（Z3，observe + CI gate）负责。

第三方替换：任何符合 charter「四条铁律」（活文档每次 edit 带 Decision-Ref）的外部 writer
都可替换对应内置 skill 接入 pipeline。charter 本身的修订走 **CADR**，不在任何 writer skill 内。

## 为什么单独成一个模块

这 10 个 skill 都是**迁移期工具**，不是 IDD 日常工具。拆开成独立模块的好处：

- 想给一个全新项目上 IDD 时，**不用把迁移脚手架硬塞进日常 pipeline**；
- 遗留考古能力（fact-extractor）在任何代码库上都能复用，跟你最终用不用 `.intent` 无关；
- 这个边界让 IDD 契约更清晰：「migration-bootstrap 跑完后，你欠 1 套 PRD/RFC/ADR + 至少 1 个跑通的 `.intent` 文件」。

## License

MIT
