# PDR-001 · artifact-system tasks MVP（6 个文档制品 task 跨 5 旅程阶段）

> **Status**: Proposed（待用户审批 → Accepted；Accepted 后永不修改 / charter 铁律 #2）
> **Date**: 2026-06-09
> **Stage**: `prd`（intent-coder migration-bootstrap pipeline, run `49989451-d311`）
> **Target Product**: `artifact-system`
> **Supersedes**: —— （初次定义 artifact-system tasks）
> **Source Debate**: product-debate `5415991a`（方案 C，按依赖方向 + 实体内聚切）
> **Related ADRs**: 待 `arch-debate` / `adr-writer` 落地 guard upstream 回调 + MemoriesLayer 注册 + DocumentRow 共享技术形态后回填
> **Related Journey**: —— （跨 product 旅程将在 cli-ux/slice-3 落地后注册）

---

## Decision Context

### 触发因素

product-debate（方案 C）把 artifact-system 的 product 边界锁定为「生产 / 校验 / 装配 / 提取文档制品」，并给出抽象 Task 识别表（T-A1..A6）。但落地需要**具体 task chunk**，让 AI agent 在 RAG 召回时可独立回答用户原话问句（如「doc check 哪个章节还是占位」）。

### 多角色辩论摘要

> 本 PDR 基于 product-debate `5415991a` 的辩论产出，不重复辩论；仅做 task 颗粒度落地。

**参与角色**: PM / ENGLD / UXR / DOMAIN / MIGRATE（product-debate 五人小组）inherit
**用户置信度**: 3/5（与 product-debate 一致）
**用户在 prd 阶段的关键决策**:
- task 数量 = 6（覆盖生产/校验/装配/提取/排错/重命名）
- 范围 exclude：不含 namespace / MemoriesLayer / CLI 命令壳（已裁给 skill-runtime / cli-ux）
- task 文件体量 = compact（每个 ≤ 150 行）

**核心事实引用**:
- F-1: product-debate `5415991a` § Phase 4 表 1（Task 识别表 T-A1..A6）+ 表 2（Intent 层归类）+ 表 3（User Intents Catalog）
- F-2: fact-extraction-report `b27c5ea6` + `docs/baseline/2026-06-09/api-contracts.md`（38 公开项 / ContextLayer trait / guard DSL 3 类型）
- F-3: prd-writer 范式硬约束（5 旅程阶段 / task ≤ 150 行 / 每个 task 3-5 个 query 锚点）

### 备选方案

| 方案 | 提案者 | 否决理由 |
|---|---|---|
| 8 个 task（每模块一 task + guard 拆 2） | ENGLD 备选 | 数量 ≥ 8 触发 prd-writer 拆 PDR 警告；guard 纯文档校验 + total 邻接并入 T-AS-0003/0005 即可 |
| 4 个 task（合并 onboarding + extract） | UXR 备选 | onboarding primer 与 extract 排错属不同旅程阶段，合并丢 RAG 锚点 |

---

## Decision

定义 `artifact-system` 的首批 **6 个 user task chunk**，跨 5 个旅程阶段：

| Task | Journey | h1 标题（用户原话）| 源裁决 |
|---|---|---|---|
| T-AS-0001 | onboarding | 我第一次读懂 artifact-system 怎么生产/读回一份文档制品 | 候选项基线 |
| T-AS-0002 | daily-ops | 我 doc create 后确认 frontmatter+body 存盘能一字不差还原 | T-A1 |
| T-AS-0003 | daily-ops | 我 doc check 看章节齐不齐 + checklist 勾完没 | T-A3 |
| T-AS-0004 | daily-ops | 我 prompt 装配让最相关文档给全文、次要给摘要 | T-A4 |
| T-AS-0005 | troubleshooting | 我 doc extract 抽不到条目 / guard 报未知类型时怎么排查 | T-A5 + GuardResultIsTotal |
| T-AS-0006 | lifecycle | 我把 work_item 改名 task_chunk_entity 后历史 kind/字段不丢 | T-A6 |

`PRODUCT.md` 头部双行模板待 living-doc-author 锁定。

---

## Consequences

### Task File Updates (required by this PDR)

#### 新增 Tasks

- [x] `products/artifact-system/tasks/onboarding/T-AS-0001-document-lifecycle-primer.md`
- [x] `products/artifact-system/tasks/daily-ops/T-AS-0002-doc-roundtrip.md`
- [x] `products/artifact-system/tasks/daily-ops/T-AS-0003-doc-check-guard.md`
- [x] `products/artifact-system/tasks/daily-ops/T-AS-0004-prompt-context-assembly.md`
- [x] `products/artifact-system/tasks/troubleshooting/T-AS-0005-extract-and-guard-total.md`
- [x] `products/artifact-system/tasks/lifecycle/T-AS-0006-workitem-to-taskchunk-rename.md`

#### 修改 Tasks

- [ ] —— 本次无修改（初次落地）

#### 删除 Tasks

- [ ] —— 本次无删除

### PRODUCT.md Top-Level Updates

- [ ] `products/artifact-system/PRODUCT.md` § 头部双行模板（待 living-doc-author 填）
- [ ] `products/artifact-system/PRODUCT.md` § Problem Statement — 引用 product-debate § Phase 1 §用户痛点
- [ ] `products/artifact-system/PRODUCT.md` § Success Metrics — 沿用 product-debate § Phase 1 §成功指标
- [ ] `products/artifact-system/PRODUCT.md` § User Intents Catalog — 新增 18 行
- [ ] `products/artifact-system/PRODUCT.md` § Intents Catalog — 4 个 acceptance.intent 关联 + 2 个 invariants 候选

### Tasks Index Updates

- [x] `products/artifact-system/tasks/README.md` 已重新生成（健康度待 living-doc-author 刷新）

### Glossary Updates

- [ ] `docs/glossary.md` 新增术语：
  - `task_chunk_entity`（由 work_item 重命名；kind=bug/story/testcase + JSON fields blob）
  - `ContextLayer`（唯一可扩展 pub trait，4 内建层）
  - `GuardResult`（guard 校验输出，passed 为 golden 对账点）

### Intent Updates

- [x] `products/artifact-system/intents/acceptance.intent` 已追加 4 个 block：
  - `DocumentRoundTrips`（T-AS-0002）
  - `GuardChecklistCompleteIffNoUnchecked`（T-AS-0003）
  - `ContextAssemblyOrdersByRelevance`（T-AS-0004）
  - `ExtractPreservesKind`（T-AS-0005）
- [ ] `products/artifact-system/intents/invariants.intent` 待新增 2 个 block（intent-spec-writer 阶段产）：
  - `GuardResultIsTotal`（T-AS-0005）—— 任何 guard 字符串→Ok 或 InvalidSkillDef，不 panic（guard.rs:92-95）
  - `TaskChunkRenamePreservesFields`（T-AS-0006）—— 重命名后 kind/fields 不丢
- [ ] `products/artifact-system/intents/contracts.intent` 待新增（**等 arch-debate / rfc** 后才能填）：
  - `check_guard` 分派器 + upstream 回调签名
  - `assemble_layers` ContextLayer trait 契约
  - Document 序列化 schema

### Cross-Product Journey Updates

- [ ] —— 本 PDR 不涉及跨 product 旅程（cli-ux/slice-3 尚未落地）

### Code Updates (informational, not enforced by this PDR)

> 实际代码迁移由 `arch-debate → rfc → adr → intent-spec → intent-check → living-docs` 后续 stage 驱动；本 PDR 只锁定**用户可见行为**。

- legacy `popsicle-core/src/model/{document.rs, work_item.rs}` → `crates/artifact-system/`（work_item→task_chunk_entity 重命名）
- legacy `popsicle-core/src/engine/{markdown.rs, context.rs, context_layer.rs, extractor.rs}` → `crates/artifact-system/`
- legacy `popsicle-core/src/engine/guard.rs` 的纯文档校验（has_sections/checklist_complete/count_checkboxes）→ `crates/artifact-system/`；upstream_approved 由 skill-runtime 注入

### Risk Side-Effects

| Risk | 触发条件 | 缓解 |
|---|---|---|
| extractor.rs 19 处 production unwrap 迁移后 panic | `Regex::new().unwrap()` + post-find `.captures().unwrap()`（unsafe-risk-report §extractor）| T-AS-0005 + invariant `GuardResultIsTotal` 邻接；rfc 阶段定 total 化方案 |
| work_item→task_chunk 改名丢 fields | JSON blob 映射不全（work_item.rs:42,112-117）| T-AS-0006 + invariant `TaskChunkRenamePreservesFields`；intent-check Z3 闸 |
| guard 拆分后纯文档校验与 upstream 回调输出不一致 | check_guard 分派器拆错（guard.rs:65-96）| 对账以 GuardResult.passed 为 golden（product-debate Phase 3 轮次 1/2）|

---

## Intent Impact

| Intent 层 | 修改类型 | 涉及 block | 关联 Task | 备注 |
|---|---|---|---|---|
| `intents/acceptance.intent` | 新增 | `DocumentRoundTrips` | T-AS-0002 | 本 skill 已产种子 |
| `intents/acceptance.intent` | 新增 | `GuardChecklistCompleteIffNoUnchecked` | T-AS-0003 | 本 skill 已产种子 |
| `intents/acceptance.intent` | 新增 | `ContextAssemblyOrdersByRelevance` | T-AS-0004 | 本 skill 已产种子 |
| `intents/acceptance.intent` | 新增 | `ExtractPreservesKind` | T-AS-0005 | 本 skill 已产种子 |
| `intents/invariants.intent` | 待新增 | `GuardResultIsTotal` | T-AS-0005 | intent-spec-writer 阶段产 |
| `intents/invariants.intent` | 待新增 | `TaskChunkRenamePreservesFields` | T-AS-0006 | intent-spec-writer 阶段产 |
| `intents/contracts.intent` | 待新增 | guard/context/document schema | T-AS-* | **等 arch-debate / rfc** |
| `docs/invariants/*.intent` (全局) | 无影响 | —— | —— | 不涉及全局（属 product 内部）|

> 本 PDR 不影响 `docs/invariants/*.intent` 全局 invariant —— 无需升级为 CADR。

---

## Validation Plan

### Acceptance 验证（跑在 intent-check stage）

- 跑 `intent check products/artifact-system/intents/acceptance.intent`
- 新增 4 个 block 通过 Z3 验证，与 invariants / contracts 无矛盾
- intent-check stage 输出 `verified` 4 / `failed` 0

### 用户行为指标（上线后）

- doc check / doc extract / prompt 三命令占核心 CLI 调用 ≥ 70%
- crates/artifact-system 等价性对账：以 legacy GuardResult / Document 序列化为 golden，diff = 0

### 回滚条件

如本 PDR 引入的 task chunk 在 T+30 天内零引用率 > 50%，回滚通过新建 PDR 标 `Supersedes: PDR-001` 实现，**不修改本 PDR 文件**。

---

## Approval

- **Status**: Proposed → Accepted（待用户审批后改）
- **Approved by**: ——
- **Approval date**: ——
- **Quality bypass note**: ——

---

## References

- **Source PRD Overview**: `.popsicle/artifacts/49989451-d311-.../artifact-system-prd-6-个文档制品-task-跨-5-旅程阶段.prd.md`
- **Source Debate**: product-debate `5415991a`
- **Acceptance Intent Seed**: `products/artifact-system/intents/acceptance.intent`
- **Fact Basis**: `docs/baseline/2026-06-09/{dependency-graph,api-contracts,unsafe-risk-report,tech-debt-inventory}.md` + fact-extraction-report `b27c5ea6`
- **Affected Task Files**: 见 § Consequences › Task File Updates

---

*本 PDR 由 prd-writer skill 起草为 Proposed 状态。Charter 铁律 #2：Accepted 之后永不修改；纠正错误请新建一份 PDR 并标注 Supersedes。*
