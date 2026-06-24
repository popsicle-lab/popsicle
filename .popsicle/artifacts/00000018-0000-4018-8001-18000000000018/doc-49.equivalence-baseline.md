---
doc_type: equivalence-baseline
id: doc-49
pipeline_run_id: 00000018-0000-4018-8001-18000000000018
status: active
title: PROJ-24 usability equivalence report
version: 1
---

# PROJ-24 usability equivalence report

> **Issue**: PROJ-24 · self-host usability completion
> **Stage**: equivalence (slice-delivery)
> **Baseline dir**: `docs/baseline/2026-06-11/cli-ux-usability/`
> **Date**: 2026-06-11

## Equivalence Reference

New-capability slice:reference 不是 legacy parity,而是「ADR-011 之后的命令面
契约不回归 + 新能力契约可重复断言」。

## Inventory

| # | Script | Asserts | Result |
|---|---|---|---|
| – | `../cli-ux-self-host/run-all.sh` | ADR-010 goldens(8)| 8/8 pass |
| – | `../cli-ux-command-alignment/run-all.sh` | ADR-011 goldens(5)| 5/5 pass |
| 1 | `golden-001-doc-check.sh` | stub 失败 → 占位符失败 → 实文+checkbox 通过 | pass |
| 2 | `golden-002-issue-close.sh` | active run 阻断 → run 完成后 close 置 done 并持久化 | pass |
| 3 | `golden-003-default-pipelines.sh` | 4 个 issue 类型默认管线全部 bundled;新命令解析 | pass |
| 4 | `golden-004-smoke-isolation.sh` | smoke 前后真实工作区 issue 数不变(保持 7)| pass |
| 5 | `golden-005-e2e-dogfood.sh` | 隔离工作区端到端:init→bug issue(默认 bugfix)→start→doc check 双路径→双 stage→completed→close | pass |

Totals: **18/18 pass**(13 回归 + 5 新契约)。

## Test Suite

- `cargo test -p cli-ux`:golden 11 · intent_properties 7 · smoke 1 · tsv 7 · unit 9,全绿
- `cargo test -p skill-runtime -p storage -p artifact-system`:全绿
- `tool run intent-validate path=products`:exit 0

## Observations

- O-201:doc check 占位符扫描会命中"谈论占位符语法"的文档字面量(P5,首次
  dogfood 即触发)。反引号/代码块豁免记入 Phase 2 优化。
- O-202:残留清理误删事故(P3)证明 state.tsv 外科手术需要专用 admin 命令
  (如 `admin prune --smoke`)而不是手写脚本;记入 follow-ups。

## Verdict

Equivalence gate **green**。Ready for cutover ADR-012。
