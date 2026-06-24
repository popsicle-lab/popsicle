---
doc_type: living-doc-author
id: doc-51
pipeline_run_id: 00000018-0000-4018-8001-18000000000018
status: active
title: PROJ-24 living doc sync report
version: 1
---

# PROJ-24 living doc sync report

> **Issue**: PROJ-24 · self-host usability completion
> **Stage**: living-docs (slice-delivery)
> **Decision-Ref**: ADR-012
> **Date**: 2026-06-11

## Synced Documents

| 文档 | 改动 | 依据 |
|---|---|---|
| AGENTS.md(根)| 管线表加 bugfix + 默认列;issue close / doc check 进命令参考与工作流规则(新增规则 11);替代实践表更新(checklist→doc check,verify→status+close)| ADR-012 §1-5 |
| products/cli-ux/PRODUCT.md | Status/Last-Decision-Ref → ADR-012;入口清单加 close/check 与 bundled 默认管线;Roadmap 增 ADR-012 | ADR-012 |
| migration/progress.md | slice-3 行更新为 PROJ-24 usability ✓ + dogfood-usable 标记 | ADR-012 §Compliance |

## Consistency Checks

- AGENTS.md 所有命令均可被 parse_args 接受(golden-003 二进制断言仍通过)
- 本 run 的 doc-48/49/50 与本文档全部经 doc check 通过(自举验证新命令)

## Not Synced (intentionally)

- docs/PROJECT_CONTEXT.md / glossary:无新术语与触发条件变化
- PDR-001:doc check 承诺已兑现,Amended-by 标注(ADR-011)已涵盖,不再叠加

## Follow-ups(传下一轮)

1. O-201:doc check 反引号/代码块占位符豁免
2. O-202:admin prune 类命令替代手工 state.tsv 清理
3. 字段命名统一(id vs doc_id)
4. PROJ-11 SQLite Phase 2
5. 10 个 deferred 命令逐个永久裁决
