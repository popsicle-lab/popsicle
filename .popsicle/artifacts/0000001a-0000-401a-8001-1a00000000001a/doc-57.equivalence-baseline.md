---
doc_type: equivalence-baseline
id: doc-57
pipeline_run_id: 0000001a-0000-401a-8001-1a00000000001a
status: active
title: PROJ-26 devops equivalence report
version: 1
---

# PROJ-26 devops equivalence report

> **Issue**: PROJ-26 · migrate legacy devops tooling
> **Stage**: equivalence (slice-delivery)
> **Baseline dir**: `docs/baseline/2026-06-11/cli-ux-devops/`
> **Date**: 2026-06-11

## Equivalence Reference

DevOps 迁移:reference 是「legacy 工具脚本的开发者工作流语义在新仓库等价
成立」——check 三件套、安装/卸载闭环、hook 安装、CI/Release 门禁——同时
裁剪 UI/completions 等不适用面并逐条记录。

## Inventory

| # | Script | Asserts | Result |
|---|---|---|---|
| – | `../cli-ux-sqlite-phase2/run-all.sh`(级联全部历史)| 23 项历史契约不回归 | 23/23 pass |
| 1 | `golden-001-make-targets.sh` | `make fmt` + `make clippy` 干净(-Dwarnings)| pass |
| 2 | `golden-002-install-script.sh` | 语法 + 选项面正确,legacy UI/completions 旗标已移除 | pass |
| 3 | `golden-003-hooks.sh` | pre-commit 语法 + `make install-hooks` 落位可执行 | pass |
| 4 | `golden-004-workflows.sh` | ci.yml 三件套齐全且无 UI 依赖;release 矩阵 4 平台 + release job | pass |

Totals: **27/27 pass**。

## Manual Dogfood

- `make check` 全绿(fmt + clippy + test 79 项)
- `scripts/install.sh --prefix /tmp/popsicle-install-test`:release 构建 →
  安装 → `popsicle help` 正常 → `--uninstall` 移除干净
- `.git/hooks/pre-commit` 已安装(本次提交将真实过 hook)

## Test Suite

- `cargo test` 全工作区 79/79
- `tool run intent-validate path=products` exit 0

## Observations

- O-401:CI 不跑 shell golden 链与 intent-validate(需 intent-lang 工具链),
  本地以 `make golden` / `make intent` 承担;intent-lang 进 CI 记入 follow-up。

## Verdict

Equivalence gate **green**。Ready for cutover ADR-014。
