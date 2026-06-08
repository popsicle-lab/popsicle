---
artifact: living-doc-sync-report
slug: {slug}
generated_by: living-doc-author
target: all                 # tasks-index | task-backrefs | last-verified | product-context | all
last_updated: {YYYY-MM-DD}
docs_scanned: 0
drift_signals: 0
docs_refreshed: 0
manual_followups: 0
query_anchors:
  - "我的活文档过期了吗？"
  - "哪些 task 没人引用 / 该归档了？"
  - "tasks 索引和健康度刷新了吗？"
---

# 活文档保活报告 — {slug}

> 由 `living-doc-author` skill 生成。**只对账与刷新活文档元数据**，不创作正文、不改
> 业务逻辑——后者必须走 prd-writer + PDR（charter 铁律）。

## Summary

| 指标 | 值 |
|---|---|
| target | all |
| 扫描文档数 | 0 |
| 发现 drift 信号 | 0 |
| 本次刷新文档数 | 0 |
| 待人工处置项 | 0 |
| 结论 | 已对齐 / 有 drift 待处理 |

一句话结论：……

## Scan Checklist

- [ ] target 已确认
- [ ] 所有 task / intent / PDR / PRODUCT.md 已枚举
- [ ] 四类 drift 信号已逐条核对，证据已记录
- [ ] 已区分「可自动刷新的元数据」与「需 PDR 的正文改动」

## Drift 信号

> 四类信号各列明细。每条带证据（文件 + 行/字段）。无则写「（无）」。

### 1. 过期 staleness

| 文档 | 证据 | 严重度 |
|---|---|---|
| 例：T-0040 | last_updated 90 天前，无更新 | 归档评审 |

### 2. 断链 broken-ref

| 来源 | 失效引用 | 类型 |
|---|---|---|
| 例：T-0001 related_intents | acceptance.intent#T-0001-... 不存在 | intent block |

### 3. 孤儿 orphan

| 对象 | 情况 |
|---|---|
| 例：acceptance block T-9999 | 无对应 task 文件（双射断裂）|
| 例：T-0031 | 无任何反向引用（> 90 天 → 归档评审）|

### 4. 未验证 unverified

| Task | 当前 last_verified | 报告中的状态 | 可回填？ |
|---|---|---|---|
| 例：T-0001 | ~ | verified（2026-05-13）| 是 |

## 刷新动作

> 本次实际改动的活文档，一行一处。只动元数据 / 索引 / 反向引用 / last_verified。

| 文件 | 改动 |
|---|---|
| 例：products/auth/tasks/README.md | 重建索引 + 健康度，Last-Generated→今天 |
| 例：T-0001.md frontmatter | last_verified ~ → 2026-05-13 |

## 健康度快照

> 刷新后的 tasks/README 健康度表当前值。这是文档腐烂预警的核心信号。

| 旅程阶段 | Task 数 | 平均行数 | 最久未更新 | 未引用 task |
|---|---|---|---|---|
| onboarding | 0 | 0 | — | 无 |
| daily-ops | 0 | 0 | — | 无 |
| troubleshooting | 0 | 0 | — | 无 |
| admin | 0 | 0 | — | 无 |
| lifecycle | 0 | 0 | — | 无 |

## 待人工处置

> 超出 living-doc-author 自动刷新范围的 drift。每项指派跟进。

- [ ] 需 PDR 的正文 drift：……（走 prd-writer + 新 PDR）
- [ ] > 90 天无引用归档评审候选：……（PM 决定是否废止）
- [ ] 验证失败需修 spec 的 task：……（见 intent-consistency-report）

---

## 检查清单（提交前勾完）

- [ ] 四类 drift 信号都已扫描并列出（或写「（无）」）
- [ ] 刷新动作每条都对应真实文件改动
- [ ] last_verified 只回填了 verified 的 task
- [ ] 健康度快照数字与刷新后的 README 一致
- [ ] 所有越界 drift 已转「待人工处置」，未擅自改正文
- [ ] frontmatter 计数与正文一致
