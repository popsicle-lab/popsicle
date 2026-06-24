---
doc_type: equivalence-baseline
id: doc-53
pipeline_run_id: 00000019-0000-4019-8001-19000000000019
status: active
title: PROJ-25 SQLite Phase 2 equivalence report
version: 1
---

# PROJ-25 SQLite Phase 2 equivalence report

> **Issue**: PROJ-25 · PROJ-11 SQLite Phase 2
> **Stage**: equivalence (slice-delivery)
> **Baseline dir**: `docs/baseline/2026-06-11/cli-ux-sqlite-phase2/`
> **Date**: 2026-06-11

## Equivalence Reference

存储后端替换:reference 是「Phase 1 TSV 行为 = Phase 2 SQLite 行为」——
WorkspaceStore 全部操作语义不变、smoke 契约不变、迁移无损。

## Inventory

| # | Script | Asserts | Result |
|---|---|---|---|
| – | `../cli-ux-usability/run-all.sh`(级联 self-host + alignment)| 历史契约 18 项不回归 | 18/18 pass |
| 1 | `golden-001-sqlite-default.sh` | 新建工作区默认 SQLite,产生 state.db 不产生 state.tsv | pass |
| 2 | `golden-002-tsv-compat.sh` | 遗留 TSV 工作区可读可写,不被擅自迁移 | pass |
| 3 | `golden-003-migrate.sh` | 迁移保行保计数器、留底 .migrated、幂等 | pass |
| 4 | `golden-004-e2e-on-sqlite.sh` | 完整 IDD 闭环(issue→run→doc check→stages→close)跑在 SQLite 上 | pass |
| 5 | `golden-005-doctor-backend.sh` | 真实工作区迁移后 doctor 报 sqlite 后端 | pass |

Totals: **23/23 pass**(18 回归 + 5 新契约)。

## Baseline Amendments (per ADR-013)

- `cli-ux-self-host/golden-008`:storage_backend 断言由固定 tsv 字符串改为
  动态(tsv|sqlite),注明 Amended-by ADR-013。
- `cli-ux-usability/golden-004`:工作区不变性计数由 grep state.tsv 改为
  `issue list` 计数,后端无关。

## Test Suite

- `cargo test` 全工作区 79/79(本切片新增 3 个后端专项测试)
- 真实工作区迁移后:8 issues 完整、PROJ-25 run 状态保持 in_progress
- `tool run intent-validate path=products` exit 0

## Observations

- O-301:doctor 只校验二进制路径匹配,不检测二进制是否过期(P3 沙箱事故的
  暴露面),staleness 检测记入 follow-up。

## Verdict

Equivalence gate **green**。Ready for cutover ADR-013。
