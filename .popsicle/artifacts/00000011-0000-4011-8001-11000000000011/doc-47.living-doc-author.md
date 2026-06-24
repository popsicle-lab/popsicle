---
doc_type: living-doc-author
id: doc-47
pipeline_run_id: 00000011-0000-4011-8001-11000000000011
status: active
title: PROJ-17 living doc sync report
version: 1
---

# PROJ-17 living doc sync report

> **Issue**: PROJ-17 · cli-ux command surface alignment
> **Stage**: living-docs (slice-delivery)
> **Decision-Ref**: ADR-011
> **Date**: 2026-06-11

## Synced Documents

| 文档 | 改动 | 依据 |
|---|---|---|
| `AGENTS.md`（根） | 全文重写绑定实现面：二进制解析含 `target/debug`；强制清单只用已实现命令；5 个真实 pipeline 模板表；「Deferred & Removed Commands」节 + 替代实践表 | ADR-011 §6（implement 阶段完成，本阶段核对一致） |
| `products/cli-ux/PRODUCT.md` | Status/Last-Decision-Ref → ADR-011；「用户视角的入口」重写为 7 实现族 + deferred/removed 清单；Roadmap 增 ADR-010/011 条目并标注 PDR-001 被修订 | ADR-011 §1-4 |
| `products/cli-ux/decisions/pdr/PDR-001-cli-ux-command-tree-redesign.md` | 头部加 **Amended-by: ADR-011** 标注，说明 preserve 17 → 7 implemented + 10 deferred 的二次裁决 | ADR-011 §Decision |
| `migration/progress.md` | Last-Decision-Ref → ADR-011；slice-3 行更新为 PROJ-17 command alignment ✓ + ADR-011 摘要 | ADR-011 §Compliance |

## Consistency Checks

- PRODUCT.md 入口清单与 `TOP_LEVEL_COMMANDS`/`COMMAND_USAGE` 逐项核对一致。
- PDR-001 处置表原文未改（保留历史决策原貌），仅以 Amended-by 标注修订关系。
- AGENTS.md 中所有出现的命令均可被 `parse_args` 接受（golden-003 二进制级断言）。

## Not Synced (intentionally)

- `docs/PROJECT_CONTEXT.md`：无 collab 触发条件类变化，不动。
- `docs/glossary.md`：未引入新术语（deferred 在 ADR-011 内定义）。
- intent-coder 上游模板：本次不反传（无模板层变化）。

## Follow-up Issues (from ADR-011)

1. D-101：bundle 默认管线模板或改写 `IssueType::default_pipeline`
2. O-102：smoke 测试隔离到临时工作区（顺带清理 23 个 smoke issue 残留）
3. `doc check` 实现（PDR-001 指定的 checklist 替代物）
4. PROJ-11：SQLite Phase 2
5. 10 个 deferred 命令逐个永久裁决（PDR 修订）
