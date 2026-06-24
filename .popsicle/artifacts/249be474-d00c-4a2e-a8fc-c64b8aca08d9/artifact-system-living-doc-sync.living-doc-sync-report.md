---
id: 435a86ba-5b8b-4b44-a12f-318899c611b8
doc_type: living-doc-sync-report
title: artifact-system living doc sync
status: final
skill_name: living-doc-author
pipeline_run_id: 249be474-d00c-4a2e-a8fc-c64b8aca08d9
spec_id: cf6e7062-b75f-4925-b633-82a04e775a76
version: 1
parent_doc_id: null
tags: []
metadata: null
created_at: 2026-06-09T10:02:27.004909Z
updated_at: 2026-06-09T10:02:27.004909Z
---

---
artifact: living-doc-sync-report
slug: artifact-system-living-doc-sync
generated_by: living-doc-author
target: implementation-status,architecture-manifest,product-header
last_updated: 2026-06-09
docs_scanned: 8
drift_signals: 3
docs_refreshed: 5
manual_followups: 2
query_anchors:
  - "我的活文档过期了吗？"
  - "哪些 task 没人引用 / 该归档了？"
  - "tasks 索引和健康度刷新了吗？"
---

# 活文档保活报告 — artifact-system-living-doc-sync

> 由 `living-doc-author` skill 生成。**只对账与刷新活文档元数据**，不创作正文、不改
> 业务逻辑——后者必须走 prd-writer + PDR（charter 铁律）。

## Summary

| 指标 | 值 |
|---|---|
| target | implementation-status,architecture-manifest,product-header |
| 扫描文档数 | 8 |
| 发现 drift 信号 | 3 |
| 本次刷新文档数 | 5 |
| 待人工处置项 | 2 |
| 结论 | 已对齐（CLI byte parity / SQLite wiring 待 cli-ux 跟进）|

一句话结论：tasks/README 已实施列、ARCHITECTURE File Manifest、PRODUCT 双行头、migration 看板与 traceability 已与 ADR-006 / equivalence-report 对齐。

## Scan Checklist

- [x] target 已确认
- [x] 所有 task / intent / PDR / PRODUCT.md 已枚举
- [x] 四类 drift 信号已逐条核对，证据已记录
- [x] 已区分「可自动刷新的元数据」与「需 PDR 的正文改动」

## Drift 信号

> 四类信号各列明细。每条带证据（文件 + 行/字段）。无则写「（无）」。

### 1. 过期 staleness

| 文档 | 证据 | 严重度 |
|---|---|---|
| `PRODUCT.md` | Status / Last-Decision-Ref 仍为 bootstrap/TBD | 自动刷新 |
| `ARCHITECTURE.md` | Status / File Manifest 仍为 bootstrap/TBD | 自动刷新 |
| `tasks/README.md` | 已实施列仍为 0 | 自动刷新 |

### 2. 断链 broken-ref

| 来源 | 失效引用 | 类型 |
|---|---|---|
| （无） | — | — |

### 3. 孤儿 orphan

| 对象 | 情况 |
|---|---|
| （无） | 6 task ↔ 5 acceptance/invariant anchors 对齐 |

### 4. 未验证 unverified

| Task | 当前 last_verified | 报告中的状态 | 可回填？ |
|---|---|---|---|
| T-AS-0001..0006 | 2026-06-09 | Z3 verified + implementation/golden covered | 已回填 |

## 刷新动作

> 本次实际改动的活文档，一行一处。只动元数据 / 索引 / 反向引用 / last_verified。

| 文件 | 改动 |
|---|---|
| `migration/progress.md` | artifact-system → cutover-done，阶段 PROJ-5 slice-delivery ✓ |
| `migration/traceability.md` | 追加 slice-2 legacy → new 映射行 |
| `products/artifact-system/PRODUCT.md` | Status + Last-Decision-Ref → ADR-006；补用户入口 / roadmap |
| `products/artifact-system/ARCHITECTURE.md` | File Manifest + module graph + boundary 更新 |
| `products/artifact-system/tasks/README.md` | 已实施列 0 → 6/6 |

## 健康度快照

> 刷新后的 tasks/README 健康度表当前值。这是文档腐烂预警的核心信号。

| 旅程阶段 | Task 数 | 平均行数 | 最久未更新 | 未引用 task |
|---|---|---|---|---|
| onboarding | 1 | <150 | 2026-06-09 | 无 |
| daily-ops | 3 | <150 | 2026-06-09 | 无 |
| troubleshooting | 1 | <150 | 2026-06-09 | 无 |
| admin | 0 | — | — | 无 |
| lifecycle | 1 | <150 | 2026-06-09 | 无 |

## 待人工处置

> 超出 living-doc-author 自动刷新范围的 drift。每项指派跟进。

| 项 | 原因 |
|---|---|
| CLI byte parity | `doc` / `prompt` / `extract` 命令归 `cli-ux` slice |
| SQLite `IndexDb` full cutover | 当前仅 `DocumentRow` 下沉，wiring 延后 |

---

## 检查清单（提交前勾完）

- [x] 四类 drift 信号都已扫描并列出（或写「（无）」）
- [x] 刷新动作每条都对应真实文件改动
- [x] last_verified 只回填了 verified 的 task
- [x] 健康度快照数字与刷新后的 README 一致
- [x] 所有越界 drift 已转「待人工处置」，未擅自改正文
- [x] frontmatter 计数与正文一致
