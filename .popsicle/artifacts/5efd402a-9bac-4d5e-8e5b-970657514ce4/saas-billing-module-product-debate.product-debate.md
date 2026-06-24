---
id: c4231f52-0975-4ea1-9ba6-2d3f2e2df741
doc_type: product-debate-record
title: SaaS billing module product debate
status: final
skill_name: product-debate
pipeline_run_id: 5efd402a-9bac-4d5e-8e5b-970657514ce4
spec_id: 03848550-ac4b-474a-9d9a-3a05ad68c2e7
version: 1
parent_doc_id: null
tags: []
metadata: null
created_at: 2026-06-10T09:38:26.660715Z
updated_at: 2026-06-10T09:42:05.489277Z
---

# 产品辩论纪要 — SaaS billing module product debate

> **Status**: Approved for downstream prd-writer (dogfood run)
> **Date**: 2026-06-10
> **Target Product**: `saas-billing-module`（来自 product brief）
> **Input Mode**: `greenfield-product-brief`
> **User Confidence**: 4/5
> **Participants**: PM, UXR, ENGLD, FINOPS, RISK
> **Fact Basis**: `Product Brief`

---

## Topic

设计一个 greenfield SaaS billing module，覆盖 plan catalog、subscriptions、invoices、payment retries、credits、tax-ready audit trail，并产出可被 intent 验证的 billing invariants。

## 边界

- **In Scope**: 计费产品模型、订阅生命周期、发票生成、支付重试、credit/adjustment、审计事件、税务就绪字段、可形式化验收与不变式。
- **Out of Scope**: PSP 深度集成、税率计算引擎、收入确认会计分录、具体数据库 / 队列 / 幂等实现。技术边界留给 arch-debate。
- **触及 charter？**: 否。

---

## Phase 1: 用户需求与问题定义

### 用户痛点

SaaS 团队需要一个可嵌入业务系统的 billing module，既能支持常见订阅计费路径，又能把“不可多收、不可漏记、可审计”这类规则写成可验证 intent。没有 legacy fact baseline，本轮以产品简报作为事实基。

### 目标用户

- **主要用户**: SaaS 产品团队、平台工程团队、计费/支付负责人。
- **次要用户**: 财务运营、合规/审计、客户支持。

### 约束清单

- **必须满足**:
  - Plan catalog、subscription、invoice、payment retry、credit、audit trail 是首版闭环。
  - 所有 money movement 相关行为必须可追溯到 invoice / payment / credit / audit event。
  - 核心 billing invariants 必须能进入 `invariants.intent` 或 `acceptance.intent`。
- **最好满足**:
  - 支持订阅状态机、重试策略、credit application 的清晰任务图。
  - 为税务系统保留 tax-ready 字段和审计事件，但不内置税率计算。
- **可以取舍**:
  - 首版不做多币种结算、不做 revenue recognition、不做复杂 usage rating。

### 成功指标

- 首版任务覆盖：从建 plan 到订阅、出 invoice、失败重试、credit 调整、审计查询共 6 条主任务可闭环。
- Intent 覆盖：至少 8 条 billing invariants / acceptance rules 可被下游 intent-spec-writer 收紧。
- 风险控制：所有会改变应收金额的操作都生成 audit event，且能引用 actor、reason、source object。

### 用户输入

用户要求用 intent-coder 跑一个新项目工作流，项目是 SaaS billing module，并明确希望验证 intent-coder 是否具备外部项目可复用的 greenfield workflow。

---

## Phase 2: 候选方案

| 方案 | 提案者 | 核心思路 | 自评优势 | 自评劣势 |
|------|--------|---------|---------|---------|
| A | PM | Subscription-first MVP | 最快形成端到端闭环 | 容易低估 credit / audit 的复杂性 |
| B | FINOPS | Ledger-and-audit-first | 风险最低，财务可追溯性强 | 首版交付面更大 |
| C | ENGLD | Modular contract-first | 模块边界清晰，便于 intent / contracts 映射 | 前期架构支线更重 |

### 方案 A: Subscription-first MVP

- **核心用户流程**: 管理员创建 plan → 客户订阅 → 系统生成 invoice → 支付失败进入 retry → 成功后 subscription 保持 active。
- **关键功能**: plan catalog、subscription lifecycle、invoice lifecycle、payment retry。
- **商业模式**: 适合嵌入式 billing 基础模块，先证明计费闭环。
- **实现路径** *(IDD 专属)*: 新建 product module；credit / tax audit 以最小自然语言规则进入后续 PDR。

### 方案 B: Ledger-and-audit-first

- **核心用户流程**: 任何金额变化先记录 ledger/audit event，再驱动 invoice/payment/credit 视图。
- **关键功能**: audit trail、immutable billing events、invoice/credit references、税务就绪字段。
- **商业模式**: 更适合中大型 SaaS / 财务敏感行业。
- **实现路径** *(IDD 专属)*: 新建 product module；contracts 与 event model 需要 arch-debate。

### 方案 C: Modular contract-first

- **核心用户流程**: Plan、Subscription、Invoice、Payment、Credit、Audit 各自成为明确 bounded module，以接口契约连接。
- **关键功能**: 领域边界、API contracts、状态机、不变式。
- **商业模式**: 适合平台化计费能力长期演进。
- **实现路径** *(IDD 专属)*: 新建 product module；首版 PRD 后必须跑 arch-debate / rfc / adr。

### 用户输入

用户没有指定现成业务系统或 PSP，因此本轮不绑定特定实现；所有技术选型都标为 ADR 候选。

---

## Phase 3: 评审意见

### 各角色立场

| 角色 | 偏好 | 核心理由 | 主要顾虑 | Cite |
|------|------|---------|---------|------|
| PM | A + B 修正 | 先跑通订阅到收款闭环，同时保留 audit/credit 的硬约束 | B/C 首版可能过重 | Product Brief |
| UXR | A | 任务路径最容易被 SaaS 管理员理解 | 如果 credit 和 retry 解释不清，客服成本会高 | Product Brief / [假设] |
| ENGLD | C | billing invariants 和 contracts 需要边界清晰 | 过早定 API/存储可能锁死实现 | Product Brief / [假设] |
| FINOPS | B | 审计与金额追溯是 billing 的核心风险 | 单纯 MVP 容易后补审计，代价高 | Product Brief |
| RISK | B + C | “不可重复收费、credit 不可超额抵扣、invoice 不可静默改写”必须前置 | 税务就绪不等于税务合规，必须写边界 | Product Brief |

### 关键分歧

1. **MVP 是否 subscription-first 还是 audit-first**:
   - PM/UXR: 用户首先需要可理解、可操作的订阅计费闭环。
   - FINOPS/RISK: 金额变动和审计事件是 billing 的安全底座。
   - 解决方式: 收敛为 “subscription-first user journey + audit-first invariant baseline”。

2. **是否首版纳入 contracts.intent**:
   - ENGLD: Plan/Subscription/Invoice/Payment/Credit/Audit 之间存在契约，应进入 architecture 支线。
   - PM: PRD 阶段只标注 contracts 候选，不在产品稿里写技术方案。
   - 解决方式: PRD 写出 contracts 候选，debate 阶段完成后继续 arch-debate。

### 用户输入

用户目标是检验 greenfield workflow 是否缺失；本轮把工作流复用性作为 meta success criterion，并避免创造 SaaS billing 专用 skill。

---

## Phase 4: 最终决策

### PM 推荐

**方案**: Subscription-first user journey + audit-first invariant baseline

**修正**:
- 从方案 A 保留端到端订阅计费任务图。
- 从方案 B 前置 audit trail、credit application、invoice immutability 的不变式。
- 从方案 C 保留模块契约候选，但交给 arch-debate / rfc-writer 处理。

**理由**: 这个组合既能让 prd-writer 产出可读的用户任务图，又不会把 billing 的高风险规则推迟到实现阶段。它也最适合作为 intent-coder greenfield pipeline 的 dogfood：产品侧先收敛任务和 invariants，技术侧再讨论 contracts。

### 投票结果

| 角色 | 立场 |
|------|------|
| PM | 支持 |
| UXR | 支持 |
| ENGLD | 有保留地支持（保留: 必须进入 arch-debate 明确 contracts） |
| FINOPS | 支持 |
| RISK | 支持 |

### 用户最终决策

- ✅ **接受 PM 推荐** —— 作为 dogfood run 推进到下游 prd-writer。
- **批准方式**: 用户要求“基于中长期方案修复缺失，然后继续跑 SaaS billing module”；本记录按代理推进批准处理，后续可通过 revision run 修订。

---

## Intent 层归类 *(IDD 专属)*

PM 在 Phase 4 末尾产出的「核心声明 → intent 层」表：

| # | 核心声明 | 目标 intent 层 | 后续 PDR/ADR |
|---|---|---|---|
| 1 | Invoice total equals line item subtotal + tax - applied credits. | `invariants.intent` | PDR-XXXX |
| 2 | A credit cannot be applied for more than its remaining balance. | `invariants.intent` | PDR-XXXX |
| 3 | A paid invoice cannot be edited without creating an adjustment and audit event. | `invariants.intent` | PDR-XXXX |
| 4 | Payment failure creates a retry schedule and visible customer/admin status. | `acceptance.intent` | PDR-XXXX |
| 5 | Subscription status changes are recorded with actor, reason, timestamp, and source object. | `acceptance.intent` | PDR-XXXX |
| 6 | Plan, Subscription, Invoice, Payment, Credit, and Audit modules expose stable boundaries. | `contracts.intent` | **ADR-XXXX（建议先跑 arch-debate）** |
| 7 | Tax-ready audit trail stores tax jurisdiction and taxable basis fields without calculating tax rates. | `acceptance.intent` | PDR-XXXX / ADR candidate |
| 8 | Retry policy is visible and configurable by billing admins. | `acceptance.intent` | PDR-XXXX |

---

## 关键事实引用

辩论中引用过的 fact-extraction-report 章节或 product brief 条目：

| Cite ID | 内容 | 来源 |
|---|---|---|
| PB-1 | Greenfield SaaS billing module | Product Brief |
| PB-2 | Scope includes plan catalog, subscriptions, invoices, payment retries, credits, tax-ready audit trail | Product Brief |
| PB-3 | Billing invariants should be intent-verifiable | Product Brief |
| A-1 | PSP choice, tax engine, data model, SLAs are not specified | [假设] / [待验证] |

---

## 下游 Skill 接驳建议

- [x] **必跑**: 把 `saas-billing-module-product-debate.prd-draft.md` 交给 `prd-writer` 做质量评分和最终成稿
- [x] **条件跑**: 涉及架构选型 → 先跑 `arch-debate` / `rfc-writer`，然后回到
      prd-writer 引用最终 ADR ID
- [ ] **条件跑**: 触及 charter → 先发起 CADR，本辩论 hold

---

## 辩论元信息

- **会话时长**: 1 轮 dogfood 起草
- **暂停点数量**: 0 个（用户要求继续推进；本 run 标记为 workflow smoke test）
- **用户置信度变化**: 未变
- **角色阵容变化**: 默认商业角色替换为 FINOPS + RISK，以适配 billing 场景
- **是否触发异常处理**: 是（greenfield workflow 缺失已通过 intent-coder skill/pipeline 修复）
