---
doc_type: living-doc-author
id: doc-59
pipeline_run_id: 0000001a-0000-401a-8001-1a00000000001a
status: active
title: PROJ-26 living doc sync report
version: 1
---

# PROJ-26 living doc sync report

> **Issue**: PROJ-26 · migrate legacy devops tooling
> **Stage**: living-docs (slice-delivery)
> **Decision-Ref**: ADR-014
> **Date**: 2026-06-11

## Synced Documents

| 文档 | 改动 | 依据 |
|---|---|---|
| AGENTS.md(根)| 新增「DevOps Entry Points」段:make check/golden/intent、install-hooks、install.sh、release 触发方式 | ADR-014 §1-5 |
| products/cli-ux/PRODUCT.md | Status/Last-Decision-Ref → ADR-014;Roadmap 增 ADR-014 | ADR-014 |
| migration/progress.md | slice-3 行更新为 PROJ-26 devops ✓ + 27/27 golden | ADR-014 §Compliance |

## Consistency Checks

- 本 run 的 doc-56/57/58 与本文档全部经 doc check 通过
- `make check` 与 pre-commit hook 在迁移当场全绿,CI 上线即绿可期

## Not Synced (intentionally)

- README.md:面向最终用户的安装说明待 release 流程首次跑通后一并更新
- LEGACY_PIN.md:pin 不变,无需更新

## Follow-ups(传下一轮)

1. O-401:intent-lang 进 CI(让 make golden / make intent 也在 CI 跑)
2. D-401:completions 命令裁决落地后恢复安装脚本的补全段
3. O-301(上轮):doctor 二进制 staleness 检测
4. README 安装章节随首次 release 更新
