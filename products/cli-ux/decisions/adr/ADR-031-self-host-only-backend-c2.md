# ADR-031 · Self-host 为唯一 workspace 后端（C2 脱离 legacy submodule）

> **Status**: Accepted
> **Date**: 2026-06-24
> **Product**: cli-ux
> **Source-Issue**: PROJ-61
> **Supersedes**: ADR-009 Phase 1 TSV 运行时；`legacy/popsicle/` submodule 维护模型

## Context

迁移 slice 已 cutover-done，但仓库仍携带：

- `legacy/popsicle/` git submodule（仅 golden 文档引用，无 CI 硬依赖）
- `.popsicle/popsicle.db`（legacy 二进制 schema，self-host 不读）
- `StateBackend::Tsv` 读写路径（dogfood 已 SQLite）

继续维护三轨存储与 submodule 增加心智负担，与「self-host 为标准」目标冲突。

## Decision

### C2 — 脱离 legacy submodule

1. **删除** `legacy/popsicle/` submodule 与 `.gitmodules` 条目。
2. **Frozen baseline** 保留在 `docs/baseline/`（含 pin commit `c76d729` 的说明）；`LEGACY_PIN.md` 改为归档索引。
3. **`make golden`** 定义为 **self-host 回归链**（`docs/baseline/2026-06-11/cli-ux-sqlite-phase2/run-all.sh`），不再要求 live legacy 源码树。
4. **equivalence-baseline** 新工作默认 `mode=tests`；legacy diff 仅作历史 baseline YAML 只读引用。

### Self-host 唯一 runtime

1. **唯一索引库**：`.popsicle/self-host/state.db`（+ `runs/*.json` + `.popsicle/artifacts/`）。
2. **移除 TSV 运行时**：删除 `StateBackend::Tsv` 与 `save_tsv`；打开工作区时若仅有 `state.tsv` 则 **一次性导入** SQLite 并 rename 为 `state.tsv.migrated`。
3. **`admin migrate`** 保留为幂等入口（TSV → SQLite），行为与自动导入一致。
4. **`admin purge-legacy-workspace [--dry-run]`**：删除 `.popsicle/popsicle.db`；`doctor` 对残留 legacy 文件报告 `legacy_artifacts` 警告。

## File Manifest

| Path | Change |
|---|---|
| `crates/cli-ux/src/self_host.rs` | SQLite-only；TSV 导入；doctor/admin |
| `.gitmodules` | 删除 submodule |
| `legacy/popsicle/` | 删除 |
| `LEGACY_PIN.md` | 归档说明 |
| `README.md` / `PROJECT_CONTEXT.md` / `Makefile` | self-host 单一叙事 |
| `docs/baseline/**/golden-*.sh` | 仅 sqlite 断言 |

## Compliance

| 门禁 | 结果 |
|---|---|
| `make check` | required pass |
| dogfood `admin purge-legacy-workspace` | 移除 popsicle.db |
