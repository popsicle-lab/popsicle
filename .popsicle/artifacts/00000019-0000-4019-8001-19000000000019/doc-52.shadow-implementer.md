---
doc_type: shadow-implementer
id: doc-52
pipeline_run_id: 00000019-0000-4019-8001-19000000000019
status: active
title: PROJ-25 SQLite Phase 2 coverage
version: 1
---

# PROJ-25 SQLite Phase 2 coverage

> **Issue**: PROJ-25 · PROJ-11 SQLite Phase 2 storage backend
> **Stage**: implement (slice-delivery)
> **Date**: 2026-06-11

## Scope

ADR-009 Phase 2(追踪号 PROJ-11):把索引态存储从 state.tsv 升级为 SQLite
单文件库,保持 WorkspaceStore 契约与 smoke 契约不变。

## Changes Delivered

### 1. storage crate 拥有 SQLite 层

- 新增 `rusqlite 0.32 (bundled)` 依赖(0.38 需要 unstable cfg_select,与
  rustc 1.94 不兼容,显式降级)。
- 新模块 `storage::sqlite`:`SqliteStateDb`(schema 建表/load/save,事务内
  全量快照写入)+ `StateSnapshot`;`RunRow` 上移到 storage(替换 cli-ux 私有
  RunIndex)。
- 行级 schema:meta(k/v 计数器)、issues、runs、documents(含 parent_id;
  body 不入库,正文始终在 artifact 文件)。

### 2. cli-ux 后端自动检测与双后端

- `TsvWorkspace` 更名 `LocalWorkspace`,持有 `StateBackend`(Tsv | Sqlite)。
- 检测序:已有 `state.db` → Sqlite;遗留 `state.tsv` → Tsv(继续可读可写);
  全新工作区 → Sqlite 默认。
- db 路径:`.popsicle/self-host/state.db` —— 显式避开 `.popsicle/popsicle.db`
  (那是 legacy 二进制的库,schema 不同,见 P1)。
- 会话工作文件保持 per-run JSON(可人工检视、按 run 原子),决策记入 ADR-013。

### 3. admin migrate 落地为真迁移

- `LocalWorkspace::migrate_to_sqlite()`:TSV → SQLite,原 TSV 改名
  `state.tsv.migrated` 留底;幂等(已是 SQLite 时报 migrated=false)。
- `AdminResult` 增加 `details` 字段;migrate 响应带 `migrated` 与
  `storage_backend`。
- doctor 的 `storage_backend` 字段动态反映当前后端。

### 4. 真实工作区 dogfood 迁移

- `admin migrate` 在本仓库执行成功:8 issues 无损、计数器保留、留底文件在,
  re-migrate 幂等返回 false。

## Problems Hit & Optimizations (记录并优化)

| # | 问题 | 处置 |
|---|---|---|
| P1 | 计划用 `.popsicle/popsicle.db`,但该路径已被 legacy 二进制的库占用(schema 完全不同),首跑 doctor 即报 "no such column" | db 改放 `.popsicle/self-host/state.db`;ADR-013 记录与 ADR-004 注释的偏差;AGENTS.md 明示不要碰 legacy db |
| P2 | rusqlite 最新版(libsqlite3-sys 0.38)要求 unstable `cfg_select`,rustc 1.94 编译失败 | 锁定 rusqlite 0.32 (bundled) |
| P3 | 沙箱把 `CARGO_TARGET_DIR` 持久化进 shell 环境,后续构建全进沙箱缓存,`./target/debug/popsicle` 停留旧版,导致 admin migrate "静默无效" | unset 后重建即恢复;这正是 doctor provenance 设计要抓的场景——但 doctor 只查二进制路径匹配,不查新旧,`build_timestamp`/staleness 检测记入 follow-up |
| P4 | smoke 的 binary-match 断言在 CARGO_TARGET_DIR 重定向环境下环境性失败 | 改为条件断言(仅当测试二进制即工作区二进制);无条件校验由 golden-008 用真实二进制承担 |
| P5 | 并行测试 temp 目录用纳秒命名偶发撞名 | 加进程内原子序号后缀根治 |

## Verification

- `cargo test` 全工作区:79/79 通过(local_workspace 10,新增 sqlite 默认/
  TSV 兼容/迁移幂等 3 项;smoke e2e 全程跑在 SQLite 后端)
- 全链 golden 23/23:self-host 8 + alignment 5 + usability 5 + sqlite-phase2 5
- golden-008(ADR-010)与 usability golden-004 按 ADR-013 修订(动态后端、
  后端无关计数),修订注释写明 Amended-by
- `tool run intent-validate path=products` exit 0

## Out of Scope

- 会话 JSON 入库(保留文件形态,ADR-013 决策)
- doctor 二进制新旧检测(follow-up)
- legacy popsicle.db 的读取/合并

- [x] storage::sqlite 模块 + 事务化快照读写
- [x] 后端自动检测(db 优先,TSV 兼容,新建默认 SQLite)
- [x] admin migrate 真迁移 + 幂等 + 留底
- [x] 真实工作区迁移 dogfood 完成
- [x] 测试与全链 golden 全绿
