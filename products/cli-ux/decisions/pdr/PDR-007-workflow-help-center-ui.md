# PDR-007: 工作流帮助中心 UI（Pipeline / Skill 目录）

> **Status**: Accepted
> **Date**: 2026-06-23
> **Product**: cli-ux
> **Source**: PROJ-57
> **Related ADRs**: ADR-027

## Decision Context

开发者需要理解 intent-coder 的 Pipeline 模板与 Skill 技能，但此前只能在 YAML / guide.md 中手工查找；Issue 运行 pipeline 时也缺少「当前阶段对应哪个 skill」的上下文帮助。

## Decision

1. **侧栏「工作流帮助」**：只读 master-detail 页，双 Tab（Pipeline 模板 / Skills）。
2. **`get_workflow_catalog`**：Tauri API，聚合 bundled/installed pipelines + intent-coder `skill.yaml`。
3. **Issue 阶段上下文**：从 Issue 详情 / Pipeline 运行页跳转时带入 `contextRunId`，高亮当前 stage。
4. **非迁移能力**：无 legacy golden 对账；以单元测试 + UI build + intent-validate 为门禁。

## Intent Impact

| Intent | Task | File |
|---|---|---|
| `WorkflowHelpCenterBrowsable` | T-CU-0017 | acceptance.intent Block 18 |

## Validation Plan

- `intent-validate path=products/cli-ux/intents` exit 0
- `cargo test -p cli-ux --test workflow_catalog`
- `cargo build -p cli-ux --features ui` + `npm run build`
- `make check`
