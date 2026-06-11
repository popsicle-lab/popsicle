# ADR-013 · SQLite Phase 2 storage backend

> **Status**: Accepted
> **Date**: 2026-06-11
> **Product**: cli-ux
> **Generated-by**: cutover-author（PROJ-25，追踪号 PROJ-11）
> **Source-Baseline**: `docs/baseline/2026-06-11/cli-ux-sqlite-phase2/`
> **Closes**: ADR-009 Phase 2 tracking（PROJ-11）

## Context

ADR-009 以 `state.tsv` 作为 Phase 1 临时索引态存储,Phase 2 SQLite 由
PROJ-11 追踪。TSV 无 schema、无事务、字段含义靠列序约定;且 doctor 长期
硬编码报告 tsv 后端。

## Decision

1. **storage crate 拥有 SQLite 层**:`SqliteStateDb`(rusqlite 0.32 bundled,
   免系统依赖),meta/issues/runs/documents 四表,事务内全量快照读写——
   状态规模为数十行,保留与 TSV 相同的 load-all/save-all 语义,增量查询
   不在本期范围。
2. **db 路径 `.popsicle/self-host/state.db`,而非 ADR-004 注释里的
   `.popsicle/popsicle.db`**:后者是 legacy 二进制的数据库(schema 完全
   不同),首次联调即冲突。divergence 在此正式记录;legacy db 退役后再
   议路径统一。
3. **后端自动检测**:`state.db` 存在 → SQLite;仅 `state.tsv` 存在 → TSV
   (遗留工作区继续可读可写,不被擅自迁移);全新工作区 → SQLite 默认。
4. **`admin migrate` 落地为真迁移**:TSV → SQLite 保行保计数器,原文件留底
   `state.tsv.migrated`,幂等;响应携带 `migrated` 与 `storage_backend`。
5. **会话工作文件保持 per-run JSON**(不入库):人工可检视、按 run 原子写、
   与 db 索引态职责分离。
6. **doctor `storage_backend` 动态化**;`TsvWorkspace` 更名 `LocalWorkspace`,
   `RunRow` 上移至 storage。

## Divergences / Deferred

- **D-301**:db 路径偏离 ADR-004 注释(见 Decision 2),属有意偏差。
- **O-301**:doctor 不检测二进制 staleness——本次沙箱 `CARGO_TARGET_DIR`
  污染导致旧二进制"静默无效"暴露了这一盲区;build 时间戳/版本指纹检测
  记入 follow-up。
- 会话 JSON 入库、legacy popsicle.db 读取/合并:不做。
- rusqlite 锁定 0.32(0.38 需 unstable `cfg_select`);工具链升级后可解锁。

## Compliance

| 门禁 | 证据 | 结果 |
|---|---|---|
| Intent Z3 | `tool run intent-validate path=products` exit 0 | pass |
| Golden | 全链 23/23(18 回归 + 5 新契约);golden-008 / usability-004 按本 ADR 修订并注明 | pass |
| cargo test | 全工作区 79/79(新增 sqlite 默认 / TSV 兼容 / 迁移幂等 3 项)| pass |
| Dogfood | 真实工作区 `admin migrate` 成功:8 issues 无损、留底、幂等;本 run 全程在 SQLite 后端推进 | pass |

## Approval

- **Status**: Accepted
- **Approved by**: PROJ-25 slice-delivery cutover stage(user 授权 agent `--confirm`,见会话记录)
- **Approval date**: 2026-06-11
