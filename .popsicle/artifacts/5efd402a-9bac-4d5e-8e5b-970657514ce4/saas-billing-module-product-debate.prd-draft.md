# PRD 草稿 — SaaS billing module product debate

> **Status**: Draft (from product-debate skill — task-centric form)
> **Target Product**: `saas-billing-module`
> **Input Mode**: `greenfield-product-brief`
> **Source Debate**: `saas-billing-module-product-debate.product-debate.md`
> **Fact Basis**: `Product Brief`
>
> 本草稿尚未通过 `prd-writer` 的质量评分；task_id 由 prd-writer 在分配阶段确定。

---

## 1. 本次变更的核心意图

SaaS 团队用一个 billing module 管理 plan、subscription、invoice、payment retry、credit 和 audit trail，并把关键计费规则收紧为 intent-verifiable invariants。

---

## 2. 背景与目标

### 用户痛点

订阅计费的产品路径、财务风险和审计要求天然耦合；如果只从功能列表起步，容易在实现后才发现 invoice total、credit application、paid invoice immutability 等规则无法验证。

### 目标用户

- **主要用户**: SaaS 产品团队、平台工程团队、计费/支付负责人。
- **次要用户**: 财务运营、合规/审计、客户支持。

### 产品目标

- 建立可读的 billing task graph，覆盖从 plan setup 到 audit query 的主要用户任务。
- 产出首批 billing invariants / acceptance seeds，驱动后续 intent-spec-writer。
- 把 contracts 候选留给 architecture 支线，避免 PRD 写死技术方案。

### 约束条件

- **必须满足**:
  - Plan catalog、subscription、invoice、payment retry、credit、audit trail 进入首版范围。
  - Money movement 相关行为必须可追溯。
  - 超出 Product Brief 的 PSP、税率、存储、SLA 判断标 `[待验证]`。
- **最好满足**:
  - Tax-ready 字段在 audit trail 中前置。
  - Retry policy 对 admin 可见、可配置。
- **可以取舍**:
  - 首版不做多币种结算、收入确认、复杂 usage rating。

---

## 3. 推荐方案

**选定方案**: Subscription-first user journey + audit-first invariant baseline

**修正与折中**:
- 用户任务以订阅计费闭环组织，避免财务/技术术语主导 PRD。
- Invariants 前置，尤其是 invoice total、credit balance、paid invoice immutability。
- 模块 contracts 只标 ADR 候选，交给 arch-debate。

### 决策理由

该方案让产品侧先完成可执行任务图，同时保留 billing 系统最敏感的安全/审计规则，不把高风险约束推迟到实现阶段。

### 被否决的备选

- **Pure subscription MVP**: 过度强调交付速度，audit 和 credit 可能后补困难。
- **Pure ledger-first**: 风险最低但首版用户路径不够清晰，容易变成财务系统 spec。
- **Pure contract-first**: 边界清晰但 PRD 期技术负担过重。

---

## 4. Tasks（核心产出，任务图范式）

### 4.1 候选 Tasks 概览

| 候选 ID | 用户原话标题 | Journey Stage | Audience | 备注 |
|---|---|---|---|---|
| TBD-1 | 我创建一个可销售的订阅 plan | onboarding | admin | 首次配置 |
| TBD-2 | 我给客户开通或变更订阅 | daily-ops | billing-admin | 订阅生命周期 |
| TBD-3 | 我查看并确认一张 invoice 的金额来源 | daily-ops | billing-admin / finance | invoice total invariant |
| TBD-4 | 我处理一次支付失败并看到重试状态 | troubleshooting | billing-admin / support | retry acceptance |
| TBD-5 | 我给客户发放并抵扣 credit | daily-ops | billing-admin / finance | credit invariant |
| TBD-6 | 我为审计导出某个客户的计费事件链 | lifecycle | finance / auditor | tax-ready audit trail |
| TBD-7 | 我配置默认支付重试策略 | admin | billing-admin | policy task |

### 4.2 每个 task 的草稿明细

#### TBD-1: 我创建一个可销售的订阅 plan

- **Journey Stage**: onboarding
- **Audience**: ["admin", "billing-admin"]
- **前提**: SaaS 产品已定义可售卖套餐。
- **限制**: 多币种和复杂 usage rating 标为 `[待验证]`。
- **本 task 可解答**:
  - 怎么创建一个新的订阅 plan？
  - plan 需要哪些价格和税务字段？
  - 下架 plan 后已有订阅会怎样？
- **完成路径（happy path）**:
  1. Admin 创建 plan，填写名称、计费周期、价格、tax-ready metadata。
  2. 系统校验 plan 可销售字段完整。
  3. Plan 进入 active 状态，可被 subscription 引用。
- **可观察的成功标志**: Active plan 可用于新 subscription。
- **Related Next Tasks**: TBD-2, TBD-3

#### TBD-2: 我给客户开通或变更订阅

- **Journey Stage**: daily-ops
- **Audience**: ["billing-admin", "support"]
- **前提**: 至少一个 active plan 存在。
- **限制**: Proration 规则标 `[待验证]`。
- **本 task 可解答**:
  - 怎么给客户开通订阅？
  - 客户升级/降级 plan 时状态怎么变化？
  - 取消订阅后还会不会继续出账？
- **完成路径（happy path）**:
  1. Admin 选择 customer 和 plan。
  2. 系统创建 subscription，并记录 actor/reason/source object。
  3. Subscription 状态进入 active 或 scheduled。
- **可观察的成功标志**: Subscription 状态变化生成 audit event。
- **Related Next Tasks**: TBD-3, TBD-6

#### TBD-3: 我查看并确认一张 invoice 的金额来源

- **Journey Stage**: daily-ops
- **Audience**: ["billing-admin", "finance"]
- **前提**: Subscription 已触发 invoice。
- **限制**: 税率计算由外部 tax service 负责 `[待验证]`。
- **本 task 可解答**:
  - invoice total 是怎么算出来的？
  - credit 和 tax 在 invoice 上怎么体现？
  - paid invoice 能不能直接改？
- **完成路径（happy path）**:
  1. 系统生成 invoice line items。
  2. 系统计算 subtotal、tax、applied credits、total due。
  3. 用户查看 invoice breakdown 和 audit references。
- **可观察的成功标志**: Invoice total 与 line item subtotal + tax - applied credits 一致。
- **Related Next Tasks**: TBD-4, TBD-5

#### TBD-4: 我处理一次支付失败并看到重试状态

- **Journey Stage**: troubleshooting
- **Audience**: ["billing-admin", "support"]
- **前提**: Invoice 进入 payment failed 状态。
- **限制**: PSP 错误码映射标 `[待验证]`。
- **本 task 可解答**:
  - 支付失败后系统会重试几次？
  - 客户和管理员在哪里看到失败原因？
  - 重试成功后 invoice 状态怎么变？
- **完成路径（happy path）**:
  1. Payment failure 被记录到 invoice。
  2. 系统生成 retry schedule。
  3. Admin 查看下一次 retry 时间、失败原因和当前状态。
- **可观察的成功标志**: Failure、retry schedule、最终 payment outcome 都有 audit event。
- **Related Next Tasks**: TBD-3, TBD-6

#### TBD-5: 我给客户发放并抵扣 credit

- **Journey Stage**: daily-ops
- **Audience**: ["billing-admin", "finance"]
- **前提**: Customer 存在，且有可解释的 credit reason。
- **限制**: Credit approval workflow 标 `[待验证]`。
- **本 task 可解答**:
  - 怎么给客户发 credit？
  - credit 能不能超过 invoice 金额？
  - credit 用完后怎么追踪剩余额度？
- **完成路径（happy path）**:
  1. Admin 创建 credit，填写 reason 和 amount。
  2. 系统在 invoice 上应用不超过剩余余额的 credit。
  3. Credit remaining balance 和 audit event 更新。
- **可观察的成功标志**: Credit applied amount 不超过 remaining balance。
- **Related Next Tasks**: TBD-3, TBD-6

#### TBD-6: 我为审计导出某个客户的计费事件链

- **Journey Stage**: lifecycle
- **Audience**: ["finance", "auditor"]
- **前提**: Customer 有 billing events。
- **限制**: 税务申报格式不在首版范围。
- **本 task 可解答**:
  - 怎么查看客户的完整计费事件？
  - 某次 invoice total 的来源能追溯到哪些对象？
  - audit trail 里有哪些 tax-ready 字段？
- **完成路径（happy path）**:
  1. 用户按 customer 和时间范围查询 audit trail。
  2. 系统返回 plan/subscription/invoice/payment/credit events。
  3. 用户导出带 actor、reason、source object、tax-ready fields 的事件链。
- **可观察的成功标志**: 任一金额变化能追溯到 source object 和 actor。
- **Related Next Tasks**: TBD-3, TBD-5

#### TBD-7: 我配置默认支付重试策略

- **Journey Stage**: admin
- **Audience**: ["billing-admin"]
- **前提**: Module 已启用 payment retry。
- **限制**: 不指定具体 PSP。
- **本 task 可解答**:
  - 怎么配置默认重试次数？
  - 不同失败原因能不能不同策略？
  - 修改策略会不会影响已有 failed invoice？
- **完成路径（happy path）**:
  1. Admin 设置 retry count、interval、stop condition。
  2. 系统校验策略不违反最大重试边界。
  3. 新的 failed payments 使用该策略。
- **可观察的成功标志**: 新 payment failure 生成符合策略的 retry schedule。
- **Related Next Tasks**: TBD-4

---

## 5. User Intents Catalog（草稿）

| User Query | → Task | Journey Stage | Audience |
|---|---|---|---|
| 「怎么创建一个新的订阅 plan？」 | TBD-1 | onboarding | admin |
| 「plan 需要哪些价格和税务字段？」 | TBD-1 | onboarding | billing-admin |
| 「怎么给客户开通订阅？」 | TBD-2 | daily-ops | billing-admin |
| 「invoice total 是怎么算出来的？」 | TBD-3 | daily-ops | finance |
| 「支付失败后系统会重试几次？」 | TBD-4 | troubleshooting | support |
| 「怎么给客户发 credit？」 | TBD-5 | daily-ops | finance |
| 「怎么查看客户的完整计费事件？」 | TBD-6 | lifecycle | auditor |
| 「怎么配置默认重试次数？」 | TBD-7 | admin | billing-admin |

---

## 6. Intent Mapping（核心声明 → intent 层）

| # | 核心声明 | 目标 intent 层 | 关联 Task | 候选 block 名 | 备注 |
|---|---|---|---|---|---|
| 1 | Invoice total equals line item subtotal + tax - applied credits. | `invariants.intent` | TBD-3 | `invoice-total-balanced` | Product Brief |
| 2 | A credit cannot be applied for more than its remaining balance. | `invariants.intent` | TBD-5 | `credit-application-within-balance` | Product Brief |
| 3 | A paid invoice cannot be edited without adjustment and audit event. | `invariants.intent` | TBD-3 / TBD-6 | `paid-invoice-adjustment-only` | Product Brief |
| 4 | Payment failure creates retry schedule and visible status. | `acceptance.intent` | TBD-4 / TBD-7 | `T-XXXX-payment-failure-retry-visible` | Product Brief |
| 5 | Subscription status changes are recorded with actor, reason, timestamp, source object. | `acceptance.intent` | TBD-2 / TBD-6 | `T-XXXX-subscription-status-audited` | Product Brief |
| 6 | Tax-ready audit trail stores tax jurisdiction and taxable basis fields without calculating tax rates. | `acceptance.intent` | TBD-1 / TBD-6 | `T-XXXX-tax-ready-audit-fields` | Product Brief |
| 7 | Plan, Subscription, Invoice, Payment, Credit, and Audit expose stable boundaries. | `contracts.intent` | all | `billing-module-contracts` | ADR 候选 |

---

## 7. Out of Tasks

- 不做 PSP 具体集成。
- 不做税率计算引擎。
- 不做 revenue recognition。
- 不做复杂 usage rating / metered billing。
- 不做多币种结算。

---

## 8. 成功指标

| 指标 | 当前基线 | 目标值 | 衡量方式 |
|------|---------|--------|---------|
| 主任务覆盖 | 0 | 7 个 task 覆盖 5 个 journey stage | prd-writer task manifest |
| 可形式化规则 | 0 | ≥ 8 条 invariants / acceptance candidates | intent-spec-writer 输入 |
| 审计追溯 | 未定义 | 金额变化均有 audit event | acceptance.intent / review |

---

## 9. 风险与缓解

| 风险 | 概率 | 影响 | 缓解措施 | Affected Tasks |
|------|------|------|---------|----------------|
| 把 tax-ready 误写成 tax-compliant | 中 | 高 | 明确 tax rate calculation out of scope | TBD-1, TBD-6 |
| Credit 规则不清导致超额抵扣 | 中 | 高 | 写入 invariants.intent | TBD-5 |
| Retry policy 与 PSP 行为耦合 | 中 | 中 | PSP mapping 标 ADR / [待验证] | TBD-4, TBD-7 |
| Module contracts 过早冻结 | 中 | 中 | arch-debate 后再写 contracts.intent | all |

---

## 10. 事实基引用

| Cite ID | 内容 | 来源 |
|---|---|---|
| PB-1 | Greenfield SaaS billing module | Product Brief |
| PB-2 | plan catalog, subscriptions, invoices, payment retries, credits, tax-ready audit trail | Product Brief |
| PB-3 | intent-verifiable billing invariants | Product Brief |
| A-1 | PSP, tax service, SLA, storage 未指定 | [待验证] |

---

## 11. 辩论元信息

- **辩论文件**: `saas-billing-module-product-debate.product-debate.md`
- **参与角色**: PM, UXR, ENGLD, FINOPS, RISK
- **用户置信度**: 4/5
- **投票结果**: 综合方案 5 票支持，其中 ENGLD 有保留
- **是否用户决策覆盖**: 否
- **关键分歧**: MVP 范围与 audit/contracts 前置程度

---

## 12. 下游 Skill 接驳

- [ ] 把本草稿喂给 `prd-writer`。
- [ ] prd-writer 分配正式 task_id 和 PDR ID。
- [ ] 含 `contracts.intent` 行，PRD 后继续 `arch-debate` / `rfc-writer` / `adr-writer`。

---

## Task-Centric 形态检查清单

- [x] §4 是「Tasks」段。
- [x] 每个 task 标了 5 个旅程阶段之一。
- [x] 每个 task 有用户原话标题。
- [x] 每个 task 有 ≥ 3 个用户原话问句。
- [x] §5 User Intents Catalog 覆盖所有 task。
- [x] §6 Intent Mapping 表完整且每条标了 intent 层。
- [x] §7 Out of Tasks 显式列出 ≥ 2 项。
