---
doc_type: living-doc-author
id: doc-55
pipeline_run_id: 00000019-0000-4019-8001-19000000000019
status: active
title: PROJ-25 living doc sync report
version: 1
---

# PROJ-25 living doc sync report

> **Issue**: PROJ-25 · PROJ-11 SQLite Phase 2
> **Stage**: living-docs (slice-delivery)
> **Decision-Ref**: ADR-013
> **Date**: 2026-06-11

## Synced Documents

| 文档 | 改动 | 依据 |
|---|---|---|
| AGENTS.md(根)| admin migrate 说明更新为真迁移;新增 Storage 段(state.db 路径、TSV 兼容、不要碰 legacy popsicle.db)| ADR-013 §2-4 |
| products/cli-ux/PRODUCT.md | Status/Last-Decision-Ref → ADR-013;Roadmap 增 ADR-013,PROJ-11 标记关闭 | ADR-013 |
| migration/progress.md | slice-3 行更新为 PROJ-25 SQLite ✓ + 23/23 golden + PROJ-11 closed | ADR-013 §Compliance |
| docs/baseline/.../cli-ux-self-host/golden-008 | storage_backend 断言动态化,注明 Amended-by ADR-013 | ADR-013 §6 |
| docs/baseline/.../cli-ux-usability/golden-004 | 计数改为 issue list,后端无关,注明 Amended-by ADR-013 | ADR-013 §3 |

## Consistency Checks

- 本 run 的 doc-52/53/54 与本文档全部经 doc check 通过
- 全链 golden 23/23 在迁移后的 SQLite 工作区复跑通过

## Not Synced (intentionally)

- ADR-004 注释中的 popsicle.db 路径:作为历史决策文本保留,偏差由 ADR-013
  D-301 正式记录
- docs/PROJECT_CONTEXT.md:无新术语

## Follow-ups(传下一轮)

1. O-301:doctor 二进制 staleness 检测(build 指纹)
2. O-201/O-202(上轮遗留):doc check 反引号豁免、admin prune
3. rusqlite 版本随工具链升级解锁
4. 10 个 deferred 命令逐个永久裁决
