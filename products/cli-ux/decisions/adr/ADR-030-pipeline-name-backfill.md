# ADR-030 · Pipeline 命名后续清理与 backfill（PROJ-60）

> **Status**: Accepted
> **Date**: 2026-06-24
> **Product**: cli-ux
> **Source-Issue**: PROJ-60
> **Related**: ADR-029

## Context

ADR-029 引入 canonical pipeline 名与 alias，但 dogfood 工作区仍存：

- `.popsicle/pipelines/` 下 6 个废弃 alias YAML
- SQLite `issues.pipeline` / `runs.pipeline_name` 全为旧字符串（33/33）
- 活文档与 golden 脚本仍引用 `slice-delivery` / `bugfix` 等

## Decision

1. **`popsicle admin backfill-pipeline-names [--dry-run]`**
   - 将 issues/runs 中 alias 字段 rewrite 为 canonical
   - 删除 `.popsicle/pipelines/{alias}.pipeline.yaml` 安装副本
2. **活文档同步**：README、CONTRIBUTING、intent-coder/README、UI 文案、golden 断言改用 canonical 名；历史 ADR「Approved by slice-delivery」不改。
3. **dogfood `.popsicle/pipelines/`**：仅保留仍有效的 `migration-bootstrap.pipeline.yaml`；canonical 文件由 init/self-heal 按需写入。

## File Manifest

| Path | Change |
|---|---|
| `crates/cli-ux/src/self_host.rs` | `admin_backfill_pipeline_names` |
| `crates/cli-ux/src/pipeline_taxonomy.rs` | `canonicalize_if_deprecated` |
| `AGENTS.md` | admin 命令 |
| 活文档 / golden / UI | canonical 名 |

## Compliance

| 门禁 | 结果 |
|---|---|
| `make check` | pass |
| dogfood backfill | 33 issues + 33 runs updated |
