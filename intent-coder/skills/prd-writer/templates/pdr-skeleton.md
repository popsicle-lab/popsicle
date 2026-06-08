# PDR-{id}: {title}

> **Status**: Proposed
> **Date**: {YYYY-MM-DD}
> **Target Product**: `{target_product}`
> **Decision Type**: Product Decision Record (PDR)
> **Supersedes**: —— / `PDR-XXXX`（如本决策替换既有决策）
> **Related ADRs**: —— / `ADR-XXXX`（如涉及技术架构）
> **Related Journey**: —— / `J-XXXX`（如本决策属于跨 product 旅程的一部分）

---

## Decision Context

### 触发因素

{是什么事件 / 痛点 / 数据让我们必须现在做这个决策？}

### 多角色辩论摘要

> 如果本 PDR 经过 `product-debate`，本段从 `{slug}.product-debate.md` 继承关键
> 论据；否则标注「未经多角色辩论」并简述用户访谈要点。

**参与角色**: {PM, UXR, GROWTH, ENGLD, BIZ ...}

**用户置信度**: {N}/5

**关键分歧**:
- {分歧 1}: {角色 A 立场 vs 角色 B 立场}
- {分歧 2}: ...

**核心事实引用**（来自 fact-extraction-report）:
- F-1: {引用内容}
- F-2: {引用内容}

### 备选方案

| 方案 | 提案者 | 否决理由 |
|------|--------|---------|
| {方案 B} | {角色} | {理由} |
| {方案 C} | {角色} | {理由} |

---

## Decision

> 一句到三句话陈述「选了什么」。**不解释理由**（理由在 Decision Context 已说）。
> 不写「将会」「计划」。

{决策陈述，例：「采用本地 CLI 直接调用 verifier 的同步 RPC 调用模式，单次验证
响应时间硬上限 5 秒，超时返回明确错误码而非降级到异步。」}

---

## Consequences

> 本 PDR Accepted 之后，**所有**被强制更新的活文档段落和 intent 文件必须在同一
> 个 PR 里同步更新（charter 第 4 条铁律）。
>
> ⚠️ 在任务图范式下，PRD 的「章节」已经下沉成**独立的 task 文件**。Consequences
> 因此精确到「文件级」——这比旧范式的「PRODUCT.md § 某章节」更精确，CI 也能
> 直接 grep 文件路径校验。

### Task File Updates (required by this PDR)

#### 新增 Tasks

- [ ] `products/{target_product}/tasks/onboarding/T-0001-first-time-verify.md`
- [ ] `products/{target_product}/tasks/onboarding/T-0002-handle-fail-counterexample.md`
- [ ] `products/{target_product}/tasks/daily-ops/T-0010-batch-verify.md`

#### 修改 Tasks

- [ ] —— 本次无修改

#### 删除 Tasks

- [ ] —— 本次无删除

### PRODUCT.md Top-Level Updates

- [ ] `products/{target_product}/PRODUCT.md` § Problem Statement — 替换为本 PDR § Decision Context
- [ ] `products/{target_product}/PRODUCT.md` § Success Metrics — 新增 {N} 个 KPI
- [ ] `products/{target_product}/PRODUCT.md` § User Intents Catalog — 新增 {N} 行问句→task 映射
- [ ] `products/{target_product}/PRODUCT.md` § Intents Catalog — 新增 acceptance.intent 关联

### Tasks Index Updates

- [ ] `products/{target_product}/tasks/README.md` 重新生成
       （`popsicle skill start living-doc-author --target tasks-index`）

### Glossary Updates

- [ ] `docs/glossary.md` 新增术语：{术语 1}, {术语 2}（如适用）

### Intent Updates

- [ ] `products/{target_product}/intents/acceptance.intent` 追加 block:
      `T-0001-response-under-5s`, `T-0002-counterexample-minimal`, `T-0010-batch-mode-no-partial-fail`
- [ ] `products/{target_product}/intents/invariants.intent` 新增 block:
      `no-concurrent-verification`（**非本 skill 输出**——由项目自带 spec writer 落地）
- [ ] `products/{target_product}/intents/contracts.intent` 新增 block:
      `verifier-api-v1`（**等 ADR-XXXX 落地后**才能填）

### Cross-Product Journey Updates

- [ ] —— 本 PDR 不涉及跨 product 旅程
- 或 [ ] `docs/user-journeys/J-XXXX-{slug}.md` § Stage N 加本 product 的 task 引用

### Code Updates (informational, not enforced by this PDR)

> 仅作为 informational。代码修改由后续的 implementation skill / PR 驱动，不属
> 于本 PDR 的强制 Consequences。

- 模块 `crates/verifier/`：实现 `Verify(spec) -> Result` 同步 RPC
- 模块 `crates/cli/`：实现 `intent check <file>` 子命令

### Risk Side-Effects

| Risk | 触发条件 | 缓解 |
|------|---------|------|
| {风险 1} | {何时会发生} | {措施} |

---

## Intent Impact

> charter 强制：每份 PDR 必须显式声明它修改的是哪一层 intent。CI 拒绝缺这一项
> 的决策。

| Intent 层 | 修改类型 | 涉及 block | 关联 Task | 备注 |
|-----------|---------|----------|----------|------|
| `intents/acceptance.intent` | 新增 | `T-0001-response-under-5s` | T-0001 | 本 skill 产种子 |
| `intents/acceptance.intent` | 新增 | `T-0002-counterexample-minimal` | T-0002 | 本 skill 产种子 |
| `intents/acceptance.intent` | 新增 | `T-0010-batch-mode-no-partial-fail` | T-0010 | 本 skill 产种子 |
| `intents/invariants.intent` | 新增 | `no-concurrent-verification` | —— (跨 task) | 项目自带 spec writer 收紧 |
| `intents/contracts.intent` | 待新增 | `verifier-api-v1` | T-0001 / T-0010 | **等 ADR-XXXX** |
| `docs/invariants/*.intent` (全局) | 无影响 | —— | —— | 不涉及全局，无需 CADR |

> ⚠️ 如果本 PDR 影响 `docs/invariants/*.intent`（全局 invariant），它本身可能需
> 要升级为 **CADR**。

---

## Validation Plan

> 怎么验证本决策的预期效果？包含传统业务指标 + AI 反馈闭环指标。

### Acceptance 验证（T+1 周）

- 跑 `intent check products/{target_product}/intents/acceptance.intent`
- 新增 3 个 block 通过 Z3 验证，与已有 invariants / contracts 无矛盾

### 用户行为指标（T+30 天）

- {指标 1}: 目标 {阈值}
- {指标 2}: 目标 {阈值}

### AI 反馈闭环指标（T+30 天 / T+90 天）

> 文章 strategy 5 在 PDR 层的落地——每个 PDR 都要预留 AI 反馈监控点。

| 指标 | T+30 天目标 | T+90 天目标 |
|---|---|---|
| Task chunk 召回次数 (T-0001) | ≥ 10 | ≥ 30 |
| Task chunk 召回次数 (T-0002) | ≥ 5 | ≥ 15 |
| AI 错答率（query 锚点对应的回答置信度 < 0.7 占比）| < 5% | < 3% |
| 用户转人工率（troubleshooting 类 task）| —— | —— |
| 零引用 task | 0 | 0（若 > 0 进入归档评审）|

### 回滚条件

如果 {度量} 在 {时间窗} 内劣化超 {阈值}，回滚到 {前一状态 / 前序 PDR}。
回滚通过新建一份 PDR 标注 `Supersedes: PDR-{id}` 实现，**不修改本 PDR 文件**。

---

## Approval

- **Status**: Proposed → Accepted（审批后由用户改）
- **Approved by**: {审批人}
- **Approval date**: {YYYY-MM-DD}
- **Quality bypass note**: ——（若 PRD 评分 < 90 且用户强制 pass，填理由）

---

## References

- **Source PRD Overview**: `{slug}.prd.md`
- **Source Debate**: `{slug}.product-debate.md`
- **Source Decision Matrix**: `{slug}.decision-matrix.md`
- **Acceptance Intent Seed**: `{slug}.acceptance.intent`
- **Fact Basis**: `{slug}.fact-extraction-report.md`
- **Affected Task Files**:
  - `products/{target_product}/tasks/onboarding/T-0001-first-time-verify.md`
  - `products/{target_product}/tasks/onboarding/T-0002-handle-fail-counterexample.md`
  - `products/{target_product}/tasks/daily-ops/T-0010-batch-verify.md`
- **Related Living Docs**:
  - `products/{target_product}/PRODUCT.md`
  - `products/{target_product}/tasks/README.md`

---

*本 PDR 由 prd-writer skill 起草为 Proposed 状态。Charter 第 2 条铁律：
Accepted 之后永不修改；纠正错误请新建一份 PDR 并标注 Supersedes。*
