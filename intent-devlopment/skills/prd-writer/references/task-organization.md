# Task 组织规则

本文件定义 intent-coder 项目内 task 的**目录归类**、**命名规则**和**反模式**。
所有规则在 prd-writer 的 quality-rubric 中都对应扣分项；本文件是它们的「源代码」。

---

## 1. 目录归类：5 个固定的用户旅程阶段

每个 product 的 `tasks/` 目录下**固定 5 个**子目录，不允许增减：

```
products/<product>/tasks/
├── README.md                  # 自动生成的索引（由 prd-writer / living-doc-author 维护）
├── onboarding/
├── daily-ops/
├── troubleshooting/
├── admin/
└── lifecycle/
```

### 5 个阶段的精确定义

| 阶段 | 触发条件 | 完成条件 | 典型 task |
|---|---|---|---|
| `onboarding/` | 用户**首次**接触某项能力 | 用户**首次成功**使用该能力 | 第一次跑 intent check 直到拿到 PASS；首次配置 SSO 登录 |
| `daily-ops/` | 用户已掌握基本能力 | 满足一个具体业务需求 | 批量校验一组 .intent；导出本月验证报告 |
| `troubleshooting/` | 操作出现异常 / 失败 | 异常解除或决定升级处理 | 验证超时后看反例并调整 spec；权限报错排查 |
| `admin/` | 涉及组织 / 配额 / 权限 / 审计 | 管理操作生效 | 给团队成员开通验证额度；查询 30 天内审计日志 |
| `lifecycle/` | 终止 / 迁出 / 续费 / 归档 | 用户与产品的某段关系结束 | 注销账号前导出所有数据；续费升级到团队版 |

### 判定准则（task 归哪个阶段）

按这个**优先级判断**——一旦命中前序条件就归该阶段，不再往后看：

1. 用户是否在「首次成功」的旅程上？→ `onboarding/`
2. 用户是否在排查异常 / 失败 / 报错？→ `troubleshooting/`
3. 操作是否涉及组织 / 配额 / 权限 / 审计？→ `admin/`
4. 操作是否涉及账号或数据的**终止**关系？→ `lifecycle/`
5. 其它都进 → `daily-ops/`

> 注意「首次成功」的 onboarding 边界：用户**用过一次但没成功**仍属于 onboarding；
> 直到首次成功，再次使用同一能力才算 daily-ops。

---

## 2. 命名双轨制：稳定 ID + 可读 slug + 用户原话标题

| 元素 | 规则 | 示例 |
|---|---|---|
| **Task ID** | `T-{nnnn}` 在 product 内 4 位 0 填充递增，**永不变** | `T-0001`, `T-0042` |
| **Slug** | `kebab-case`，3-6 词，动词或名词短语，**可改** | `first-time-verify` |
| **文件名** | `{ID}-{slug}.md` | `T-0001-first-time-verify.md` |
| **目录** | 按 5 个旅程阶段之一 | `tasks/onboarding/` |
| **h1 标题** | **完整人话句子**，第一人称（「我……」）或祈使句 | `# 我第一次跑 intent check 直到拿到 PASS` |
| **frontmatter title** | 与 h1 完全一致 | `title: "我第一次跑 intent check 直到拿到 PASS"` |

### 为什么三层

| 层 | 稳定性 | 用途 |
|---|---|---|
| `T-0001` | 永不变 | 所有 Decision-Ref / intent block / PDR Consequences / 跨 task 引用都用它 |
| `first-time-verify` | 可改 | 文件名和 URL 的人类可读部分。用户研究发现更好措辞时可重命名（git mv） |
| 「我第一次跑 intent check 直到拿到 PASS」 | 随产品演进 | 用户浏览目录时一眼能懂；AI 拿来当最强 query 锚点 |

### Slug 改名时怎么办

例：用户研究发现「跑 intent check」对新用户太技术，改成「让 AI 校验我的 spec」。

```bash
git mv products/verifier/tasks/onboarding/T-0001-first-time-verify.md \
       products/verifier/tasks/onboarding/T-0001-first-spec-check.md
```

然后：
- 改 frontmatter 的 `slug` 字段
- 改 h1 标题
- **不改** `task_id`
- **不需要**改其它引用 `T-0001` 的文件

---

## 3. 反模式（任一项命中 = quality-rubric 扣分）

### 目录反模式

| 反模式 | 为什么不行 |
|---|---|
| `tasks/user-management/`、`tasks/billing/`、`tasks/dashboard/` | 这是按 **feature 模块**——退化回功能树，与任务图哲学矛盾 |
| `tasks/p0/`、`tasks/p1/`、`tasks/mvp/` | 优先级会变（P1 升 P0），目录要跟着搬家 |
| `tasks/v3.2/`、`tasks/2026-q2/`、`tasks/legacy/` | 版本号 / 季度 / 「legacy」是历史叙事，违反 charter 第 1 条铁律 |
| 自创第 6 个旅程阶段目录（`tasks/integration/`、`tasks/migration/`）| 5 个阶段是**固定**的；新需求要么归入现有阶段，要么走 CADR |

### 命名反模式

| 反模式 | 为什么不行 | 改成 |
|---|---|---|
| `T-onboarding-0001` | 分类嵌入 ID，task 改归类时 ID 就乱 | `T-0001`（分类在目录） |
| `T-1.md` | ID 没 0 填充，按字典排序会乱 | `T-0001-{slug}.md` |
| `reset-password.md`（无 ID）| 跨 task 引用没有稳定键 | `T-0023-reset-password.md` |
| `T-0001.md`（无 slug）| 目录扫一眼看不出内容 | `T-0001-first-time-verify.md` |
| h1 = `# 重置密码功能` | 写功能名而不是用户原话 | `# 我忘记密码后 5 分钟内重新登录` |
| h1 = `# Reset Password Feature` | 中文项目里用英文标题（query 锚点失效）| 用用户实际使用语言 |
| 文件名含中文 | git / URL / 编码兼容性问题 | ID + 英文 slug（标题可中文）|

### 内容反模式

| 反模式 | 为什么不行 | 改成 |
|---|---|---|
| Task 文件 > 250 行 | 它不是 task，是 epic | 拆成多个 task，用 Related Next Tasks 串联 |
| 完成路径里大量 if-else 分支 | 主路径不清晰；AI 召回后还得猜分支 | 主路径走 happy path，异常分支拆到 `troubleshooting/` |
| 用「如上所述」「参考前文」 | RAG chunk 切开后跨引用失效 | 完整描述；引用其它 task 用 `[T-XXXX](...)` |
| 「上线后用户将能 X」 | 未来时叙事，违反 charter 第 1 条铁律 | 现在时（「用户能 X」）|
| 「我们之前用 polling，现在改成 SSE」 | 历史叙事 | 历史进 PDR；task 只写当前状态 |
| 缺 `Decision-Ref` | 违反 charter 第 3 条铁律 | 文件末尾 `Decision-Ref: PDR-XXXX` |

---

## 4. 跨 product 的任务：`docs/user-journeys/` 全局层

如果一个用户旅程**跨多个 product**（例：新员工入职涉及 admin-console + auth + billing），**不**在任一 product 内重复定义，而是放在全局层：

```
docs/user-journeys/
├── README.md
├── J-0001-new-employee-onboarding.md
└── J-0002-team-collab-setup.md
```

### Journey 文件结构

Journey 文件**不重复定义**单 product 内的 task，只编排顺序：

```markdown
---
journey_id: J-0001
slug: new-employee-onboarding
title: "新员工入职当天开通所有权限"
involved_products: ["admin-console", "auth", "billing"]
audience: ["HR", "IT admin"]
decision_ref: PDR-0099  # 跨 product 决策必须有 CADR 或上级 PDR
last_updated: 2026-05-13
---

# {title}

## Stages

| # | Product | Task | Why this order |
|---|---|---|---|
| 1 | admin-console | [T-0030](../../products/admin-console/tasks/admin/T-0030-create-employee-account.md) | 必须先有账号 |
| 2 | auth          | [T-0015](../../products/auth/tasks/admin/T-0015-assign-initial-roles.md)        | 账号建好后配权限 |
| 3 | billing       | [T-0008](../../products/billing/tasks/admin/T-0008-allocate-seat-license.md)    | 当天结束前占座位 |

## Cross-Cutting Constraints

- 整个 journey 必须在 30 分钟内完成（acceptance.intent#J-0001-completes-in-30min）
- 任一 stage 失败必须能回滚前序 stage（intents/invariants.intent#journey-atomicity）
```

### Journey ID 命名

| 元素 | 规则 |
|---|---|
| Journey ID | `J-{nnnn}` 全局递增（不分 product）|
| Slug | `kebab-case`，描述旅程的业务名 |
| 文件名 | `J-{nnnn}-{slug}.md` |
| 目录 | `docs/user-journeys/` 单层，**不**按阶段分类（journey 本身已经是阶段无关的）|

### 何时该用 Journey 而不是 Task

- 涉及 ≥ 2 个 product：**必须**用 Journey
- 单 product 内的多 task 串联：用 task 的 `related_next_tasks` 字段，**不**升格为 Journey
- 只有一个 task：直接是 task，不要为了「显得正式」包一层 Journey

---

## 5. 一致性自检（每次起草后跑）

prd-writer 在 `review` 状态前必须跑下面这张表：

- [ ] 所有 task 在 5 个标准目录之一（无第 6 个目录）
- [ ] 所有 task 文件名匹配 `T-{4 位数字}-{kebab-case-slug}.md`
- [ ] 所有 task 的 frontmatter 8 个必填字段齐全
- [ ] task 间引用全部用 `task_id`（无裸 slug 引用）
- [ ] 所有 task h1 与 frontmatter `title` 一致
- [ ] 没有 h1 形如 `# xxx 功能` / `# 实现 xxx` 的功能描述式标题
- [ ] 所有 task 文件 ≤ 250 行
- [ ] 所有 task 含 `Decision-Ref: PDR-XXXX` 末行
- [ ] `PRODUCT.md` 的 User Intents Catalog 包含本次新增的所有 task
- [ ] 跨 product 旅程已升格为 `docs/user-journeys/J-XXXX-*.md`，未冗余在 task 中

任一项失败 → 退回 drafting。
