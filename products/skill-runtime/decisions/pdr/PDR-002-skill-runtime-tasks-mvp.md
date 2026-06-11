# PDR-002 · skill-runtime tasks MVP（6 个 task 跨 5 旅程阶段）

> **Status**: Proposed（待用户审批 → Accepted；Accepted 后永不修改 / charter 铁律 #2）
> **Date**: 2026-06-08
> **Stage**: `prd`（intent-coder migration-bootstrap pipeline, run `f89529af-d8ce`）
> **Target Product**: `skill-runtime`
> **Decision Type**: Product Decision Record (PDR)
> **Supersedes**: —— （初次定义 skill-runtime tasks）
> **Source PDR**: [`PDR-001-skill-runtime-scope-and-d4-legacy-slimming.md`](./PDR-001-skill-runtime-scope-and-d4-legacy-slimming.md)（scope 来源）
> **Related ADRs**: 待 `arch-debate` / `adr-writer` 落地 `ADR-001 intent-coder is internal consumer` + `ADR-002 命令树重组` 后回填
> **Related Journey**: —— （跨 product 旅程将在 artifact-system / cli-ux 落地后注册）

---

## Decision Context

### 触发因素

PDR-001 把 `skill-runtime` 的 product scope 锁定为「加载 skill、串成 pipeline、按 stage 推进」（一行用途 22 字）。但 PDR-001 §Phase 4 表 3 只给了 5 条 abstract 的 User Intents Catalog；落地需要 **具体的 task chunk**，让 AI agent 在 RAG 召回时可独立回答用户原话问句。

### 多角色辩论摘要

> 本 PDR 基于 PDR-001 的辩论产出，不重复辩论；仅做 task 颗粒度落地。

**参与角色**: PM / UXR / ENGLD（PDR-001 三人小组）继续 inherit
**用户置信度**: 3/5（与 PDR-001 一致）
**用户在 prd 阶段的关键决策**:
- task 数量 = 6（不是 4 也不是 8）
- 范围 `exclude_doc`：本 PRD 不含 doc 操作（doc 归 artifact-system 的未来 PDR）
- task 文件体量 = compact（每个 ~100 行）

**核心事实引用**:
- F-1: PDR-001 §Phase 4 表 3 (User Intents Catalog UI-1..UI-5)
- F-2: fact-extraction-report § api-contracts.md §popsicle-cli（既有命令族 `skill / pipeline / stage / issue / doc`）
- F-3: prd-writer 范式硬约束（5 旅程阶段 / task ≤250 行 / 每个 task 3-5 个 query 锚点）

### 备选方案

| 方案 | 提案者 | 否决理由 |
|---|---|---|
| 4 个 task（合并 T-0002/T-0003） | UXR 备选 | 合并后单 task 步骤 > 7，违反 happy-path 上限；T-0006 lifecycle 也不该提前砍 |
| 8 个 task（加 admin 库锁释放 + lifecycle 回滚） | ENGLD 备选 | 数量 ≥ 8 触发 prd-writer 拆 PDR 警告；本次 MVP 先收敛到 6 个，库锁释放并入 T-0004 |

---

## Decision

定义 `skill-runtime` 的首批 **6 个 user task chunk**，跨 5 个旅程阶段：

| Task | Journey | h1 标题（用户原话）|
|---|---|---|
| T-0001 | onboarding | 我第一次给 intent-coder 加载 skill 包跑通 migration-bootstrap pipeline |
| T-0002 | daily-ops | 我作为 AI agent 把 pipeline 推到下一个 stage（含审批暂停点）|
| T-0003 | daily-ops | 我作为 AI agent 查询当前 pipeline / stage / doc 处于什么状态 |
| T-0004 | troubleshooting | 我的 pipeline 卡在 blocked / 失败的 stage，我要把它恢复 |
| T-0005 | admin | 我作为人类维护者审查一个 pipeline run 的完整产物链以做合规复盘 |
| T-0006 | lifecycle | 我作为 skill 包维护者发布一个新 skill 版本并把现有 pipeline 升上去 |

`PRODUCT.md` 头部双行模板锁定（line 1 ≤ 25 字 + line 2 ≤ 35 字）。

---

## Consequences

### Task File Updates (required by this PDR)

#### 新增 Tasks

- [x] `products/skill-runtime/tasks/onboarding/T-0001-first-pipeline-run.md`
- [x] `products/skill-runtime/tasks/daily-ops/T-0002-advance-stage.md`
- [x] `products/skill-runtime/tasks/daily-ops/T-0003-inspect-state.md`
- [x] `products/skill-runtime/tasks/troubleshooting/T-0004-recover-blocked.md`
- [x] `products/skill-runtime/tasks/admin/T-0005-audit-trail.md`
- [x] `products/skill-runtime/tasks/lifecycle/T-0006-skill-version-bump.md`

#### 修改 Tasks

- [ ] —— 本次无修改（初次落地）

#### 删除 Tasks

- [ ] —— 本次无删除

### PRODUCT.md Top-Level Updates

- [ ] `products/skill-runtime/PRODUCT.md` § 头部双行模板：
  - Line 1: 「加载 skill、串成 pipeline、按 stage 推进。」（22 字）
  - Line 2: 「为 AI coding agent 解决 IDD 工作流的执行编排与状态推进 痛点。」（32 字）
- [ ] `products/skill-runtime/PRODUCT.md` § Problem Statement — 引用 PDR-001 §用户痛点
- [ ] `products/skill-runtime/PRODUCT.md` § Success Metrics — 沿用 PDR-001 §Phase 1 §成功指标 5 项
- [ ] `products/skill-runtime/PRODUCT.md` § User Intents Catalog — 新增 18 行（每个 task 至少 3 个 query 锚点 × 6 tasks）
- [ ] `products/skill-runtime/PRODUCT.md` § Intents Catalog — 新增 4 个 acceptance.intent 关联 + 2 个 invariants.intent 候选

### Tasks Index Updates

- [x] `products/skill-runtime/tasks/README.md` 已重新生成（健康度统计待 living-doc-author 后续刷新）

### Glossary Updates

- [ ] `docs/glossary.md` 新增术语：
  - `journey_stage`（5 选 1 的旅程阶段枚举）
  - `task chunk`（task 的 RAG chunk 表示）
  - `query anchor`（task frontmatter 的用户原话问句字段）
  - `task_chunk_entity`（已在 PDR-001 列入，本 PDR 不重复触发）

### Intent Updates

- [x] `products/skill-runtime/intents/acceptance.intent` 已追加 4 个 block：
  - `PipelineBootstrapsToFirstPause`（T-0001）
  - `StageAdvanceWithApproval`（T-0002）
  - `RecoveredPipelineCanAdvance`（T-0004）
  - `UpgradeDoesNotAffectCompletedRuns`（T-0006）
- [ ] `products/skill-runtime/intents/invariants.intent` 待新增 2 个 block（由 intent-spec-writer 阶段产）：
  - `ReadOnlyCommandsDoNotMutate`（T-0003）
  - `AuditTrailIsReconstructible`（T-0005）
- [ ] `products/skill-runtime/intents/contracts.intent` 待新增（**等 ADR-002 命令树重组落地后** 才能填）：
  - `popsicle skill load` 输出 schema
  - `popsicle pipeline status` 输出 schema
  - `popsicle pipeline stage complete` 状态机契约

### Cross-Product Journey Updates

- [ ] —— 本 PDR 不涉及跨 product 旅程（首切片 + artifact-system / cli-ux 尚未落地）

### Code Updates (informational, not enforced by this PDR)

> 实际代码迁移由 `arch-debate → rfc → adr → intent-spec → intent-check → living-docs` 后续 stage 驱动；本 PDR 只锁定**用户可见行为**。

- 来自 legacy `crates/popsicle-core/src/{commands/skill,commands/pipeline,commands/stage,commands/issue}` 的命令族 → 迁到新 popsicle-new crate（具体 crate 拆分待 arch-debate）
- `legacy/popsicle/crates/popsicle-core/src/model/pipeline.rs::PipelineRun` 实体迁入 skill-runtime
- `legacy/popsicle/crates/popsicle-core/src/commands/skill.rs::{load, list, show}` 命令迁入

### Risk Side-Effects

| Risk | 触发条件 | 缓解 |
|---|---|---|
| `intent check` 对种子 4 个 block 失败 | acceptance.intent 语法不严格符合 intent-lang | intent-spec-writer 阶段做语法 lint；intent-check stage 跑 Z3 闸 |
| 6 个 task 中 T-0003 / T-0005 没有 acceptance block，PRD 评分 AI 可消化度 < 16 | INV 类 task 没有 acceptance 种子 | 在 PRD § Intent Mapping 显式标 `invariants.intent` + Intent Impact 记 invariants 候选 → 让评分计入 invariants 关联 |
| 命令名（如 `popsicle skill load --module`）与未来 ADR-002 命令树重组后不一致 | task 文件直接写了命令字面量 | 文件末尾标 `Decision-Ref: PDR-002`；命令重组的 PDR 会 Supersedes 本 PDR + 同步更新 task |

---

## Intent Impact

| Intent 层 | 修改类型 | 涉及 block | 关联 Task | 备注 |
|---|---|---|---|---|
| `intents/acceptance.intent` | 新增 | `PipelineBootstrapsToFirstPause` | T-0001 | 本 skill 已产种子 |
| `intents/acceptance.intent` | 新增 | `StageAdvanceWithApproval` | T-0002 | 本 skill 已产种子 |
| `intents/acceptance.intent` | 新增 | `RecoveredPipelineCanAdvance` | T-0004 | 本 skill 已产种子 |
| `intents/acceptance.intent` | 新增 | `UpgradeDoesNotAffectCompletedRuns` | T-0006 | 本 skill 已产种子 |
| `intents/invariants.intent` | 待新增 | `ReadOnlyCommandsDoNotMutate` | T-0003 | intent-spec-writer 阶段产 |
| `intents/invariants.intent` | 待新增 | `AuditTrailIsReconstructible` | T-0005 | intent-spec-writer 阶段产 |
| `intents/contracts.intent` | 待新增 | popsicle CLI 输出 schema 3 条 | T-0001/02/03/04/05/06 | **等 ADR-002** |
| `docs/invariants/*.intent` (全局) | 无影响 | —— | —— | 不涉及全局（属 product 内部）|

> 本 PDR 不影响 `docs/invariants/*.intent` 全局 invariant —— 无需升级为 CADR。

---

## Validation Plan

### Acceptance 验证（T+1 周，跑在 intent-check stage）

- 跑 `intent check products/skill-runtime/intents/acceptance.intent`
- 新增 4 个 block 通过 Z3 验证，与已有 invariants / contracts 无矛盾
- intent-check stage 输出 `verified` 4 / `skipped` 0 / `failed` 0

### 用户行为指标（T+30 天 / 上线后）

- popsicle CLI 调用 `pipeline status` / `stage complete` / `skill load` 三个命令占总 CLI 调用数 ≥ 70%（验证 task chunk 覆盖了主路径）
- intent-coder 在 popsicle-new 上完整跑通 `init → living-docs` pipeline 1 次

### AI 反馈闭环指标（T+30 天 / T+90 天）

| 指标 | T+30 天目标 | T+90 天目标 |
|---|---|---|
| Task chunk 召回次数 (T-0001) | ≥ 5 | ≥ 15 |
| Task chunk 召回次数 (T-0002/T-0003) | ≥ 10 各 | ≥ 30 各 |
| AI 错答率（query 锚点对应回答置信度 < 0.7 占比）| < 10% | < 5% |
| 零引用 task | ≤ 2（T-0005 / T-0006 可暂时零引用，因为人类用户少）| 0 — 全部需要至少 1 次引用 |

### 回滚条件

如本 PDR 引入的 task chunk 在 T+30 天内零引用率 > 50%，回滚通过新建一份 PDR 标 `Supersedes: PDR-002` 实现，**不修改本 PDR 文件**。

---

## Approval

- **Status**: Proposed → Accepted（待用户审批后改）
- **Approved by**: ——
- **Approval date**: ——
- **Quality bypass note**: ——（若 PRD overview 评分 < 90 且用户强制 pass 时填理由）

---

## References

- **Source PRD Overview**: `.popsicle/artifacts/f89529af-d8ce-…/skill-runtime-prd-6-tasks-across-5-journey-stages.prd.md`（popsicle-managed）
- **Source Debate**: `PDR-001-skill-runtime-scope-and-d4-legacy-slimming.md`
- **Acceptance Intent Seed**: `products/skill-runtime/intents/acceptance.intent`
- **Fact Basis**: `docs/baseline/2026-06-08/{dependency-graph,api-contracts,unsafe-risk-report,tech-debt-inventory}.md`
- **Affected Task Files**:
  - `products/skill-runtime/tasks/onboarding/T-0001-first-pipeline-run.md`
  - `products/skill-runtime/tasks/daily-ops/T-0002-advance-stage.md`
  - `products/skill-runtime/tasks/daily-ops/T-0003-inspect-state.md`
  - `products/skill-runtime/tasks/troubleshooting/T-0004-recover-blocked.md`
  - `products/skill-runtime/tasks/admin/T-0005-audit-trail.md`
  - `products/skill-runtime/tasks/lifecycle/T-0006-skill-version-bump.md`
- **Related Living Docs**:
  - `products/skill-runtime/PRODUCT.md`（待 living-doc-author 阶段填）
  - `products/skill-runtime/tasks/README.md`

---

*本 PDR 由 prd-writer skill 起草为 Proposed 状态。Charter 铁律 #2：Accepted 之后永不修改；纠正错误请新建一份 PDR 并标注 Supersedes。*
