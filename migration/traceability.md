# Migration Traceability

> **Status**: skill-runtime + artifact-system + cli-ux cutover-done
> **Last-Updated**: 2026-06-11

| Legacy 路径（`legacy/popsicle/`）| 新位置（`popsicle-new`）| 责任 Spec | 切流 ADR | 等价性 baseline | 状态 |
|---|---|---|---|---|---|
| `crates/popsicle-core/src/registry/loader.rs` | `crates/skill-runtime/src/loader.rs` | slice-1-skill-runtime | ADR-005 | `docs/baseline/2026-06-09/skill-runtime/` | cutover-done |
| `crates/popsicle-core/src/model/skill.rs`（SkillLoadResult 语义）| `crates/skill-runtime/src/skill_load.rs` | slice-1-skill-runtime | ADR-005 | G-001 | cutover-done |
| `crates/popsicle-core/src/model/pipeline.rs`（PipelineRun 编排）| `crates/skill-runtime/src/pipeline_session.rs` | slice-1-skill-runtime | ADR-005 | G-006 | cutover-done |
| `crates/popsicle-core/src/engine/context_layer.rs`（MemoriesLayer）| `crates/skill-runtime/src/memory_layer.rs` | slice-1-skill-runtime | ADR-005 | — | cutover-done |
| `crates/popsicle-core/src/model/issue.rs` | `crates/skill-runtime/src/issue.rs`（MVP）| slice-1-skill-runtime | ADR-005 | — | cutover-done |
| `crates/popsicle-core/src/storage/index.rs`（documents 表）| `crates/storage/src/document_row.rs` | slice-1-skill-runtime | ADR-004 + ADR-005 | — | cutover-done（MemoryStore 占位）|
| `crates/popsicle-core/src/model/document.rs` | `crates/artifact-system/src/document.rs` | slice-2-artifact-system | ADR-006 | G-001 | cutover-done |
| `crates/popsicle-core/src/engine/guard.rs` | `crates/artifact-system/src/guard.rs` | slice-2-artifact-system | ADR-004 + ADR-006 | G-002/G-006 | cutover-done |
| `crates/popsicle-core/src/engine/context.rs` / `engine/context_layer.rs` | `crates/artifact-system/src/context.rs` | slice-2-artifact-system | ADR-004 + ADR-006 | G-003 | cutover-done |
| `crates/popsicle-core/src/engine/extractor.rs` | `crates/artifact-system/src/extractor.rs` | slice-2-artifact-system | ADR-004 + ADR-006 | G-004 | cutover-done |
| `crates/popsicle-core/src/model/work_item.rs` | `crates/artifact-system/src/task_chunk.rs` | slice-2-artifact-system | ADR-006 | G-005 | cutover-done |
| `crates/popsicle-core/src/storage/index.rs`（documents row shape）| `crates/storage/src/document_row.rs` | slice-2-artifact-system | ADR-004 + ADR-006 | — | cutover-done（SQLite wiring 延后）|
| `crates/popsicle-cli/src/main.rs`（top-level command surface）| `crates/cli-ux/src/lib.rs::TOP_LEVEL_COMMANDS` / `top_level_help` + `crates/cli-ux/src/main.rs` | slice-3-cli-ux | ADR-008 | G-001 / `docs/baseline/2026-06-10/cli-ux/` | cutover-done |
| `crates/popsicle-cli/src/commands/issue.rs` + `pipeline.rs`（issue start/run signal）| `crates/cli-ux/src/lib.rs::start_issue_run` | slice-3-cli-ux | ADR-008 | G-002 | cutover-done |
| `crates/popsicle-cli/src/commands/doc.rs`（artifact + document row）| `crates/cli-ux/src/lib.rs::create_document_artifact` | slice-3-cli-ux | ADR-008 | G-003 | cutover-done |
| `crates/popsicle-cli/src/commands/pipeline.rs`（stage complete approval）| `crates/cli-ux/src/lib.rs::complete_pipeline_stage` | slice-3-cli-ux | ADR-008 | G-004 | cutover-done |
| `crates/popsicle-cli/src/commands/{admin,migrate,reinit}.rs` | `crates/cli-ux/src/lib.rs::AdminCommand` / `parse_args` | slice-3-cli-ux | ADR-008 | G-005 | cutover-done |
| legacy `checklist` / `item` / `sync` command families | `REMOVED_TOP_LEVEL_COMMANDS` + actionable errors | slice-3-cli-ux | ADR-008 | G-006 | cutover-done |
| legacy issue/pipeline/doc CLI（full SQLite）| `crates/cli-ux/src/self_host.rs` + `.popsicle/self-host/` | slice-3-cli-ux | ADR-010 | `docs/baseline/2026-06-11/cli-ux-self-host/` | cutover-done（TSV Phase 1）|
| legacy `popsicle.db` IndexDb | Phase 2 `WorkspaceStore` SQLite（PROJ-11）| slice-3-cli-ux | ADR-013 | `docs/baseline/2026-06-11/cli-ux-sqlite-phase2/` | cutover-done |
| `legacy/popsicle/ui/`（14 页 Tauri SPA）| `ui/`（MVP+：Issues/Pipeline/Document/Task/Intent）| slice-4-ui | ADR-015 | `docs/baseline/2026-06-11/cli-ux-ui/` | cutover-done |
| `legacy/popsicle/crates/popsicle-cli/src/ui/commands.rs` | `crates/cli-ux/src/ui/commands.rs` | slice-4-ui | ADR-015 | G-002/G-003 | cutover-done |
| `legacy/popsicle/crates/popsicle-core`（IndexDb Tauri IPC）| `LocalWorkspace` + `.popsicle/self-host/state.db` | slice-4-ui | ADR-015 D-501 | — | cutover-done |
| `legacy/popsicle` 多项目 / 最近打开（无 global.json）| `global_config.rs` + UI `ProjectSwitcher` | slice-4-ui | ADR-016 | `docs/baseline/2026-06-11/cli-ux-project-ui/` | cutover-done |
| `popsicle project *` CLI 注册表 | Tauri `open_project_cmd` / `list_registered_projects` | slice-4-ui | ADR-016 | `docs/baseline/2026-06-11/cli-ux-global/` | cutover-done |

## 规则

- 切流 ADR **Accepted** 前状态一律 `in-shadow`。
- CLI 字节级 parity 不作为 cli-ux cutover 门禁；PDR-001 / ADR-007 锁定 semantic command effects。
