# ADR-028 · intent-coder Pipeline/Skill 数据源统一（PROJ-58）

> **Status**: Accepted
> **Date**: 2026-06-23
> **Product**: cli-ux
> **Source-Issue**: PROJ-58
> **Supersedes**: ADR-017 中「pipeline 单独 `include_str!` assets/」的隐含双份维护（D-701 补充）

## Context

`workflow_catalog`（UI 帮助中心）与 `load_pipeline_def`（Issue 运行）对 Pipeline 的解析不一致：

- Skill：dogfood 优先读 `<root>/intent-coder/skills/`
- Pipeline：优先读 `.popsicle/pipelines/`，不读 live `intent-coder/pipelines/`

同时 `crates/cli-ux/assets/pipelines/` 与 `intent-coder/pipelines/` 内容重复，ADR-017 嵌入整包后仍保留历史 `include_str!` 列表。

## Decision

1. **`intent_coder_resolve.rs`**：统一 skills / pipelines 路径解析。
   - dogfood（存在 `intent-coder/module.yaml`）→ live `intent-coder/{skills,pipelines}/`
   - 否则 pipelines 顺序：`.popsicle/pipelines/` → `.popsicle/modules/intent-coder/pipelines/`
   - skills：live 或 module（与 ADR-017 dogfood override 一致）
2. **Bundled pipeline 单一来源**：`embedded_pipeline_names` / `embedded_pipeline_content` 从 `include_dir!(intent-coder)` 的 `pipelines/` 读取；删除 `crates/cli-ux/assets/pipelines/`。
3. **`load_pipeline_def` / `list_installed_pipeline_names` / `workflow_catalog`** 共用 `intent_coder_resolve`。
4. **Self-heal** 仍写入 `.popsicle/pipelines/`（兼容 init 与旧 workspace）。

## Consequences

| 场景 | 行为 |
|---|---|
| popsicle monorepo dogfood | 改 `intent-coder/pipelines/` 立即反映 UI + CLI |
| DMG / 新项目 init | 行为不变：`.popsicle/pipelines/` + module extract |
| 构建 | 仅 `intent-coder/pipelines/` 为 canonical YAML |
| 旧 workspace 残留 alias YAML / Issue 字段 | `admin backfill-pipeline-names`（PROJ-60 / ADR-029 后续） |

## File Manifest

| Path | Change |
|---|---|
| `crates/cli-ux/src/intent_coder_resolve.rs` | 新增 |
| `crates/cli-ux/src/intent_coder_bundle.rs` | embedded pipeline API |
| `crates/cli-ux/src/self_host.rs` | 重构 load/install |
| `crates/cli-ux/src/workflow_catalog.rs` | 共用 skills dir |
| `crates/cli-ux/tests/intent_coder_resolve.rs` | 集成测试 |
| `crates/cli-ux/assets/pipelines/` | 删除 |

## Compliance

| 门禁 | 结果 |
|---|---|
| `make check` | pass |
| `intent-validate path=products/cli-ux/intents` | pass |
| legacy golden | N/A（重构；`intent_coder_resolve` 测试替代） |
