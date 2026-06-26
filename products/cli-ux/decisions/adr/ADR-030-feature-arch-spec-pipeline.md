# ADR-030 · feature-arch-spec pipeline

> **Status**: Accepted
> **Date**: 2026-06-26
> **Product**: cli-ux
> **Source-Issue**: PROJ-65
> **Supplements**: ADR-029

## Context

ADR-029 定义 `feature-spec`（轻量增量 spec）与 `product-greenfield-spec`（绿地 product）。已有 product 上的**大增量能力**（需 PDR、ADR、task、acceptance.intent，但非新 product）无对应 pipeline，导致误用 greenfield 或 arch-decision + feature-spec 两 Issue 串联。

## Decision

1. **新增 canonical pipeline `feature-arch-spec`**，域前缀 `feature-`（ADR-029 一致）。
2. **Stage 链**：`prd → arch-debate → rfc → adr → intent-spec → intent-check → living-docs`（无 `facts`、无 `product-debate`）。
3. **Delivery 后续**：spec 完成后使用既有 `feature-delivery` 实现。
4. **默认 profile 不变**：`pm-spec-only` / `daily-dev` 的 technical 默认仍为 `feature-spec` / `feature-delivery`；大增量 Issue **显式** `--pipeline feature-arch-spec`。
5. **无 deprecated alias**（首版）。

## Alternatives

| 方案 | 否决理由 |
|------|----------|
| 扩 feature-spec 加可选 stage | pipeline 引擎无 stage skip |
| product-greenfield-spec + 跳过 debate | 无 skip；反模式 |
| arch-decision + feature-spec 两 Issue | traceability 碎；无 living-docs |

## Consequences

- `intent-coder/pipelines/feature-arch-spec.pipeline.yaml` 新增并 bundled
- `intent-coder/skills/issue-author/guide.md` 决策树更新
- `AGENTS.md`、`intent-coder/README.md` pipeline 表补一行
- ADR-029 表 `feature-` 域增加 `feature-arch-spec`

## File Manifest

| Path | Change |
|------|--------|
| `intent-coder/pipelines/feature-arch-spec.pipeline.yaml` | 新增 |
| `intent-coder/README.md` | pipeline 表 |
| `intent-coder/skills/issue-author/guide.md` | 决策树 |
| `AGENTS.md` | pipeline 表 |
| `products/cli-ux/decisions/adr/ADR-030-feature-arch-spec-pipeline.md` | 本 ADR |
| `crates/cli-ux/src/pipeline_taxonomy.rs` | 测试补 `feature-arch-spec` domain |

## Intent Impact

| 层 | 变更 |
|----|------|
| cli-ux acceptance | 无 block 变更（工作流元数据） |
| docs/invariants | 无 |

## Approval

- **Status**: Accepted
- **Approval date**: 2026-06-26
