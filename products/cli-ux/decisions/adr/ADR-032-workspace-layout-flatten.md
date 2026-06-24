# ADR-032 · 工作区布局扁平化（self-host/ 上提）

> **Status**: Accepted
> **Date**: 2026-06-24
> **Product**: cli-ux
> **Source-Issue**: PROJ-62
> **Supersedes**: ADR-013 Decision 2 路径（`.popsicle/self-host/state.db`）

## Context

ADR-013 将 SQLite 放在 `.popsicle/self-host/` 以避开 legacy `.popsicle/popsicle.db`。
ADR-031 后 legacy db 已 purge，self-host 子目录只剩历史命名包袱；`artifacts/` 已在
`.popsicle/` 根，索引与 session 文件分属两层不合理。

## Decision

1. **扁平 runtime 布局**：
   - `.popsicle/state.db` — 唯一 Issue/Run/Doc 索引
   - `.popsicle/runs/*.json` — pipeline session 工作文件
   - `.popsicle/artifacts/`、`project.yaml`、`modules/` — 不变
2. **自动迁移**：打开工作区时若存在 `.popsicle/self-host/state.db` 且无 flat db → rename 上提；TSV import-only 逻辑不变。
3. **`admin relocate-workspace [--dry-run]`**：幂等显式入口；`doctor` 对残留 `legacy_self_host_subdir` 报告 `legacy_relocate_next`。
4. **Rust 模块**：`workspace.rs` + `WorkspaceDomain`（PROJ-63 完成 ADR-032 遗留 rename）。

## File Manifest

| Path | Change |
|---|---|
| `crates/cli-ux/src/workspace.rs` | 路径 + 迁移 + admin + domain |
| `crates/cli-ux/src/lib.rs` | CLI surface |
| `crates/cli-ux/tests/local_workspace.rs` | 迁移单测 |
| `docs/baseline/**` | golden 路径断言 |
| `AGENTS.md` / `PROJECT_CONTEXT.md` / `README.md` | 活文档 |

## Compliance

| 门禁 | 结果 |
|---|---|
| `make check` | pass |
| dogfood 自动迁移 | `.popsicle/self-host/` 移除 |
| 行为 | Issue/Run/Doc 语义不变 |

## Approval

- **Status**: Accepted
- **Approved by**: PROJ-62 platform-refactor adr-close stage
- **Approval date**: 2026-06-24
