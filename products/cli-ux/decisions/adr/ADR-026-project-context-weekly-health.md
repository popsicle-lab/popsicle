# ADR-026 · Project context single source + weekly health pipeline（PROJ-55）

> **Status**: Accepted
> **Date**: 2026-06-23
> **Product**: cli-ux
> **Source-PDR**: PDR-006

## Context

工程画像分散在 PROJECT_CONTEXT 骨架、project.yaml 偏好、traceability 之间；living-doc 职责过载。需单一 Markdown 源、weekly 巡检拆分、Settings 编辑与 agent 注入。

## Decision

1. **`docs/PROJECT_CONTEXT.md`** 为唯一权威源；`project_context.rs` 读写与注入。
2. **`weekly-health-check`** bundled pipeline（无 approval）。
3. **Settings** Tauri `get_project_context_md` / `save_project_context_md`。
4. **`agent_prompt_context`** 追加 `[Project context]`（§工程画像，max 4KB）。
5. **living-docs**（slice-delivery）不含 tasks-index / product-context。

## File Manifest

| Path | Change |
|---|---|
| `crates/cli-ux/src/project_context.rs` | 新增 |
| `crates/cli-ux/src/project_config.rs` | 注入拼接 |
| `crates/cli-ux/src/self_host.rs` | bundled weekly-health-check |
| `crates/cli-ux/assets/pipelines/weekly-health-check.pipeline.yaml` | 新增 |
| `crates/cli-ux/assets/pipelines/slice-delivery.pipeline.yaml` | living-docs 说明 |
| `intent-coder/pipelines/weekly-health-check.pipeline.yaml` | 新增 |
| `intent-coder/skills/living-doc-author/*` | 频率分组口径 |
| `intent-coder/skills/project-init/skill.yaml` | PROJECT_CONTEXT 说明 |
| `docs/PROJECT_CONTEXT.md` | 填实 |
| `docs/MIGRATION.md` | 新增 A04 |
| `ui/src/pages/SettingsView.tsx` | 工程画像编辑区 |
| `ui/src/hooks/useTauri.ts` | IPC 封装 |
| `ui/src/i18n/messages.ts` | 文案 |
| `products/cli-ux/intents/acceptance.intent` | Block 15–17 |
| `products/cli-ux/tasks/daily-ops/T-CU-0016-*.md` | 新增 task |
| `crates/cli-ux/tests/project_context.rs` | 新增 |
| `docs/baseline/2026-06-23/cli-ux-weekly-health/` | golden |

## Compliance

| 门禁 | 结果 |
|---|---|
| `make check` | pass |
| intent-validate | pass |
