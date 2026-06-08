# PRD 草稿 — {topic}

> **Status**: Draft (from product-debate skill — task-centric form)
> **Target Product**: `{target_product}`
> **Source Debate**: `{slug}.product-debate.md`
> **Fact Basis**: `{slug}.fact-extraction-report.md` ✅ / `[未引用]`
>
> ⚠️ 本草稿尚未通过 `prd-writer` 的质量评分；草稿中的 task 是**候选**，task_id
> 由 prd-writer 在分配阶段确定（看 product 内现有最大号 +1）。**不要**直接落地。

---

## 1. 本次变更的核心意图

> 一句话：用户原本能/不能 X，本次变更后用户能 Y。现在时。

{例：用户能在本地用 `intent check` 命令对单个 .intent 文件做 5 秒内的验证。}

---

## 2. 背景与目标

### 用户痛点

{1-2 句陈述，对应 debate-record §Phase 1}

### 目标用户

- **主要用户**: {画像}
- **次要用户**: {画像}

### 产品目标

- {目标 1 + 可衡量指标}
- {目标 2 + 可衡量指标}

### 约束条件

- **必须满足**:
  - {约束 1}
- **最好满足**:
  - {约束 2}
- **可以取舍**:
  - {约束 3}

---

## 3. 推荐方案

**选定方案**: {方案名}（对应 debate-record §Phase 4）

**修正与折中**:
- {从其它方案借鉴的部分}
- {基于评审反馈的修正}

### 决策理由

{从 Phase 4 PM 综合推荐中提取，包含关键权衡}

### 被否决的备选

- **{方案 X}**: {否决理由}
- **{方案 Y}**: {否决理由}

---

## 4. Tasks（核心产出，任务图范式）

> Phase 4 收敛时由 PM 识别的所有用户 task。**每个 task** = 一个独立的用户意图，
> 能用一句完整用户原话讲出来。
>
> 5 个旅程阶段固定：`onboarding` / `daily-ops` / `troubleshooting` / `admin` / `lifecycle`
> 详细判定准则见 prd-writer 的 `references/task-organization.md` § 1。

### 4.1 候选 Tasks 概览

| 候选 ID | 用户原话标题 | Journey Stage | Audience | 备注 |
|---|---|---|---|---|
| TBD-1 | {例：我第一次跑 intent check 直到拿到 PASS} | onboarding | new-user | 首切片必含 |
| TBD-2 | {例：拿到 FAIL 后看反例并调整 spec} | onboarding | new-user | 依赖 TBD-1 |
| TBD-3 | {例：一次性校验一组 .intent} | daily-ops | end-user | |
| TBD-4 | {例：验证超时怎么办} | troubleshooting | end-user | 拆自 TBD-1 的异常分支 |
| TBD-5 | {例：给团队成员开通验证额度} | admin | admin | |

> ⚠️ **task_id 是占位符**——`TBD-N` 由 prd-writer 在分配阶段替换为
> `T-XXXX`（product 内全局递增 4 位 0 填充）。

### 4.2 每个 task 的草稿明细

#### TBD-1: 我第一次跑 intent check 直到拿到 PASS

- **Journey Stage**: onboarding
- **Audience**: ["end-user", "new-user"]
- **前提**: 用户已安装 cli ≥ 0.2，本地有一个 .intent 文件
- **限制**: 免费额度 5 次 / 月
- **本 task 可解答**（用户原话问句，至少 3 个）：
  - 怎么第一次跑 `intent check`？
  - 跑完没拿到 PASS / FAIL 怎么办？
  - 5 秒内没结果是不是卡死了？
- **完成路径（happy path）**:
  1. 执行 `intent check my-spec.intent`
  2. CLI 显示 「Uploading spec…」
  3. CLI 返回 `✓ PASS` 或 `✗ FAIL`（5 秒内）
- **可观察的成功标志**: 用户看到 `✓ PASS` 且响应时间 < 5 秒
- **Related Next Tasks**: TBD-2, TBD-3

#### TBD-2: {下一个 task}

{同上结构}

#### ... 其它 task 依次填写

---

## 5. User Intents Catalog（草稿）

> 把上面所有 task 的「本 task 可解答」问句合并成一张表——这是 prd-writer 写
> PRODUCT.md 顶层时直接复制的素材。

| User Query | → Task | Journey Stage | Audience |
|---|---|---|---|
| 「怎么第一次跑校验？」 | TBD-1 | onboarding | new-user |
| 「5 秒内没结果是卡死了吗？」 | TBD-1 | onboarding | new-user |
| 「拿到 FAIL 怎么看反例？」 | TBD-2 | onboarding | new-user |
| 「怎么一次校验一堆 .intent？」 | TBD-3 | daily-ops | end-user |
| 「为什么免费版限 5 次？」 | —— *(策略问题，去 PDR)* | —— | 商业咨询 |

---

## 6. Intent Mapping（核心声明 → intent 层）*(IDD 专属，必填)*

> 本表来自 debate-record §Intent 层归类。`task` 列把声明绑到具体 task。下游
> `prd-writer` 用这张表生成 acceptance.intent 种子和 PDR 骨架。

| # | 核心声明 | 目标 intent 层 | 关联 Task | 候选 block 名 | 备注 |
|---|---|---|---|---|---|
| 1 | 「用户提交 .intent 后 5 秒内必须收到 PASS/FAIL」 | `acceptance.intent` | TBD-1 | `T-XXXX-response-under-5s` | block 名 XXXX 由 prd-writer 分配 |
| 2 | 「拿到 FAIL 时必须包含可读最短反例」 | `acceptance.intent` | TBD-2 | `T-XXXX-counterexample-minimal` | |
| 3 | 「同一个验证任务不能被并发跑两次」 | `invariants.intent` | —— (跨 task) | `no-concurrent-verification` | 由项目自带 spec writer 处理 |
| 4 | 「verifier 对 cli 暴露 `Verify(spec) -> Result` 接口」 | `contracts.intent` | TBD-1 / TBD-3 | `verifier-api-v1` | **待 ADR-XXXX** |
| 5 | 「免费用户每月 5 次验证额度」 | —— 自然语言（task frontmatter `limits` 字段）| TBD-1 等 | —— | 商业策略可调 |

---

## 7. Out of Tasks（本次显式不做什么）

- ❌ 不做 {例：远程 verifier 集群部署}（跨 product，留给 J-XXXX 旅程）
- ❌ 不做 {例：.intent 文件的可视化编辑}（属于 ui product 的另一个 PDR）
- ❌ 不做 {例：验证结果的历史查询}（lifecycle 类需求，下一个 PDR）

---

## 8. 成功指标

| 指标 | 当前基线 | 目标值 | 衡量方式 |
|------|---------|--------|---------|
| {指标 1} | {基线} | {目标} | {方式} |
| {指标 2} | {基线} | {目标} | {方式} |

---

## 9. 风险与缓解

| 风险 | 概率 | 影响 | 缓解措施 | Affected Tasks |
|------|------|------|---------|----------------|
| {风险 1} | 高/中/低 | 高/中/低 | {措施} | TBD-1, TBD-3 |
| {风险 2} | 高/中/低 | 高/中/低 | {措施} | TBD-2 |

---

## 10. 事实基引用

| Cite ID | 内容 | 来源 |
|---|---|---|
| F-1 | {LoC = 12,431} | `{slug}.fact-extraction-report.md § Bounded Contexts row K` |
| F-2 | {高 churn 模块 verifier/z3.rs} | `{slug}.fact-extraction-report.md § Risk Hotspots row K` |

未引用条目用 `[未经事实基验证]` 标记。

---

## 11. 辩论元信息

- **辩论文件**: `{slug}.product-debate.md`
- **参与角色**: {角色列表}
- **用户置信度**: {N}/5
- **投票结果**: 方案 A {N} 票 / 方案 B {N} 票 / ...
- **是否用户决策覆盖**: 否 / 是（{理由}）
- **关键分歧**: {简述未完全解决的分歧}

---

## 12. 下游 Skill 接驳

- [ ] **下一步**: 把本草稿喂给 `prd-writer` skill：
  - 替换 TBD-N 为正式 task_id（T-XXXX）
  - 按本草稿 §4 起草 N 份 task 文件（落到对应 journey_stage 目录）
  - 起草 prd-overview / PDR / acceptance.intent 种子 / tasks/README.md
  - 跑质量评分，迭代到 ≥ 90
- [ ] **若涉及架构**（Intent Mapping 中含 `contracts.intent` 行）：先跑
      `arch-debate` / `rfc-writer` 落地 ADR，再回 prd-writer 关联 ADR ID
- [ ] **完成后**: 按 PDR Consequences 把 N 份 task 文件落到
      `products/<target_product>/tasks/{journey_stage}/T-XXXX-<slug>.md`

---

## Task-Centric 形态检查清单（prd-writer 接收时会校验）

- [ ] §4 是「Tasks」段（不是「Functional Requirements」）
- [ ] 每个 task 标了 5 个旅程阶段之一
- [ ] 每个 task 有用户原话标题（不形如「xxx 功能」）
- [ ] 每个 task 有 ≥ 3 个用户原话问句
- [ ] §5 User Intents Catalog 完整覆盖所有 task
- [ ] §6 Intent Mapping 表完整且每条标了 intent 层
- [ ] §7 Out of Tasks 显式列出 ≥ 2 项
