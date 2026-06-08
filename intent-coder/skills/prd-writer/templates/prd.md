# PRD Overview — {feature_or_intent_title}

> **本文件是一次 PRD 变更的「清单 + 概览」，不是一份独立可读的 PRD。**
> 它定义本次变更（由 `PDR-{id}` 引入）会创建/修改的所有 task 文件和 PRODUCT.md 顶层片段。
>
> 落地时**不**把本文件直接放进 `products/<product>/`——而是按下方「文件清单」
> 把内容分别合并到对应位置。
>
> **Status**: Draft → Review → Approved
> **Target Product**: `{target_product}`
> **Source Debate**: `{slug}.product-debate.md` ✅ / `[未经辩论]`
> **PDR**: `PDR-{id}-{slug}.md`
> **Quality Score**: {N}/100（评分维度见 `references/quality-rubric.md`）
> **Last-Updated**: {YYYY-MM-DD}

---

## 1. 本次变更的核心意图

> 一句话陈述：**用户原本能 / 不能 X，本次变更后用户能 Y**。
> 用现在时，不用「将会」「计划」。

{例：用户能在本地用 `intent check` 命令对单个 .intent 文件做 5 秒内的验证并拿到 PASS/FAIL 结果。}

---

## 2. Problem Statement（合并到 PRODUCT.md › Problem Statement 段）

**Current Situation**: {当前痛点 — cite fact-extraction-report § X}

**Proposed Solution**: {高层描述 — 现在时}

**Business Impact**: {可量化产出}

`Decision-Ref: PDR-{id}` | `Fact: <slug>.fact-extraction-report.md § X`

---

## 3. Success Metrics（合并到 PRODUCT.md › Success Metrics 段）

| Metric | Baseline | Target | Measurement | Cite |
|---|---|---|---|---|
| {首次验证成功率} | {当前值} | {目标值} | {方法} | `fact-ext §X` |
| {P95 响应时间} | {当前值} | {目标值} | {方法} | `[未经事实基验证]` |

`Decision-Ref: PDR-{id}`

---

## 4. 文件清单（本次变更涉及的所有文件）

> 这是 PDR Consequences 的镜像——两者**必须**一致。

### 新增 Tasks

| Task ID | 标题 | Journey Stage | 文件路径 |
|---|---|---|---|
| T-0001 | 我第一次跑 intent check 直到拿到 PASS | onboarding | `products/{target_product}/tasks/onboarding/T-0001-first-time-verify.md` |
| T-0002 | 拿到 FAIL 后看反例并调整 spec | onboarding | `products/{target_product}/tasks/onboarding/T-0002-handle-fail-counterexample.md` |
| T-0010 | 批量验证一组 .intent 文件 | daily-ops | `products/{target_product}/tasks/daily-ops/T-0010-batch-verify.md` |

> 每个新增 task 都按 `templates/task.md` 单独成文。本文件**不**复制 task 正文。

### 修改 Tasks

| Task ID | 修改说明 | Supersedes |
|---|---|---|
| —— | —— | —— |

### 删除 Tasks

| Task ID | 废止理由 | Supersedes |
|---|---|---|
| —— | —— | —— |

### PRODUCT.md 顶层更新

合并到 `products/{target_product}/PRODUCT.md` 的内容片段：

- [ ] Problem Statement 段（见本文件 §2）
- [ ] Success Metrics 段（见本文件 §3）
- [ ] User Intents Catalog 表新增 {N} 行（见本文件 §5）
- [ ] Intents Catalog 表新增 acceptance.intent 关联（见本文件 §6）

### Acceptance Intent 种子

- `{slug}.acceptance.intent` —— 新增 {N} 个 acceptance block（见 PDR Intent Impact）

### PDR

- `PDR-{id}-{slug}.md`（Status: Proposed → 用户审批后改 Accepted）

---

## 5. User Intents Catalog（合并到 PRODUCT.md › User Intents Catalog 表）

> 本次变更新增的「自然语言用户问句 → task」映射条目。AI Copilot 的最强索引。

| User Query | → Task | Journey Stage | Audience |
|---|---|---|---|
| "怎么第一次跑校验？" | T-0001 | onboarding | new-user |
| "5 秒内没结果是卡死了吗？" | T-0001 | onboarding | new-user |
| "拿到 FAIL 怎么看反例？" | T-0002 | onboarding | new-user |
| "怎么一次校验一堆 .intent？" | T-0010 | daily-ops | end-user |
| "为什么免费版限 5 次？" | —— *(策略问题，去 PDR-{id})* | —— | 商业咨询 |

---

## 6. Intent Mapping（核心声明 → intent 层）

> 本表从 `prd-draft.md` 继承。它驱动 acceptance.intent 种子的生成，**也驱动 PDR
> 的 Intent Impact 章节**——三者必须一致。

| # | 核心声明（PRD 原文摘录）| 目标 intent 层 | 关联 Task | acceptance block 名 |
|---|---|---|---|---|
| 1 | 「用户提交 .intent 后 5 秒内必须收到 PASS/FAIL」 | `acceptance.intent` | T-0001 | `T-0001-response-under-5s` |
| 2 | 「拿到 FAIL 时必须包含可读的最短反例」 | `acceptance.intent` | T-0002 | `T-0002-counterexample-minimal` |
| 3 | 「同一个验证任务不能被并发跑两次」 | `invariants.intent` | —— (跨 task) | `no-concurrent-verification` |
| 4 | 「verifier 对 cli 暴露 `Verify(spec) -> Result` 接口」 | `contracts.intent` | T-0001 / T-0010 | `verifier-api-v1` *(待 ADR-XXXX)* |
| 5 | 「免费用户每月 5 次验证额度」 | —— 自然语言（task frontmatter `limits` 字段）| T-0001 等多个 | —— |

> 标 `contracts.intent` 的条目本 PRD **不**实际产出 intent 内容——它们等待
> arch-debate / rfc-writer 落地 ADR 后再回填。

---

## 7. Out of Tasks（本次变更显式不做什么）

> 替代旧 PRD 的「Out of Scope」段。用 task 颗粒度描述，便于评审者识别盲区。

- ❌ 不做「远程 verifier 集群部署」（跨 product，留给 J-XXXX 旅程）
- ❌ 不做「.intent 文件的可视化编辑」（属于 ui product 的另一个 PDR）
- ❌ 不做「验证结果的历史查询」（lifecycle 类需求，下一个 PDR）

`Decision-Ref: PDR-{id}`

---

## 8. Risk Assessment

| Risk | Probability | Impact | Mitigation | Affected Tasks | Fact Cite |
|---|---|---|---|---|---|
| {风险 1} | High/Med/Low | High/Med/Low | {缓解} | T-0001, T-0010 | `fact-ext § Risk Hotspots row K` |
| {风险 2} | High/Med/Low | High/Med/Low | {缓解} | T-0002 | `[未经事实基验证]` |

`Decision-Ref: PDR-{id}`

---

## 9. Dependencies & Blockers

**Dependencies**:
- {依赖 1}: {描述和 owner}

**Known Blockers**:
- {阻塞 1}: {描述和解决计划}

**External-Writer Dependencies**（IDD 专属）:
- [ ] `arch-debate` / `rfc-writer` 落地 ADR-XXXX，定义 `Verifier API v1` 契约
       → 阻塞 task T-0001 / T-0010 进入 Accepted
- [ ] 项目自带 intent spec writer 把 `{slug}.acceptance.intent` 种子收紧后合并
       到 `products/{target_product}/intents/acceptance.intent`

---

## 10. Telemetry & Validation（AI 反馈闭环钩子）

> 本节是文章 strategy 5「AI 反馈闭环」在 PRD 层的预留契约位。具体机制由
> living-doc-author / 平台层实现，prd-writer 只声明期望。

### 上线后要监控的信号

- **Task 引用热度**：上线 30 天后，T-0001 / T-0002 / T-0010 的 AI 召回次数应 ≥ 10
  —— 零引用进归档评审
- **AI 错答率**：T-0001 query 锚点对应的 AI 回答，置信度 < 0.7 的占比应 < 5%
- **用户「转人工」率**：T-0020（troubleshooting 类）对应的会话中，用户主动转人
  工的占比应 < 15%

### Validation 节奏

- T+7 天：跑一次 task chunk 召回测试，确认 5 个旅程目录的 chunking 边界正确
- T+30 天：进入 PDR Validation Plan 的第一次 review
- T+90 天：决定零引用 task 的归档

---

## 11. Charter Compliance Self-Check

> 起草完成后由本 skill 自动跑；prompts/scoring 评分会读取本段。

- [ ] 文件清单（§4）与 PDR Consequences 完全一致
- [ ] 每个新增 task 文件单独存在且符合 `templates/task.md` 结构
- [ ] User Intents Catalog 包含每个新增 task 的至少 3 个 query 锚点
- [ ] Intent Mapping 与 acceptance.intent 种子 block 一一对应
- [ ] 无历史/未来叙事短语（grep 通过）
- [ ] 所有「数字 / LoC / 模块名 / 风险条目」cite fact-ext（或标 `[未经事实基验证]`）
- [ ] `Decision-Ref: PDR-{id}` 在 §2 / §3 / §5 / §7 / §8 各出现一次

---

## 12. 落地步骤（用户审批 PDR 后执行）

1. PDR Accepted（Status 从 Proposed 改 Accepted，写入 `products/{target_product}/decisions/pdr/`）
2. 把 §2 / §3 / §5 / §6 内容合并到 `products/{target_product}/PRODUCT.md` 对应段落
3. 把每个新增 task 文件放到对应 `products/{target_product}/tasks/{journey_stage}/` 目录
4. 把 `{slug}.acceptance.intent` 种子交给项目自带 intent spec writer 收紧后合并
5. 跑 `popsicle skill start living-doc-author --target tasks-index` 刷新 `tasks/README.md`
6. 跑 `intent-consistency-check`（上线后）—— Z3 闸放行
7. PR 评审：CI 会强制检查每个修改文件都有 `Decision-Ref: PDR-{id}`

---

*PRD overview 由 prd-writer skill 通过质量评分迭代产出。落地前请确认 PDR 已 Accepted。*
