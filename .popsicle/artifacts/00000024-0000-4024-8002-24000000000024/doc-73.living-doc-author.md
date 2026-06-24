---
doc_type: living-doc-author
id: doc-73
pipeline_run_id: 00000024-0000-4024-8002-24000000000024
status: active
title: PROJ-36 living docs sync
version: 1
artifact: living-doc-sync-report
slug: proj-36-living-doc-sync
generated_by: living-doc-author
target: all
last_updated: 2026-06-11
docs_scanned: 4
drift_signals: 0
docs_refreshed: 3
manual_followups: 0
query_anchors:
  - "我的活文档过期了吗？"
  - "哪些 task 没人引用 / 该归档了？"
---

# 活文档保活报告 — PROJ-36 living docs sync

> PROJ-36 cutover ADR-018 后对齐 migration + PRODUCT 活文档。

## Summary

| 指标 | 值 |
|---|---|
| target | all |
| 扫描文档数 | 4 |
| 发现 drift 信号 | 0 |
| 本次刷新文档数 | 3 |
| 待人工处置项 | 0 |
| 结论 | 已对齐 |

一句话结论：PRODUCT.md、migration/progress.md、migration/traceability.md 已反映 ADR-018 UI modern layout cutover。

## Scan Checklist

- [x] target 已确认（PROJ-36 post-cutover）
- [x] PRODUCT / migration 活文档已枚举
- [x] 四类 drift 信号已核对（均无）
- [x] 仅元数据/索引刷新，无业务正文越界改动

## Drift 信号

### 1. 过期 staleness

（无）

### 2. 断链 broken-ref

（无）

### 3. 孤儿 orphan

（无）

### 4. 未验证 unverified

（无）

## 刷新动作

| 文件 | 改动 |
|---|---|
| `products/cli-ux/PRODUCT.md` | Status/Last-Decision-Ref + ADR-018 committed roadmap 行 |
| `migration/progress.md` | Last-Decision-Ref ADR-018（PROJ-36）|
| `migration/traceability.md` | legacy ui layout → ui/ modern shell，ADR-018，cutover-done |
| `products/cli-ux/decisions/adr/ADR-018-ui-modern-layout.md` | Accepted cutover ADR（prior promote）|

## 健康度快照

| 旅程阶段 | Task 数 | 备注 |
|---|---|---|
| daily-ops | 4 | UI layout 由 ADR-018 覆盖，tasks 未改 |
| onboarding | 4 | 与 ADR-016 一致 |

## 待人工处置

（无）

## 检查清单（提交前勾完）

- [x] 四类 drift 信号都已扫描并列出
- [x] 刷新动作每条都对应真实文件改动
- [x] last_verified 未越界回填
- [x] 健康度快照与 README 一致
- [x] 越界 drift 已转待人工处置（无）
- [x] frontmatter 计数与正文一致
