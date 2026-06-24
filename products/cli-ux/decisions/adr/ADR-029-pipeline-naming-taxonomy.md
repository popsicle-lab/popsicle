# ADR-029 · Pipeline 命名体系（域前缀 taxonomy）

> **Status**: Accepted
> **Date**: 2026-06-23
> **Product**: cli-ux
> **Source-Issue**: PROJ-59

## Context

Pipeline 名称（`slice-spec`、`slice-delivery`、`bugfix` 等）无法让用户快速判断：
是否涉及 legacy/golden/cutover、对象是 migration slice 还是日常 feature、属于 spec 还是 delivery。

PROJ-46/57/58 反复出现 pipeline 误选；`workflow.profile` 默认映射亦不够直观。

## Decision

采用 **`{domain}-{object?}-{phase}`** 命名：

| 域前缀 | 含义 | Pipeline |
|--------|------|----------|
| `migration-` | legacy / golden / cutover | `migration-bootstrap`, `migration-slice-spec`, `migration-slice-delivery` |
| `product-` | 绿地 product | `product-greenfield-spec` |
| `feature-` | 已有 product 增量能力 | `feature-spec`, `feature-delivery` |
| `doc-` | 仅文档 | `doc-retro-spec`, `doc-sync-weekly` |
| `fix-` | 单点回归 | `fix-regression` |
| `arch-` | 架构 ADR 链 | `arch-decision` |
| `platform-` | 内部重构 | `platform-refactor` |

**`slice` 仅出现在 `migration-*` 下**，不与日常 feature 混用。

### 废弃 alias（仍可通过 `canonical_pipeline_name` 解析）

| 旧名 | 新名 |
|------|------|
| `slice-spec` | `migration-slice-spec` |
| `slice-delivery` | `migration-slice-delivery` |
| `greenfield-product-spec` | `product-greenfield-spec` |
| `tech-decision` | `arch-decision` |
| `bugfix` | `fix-regression` |
| `weekly-health-check` | `doc-sync-weekly` |

### 默认 pipeline（`workflow.profile`）

| profile | technical | bug |
|---------|-----------|-----|
| `daily-dev` | `feature-delivery` | `fix-regression` |
| `migration` | `migration-slice-spec` | `fix-regression` |
| `pm-spec-only` | `feature-spec` | `fix-regression` |

## File Manifest

| Path | Change |
|---|---|
| `crates/cli-ux/src/pipeline_taxonomy.rs` | 新增 canonical + alias |
| `intent-coder/pipelines/*.pipeline.yaml` | 重命名 + 新增 4 条 |
| `crates/cli-ux/src/pipeline_gate.rs` | 门禁改用 canonical |
| `crates/cli-ux/src/project_config.rs` | 默认映射 |
| `crates/cli-ux/src/workflow_catalog.rs` | `pipeline_domain` 分组 |
| `ui/src/pages/WorkflowsView.tsx` | 域 badge |
| `skill-runtime/src/issue.rs` | type 默认 |

## Compliance

| 门禁 | 结果 |
|---|---|
| `make check` | pass |
| alias 解析 | `pipeline_taxonomy` + `intent_coder_resolve` 测试 |
