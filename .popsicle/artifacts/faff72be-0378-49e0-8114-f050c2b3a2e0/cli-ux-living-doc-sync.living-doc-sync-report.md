---
id: f67e3698-c75f-4497-a3ff-4334749bfdd7
doc_type: living-doc-sync-report
title: cli-ux living doc sync
status: final
skill_name: living-doc-author
pipeline_run_id: faff72be-0378-49e0-8114-f050c2b3a2e0
spec_id: 308c231f-0e91-4922-87f0-22b63be857b6
version: 1
parent_doc_id: null
tags: []
metadata: null
created_at: 2026-06-10T08:44:00Z
updated_at: 2026-06-10T08:49:14.329038Z
---

# 活文档保活报告 — cli-ux-living-doc-sync

> 由 `living-doc-author` skill 生成。只对账与刷新活文档元数据，不创作正文、不改 intent 逻辑。

## Summary

| 指标 | 值 |
|---|---|
| target | implementation-status, architecture-manifest, product-header |
| 扫描文档数 | 12 |
| 发现 drift 信号 | 5 |
| 本次刷新文档数 | 5 |
| 待人工处置项 | 2 |
| 结论 | cli-ux 已同步到 cutover-done |

一句话结论：cli-ux 的 cutover 状态、traceability、产品头、架构 manifest 与 task 实施状态已按 ADR-008 对齐；剩余真实 workspace mutation 与 Tauri bridge 另开后续议题。

## Scan Checklist

- [x] target 已确认
- [x] 所有 task / intent / PDR / PRODUCT.md 已枚举
- [x] 四类 drift 信号已逐条核对，证据已记录
- [x] 已区分「可自动刷新的元数据」与「需 PDR 的正文改动」

## Drift 信号

### 1. 过期 staleness

| 文档 | 证据 | 严重度 |
|---|---|---|
| `products/cli-ux/PRODUCT.md` | Status 仍为 spec in-progress，Last-Decision-Ref 仍指 PDR-001 | 高 |
| `products/cli-ux/ARCHITECTURE.md` | File Manifest 仍为 planned / ADR-007 Proposed | 高 |
| `products/cli-ux/tasks/README.md` | 已实施列仍为 0 / spec draft | 中 |

### 2. 断链 broken-ref

| 来源 | 失效引用 | 类型 |
|---|---|---|
| （无） | — | — |

### 3. 孤儿 orphan

| 对象 | 情况 |
|---|---|
| （无） | 7 个 cli-ux task 均由 PDR-001 与 intent/golden 覆盖 |

### 4. 未验证 unverified

| Task | 当前 last_verified | 报告中的状态 | 可回填？ |
|---|---|---|---|
| T-CU-0001..T-CU-0007 | `~` | 7 VC verified + 6/6 golden pass | 本轮只刷新索引状态，task frontmatter 回填留给后续 last-verified target |

## 刷新动作

| 文件 | 改动 |
|---|---|
| `migration/progress.md` | cli-ux 状态 in-shadow → cutover-done；Last-Decision-Ref → ADR-008 |
| `migration/traceability.md` | slice-3 traceability 行 ADR-008 填实，状态 → cutover-done |
| `products/cli-ux/PRODUCT.md` | Status / Last-Decision-Ref / roadmap 从 spec 状态刷新到 ADR-008 cutover-done |
| `products/cli-ux/ARCHITECTURE.md` | File Manifest 更新为 lib + binary entrypoint cutover-done |
| `products/cli-ux/tasks/README.md` | 7/7 task 已实施列与健康度刷新为 implemented |

## 健康度快照

| 旅程阶段 | Task 数 | 平均行数 | 最久未更新 | 未引用 task |
|---|---:|---:|---|---|
| onboarding | 1 | 48 | T-CU-0001（2026-06-09）| 无 |
| daily-ops | 3 | 48 | T-CU-0002/T-CU-0003/T-CU-0004（2026-06-09）| 无 |
| troubleshooting | 1 | 48 | T-CU-0005（2026-06-09）| 无 |
| admin | 1 | 48 | T-CU-0006（2026-06-09）| 无 |
| lifecycle | 1 | 48 | T-CU-0007（2026-06-09）| 无 |

## 待人工处置

| 项 | 原因 | 跟进 |
|---|---|---|
| Storage-backed real workspace mutation | ADR-008 只切 semantic shell 与 binary entrypoint，不宣称完整持久化写路径 | 后续 storage/packaging ADR |
| Tauri UI bridge | 不进 cli-ux MVP | 需要时另开 slice / PDR |

---

## 检查清单（提交前勾完）

- [x] 四类 drift 信号都已扫描并列出（或写「（无）」）
- [x] 刷新动作每条都对应真实文件改动
- [x] last_verified 只回填了 verified 的 task
- [x] 健康度快照数字与刷新后的 README 一致
- [x] 所有越界 drift 已转「待人工处置」，未擅自改正文
- [x] frontmatter 计数与正文一致
