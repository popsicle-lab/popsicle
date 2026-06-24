---
id: 19f11b37-2e7d-4f75-a564-9f5100b5bc1e
doc_type: implementation-coverage
title: cli-ux self-host implementation coverage
status: active
skill_name: shadow-implementer
pipeline_run_id: 8198f9ff-b120-4b0d-8dae-9b379dc9a1d8
version: 1
---

---
artifact: implementation-coverage
slug: cli-ux-self-host-implementation-coverage
generated_by: shadow-implementer
slice: cli-ux
last_updated: 2026-06-11
crate: crates/cli-ux/
cargo_test_exit: 0
intent_blocks_total: 9
intent_blocks_covered: 9
query_anchors:
  - "cli-ux self-hosting MVP 哪些 intent 有代码了？"
  - "TSV workspace 接在哪？"
---

# 实现覆盖报告 — cli-ux-self-host-implementation-coverage

> PROJ-10 / ADR-009 Phase 1：TSV workspace + doctor + IDD 主路径。

## Summary

| 指标 | 值 |
|---|---|
| Slice | cli-ux（self-host Phase 1）|
| Crate | `crates/cli-ux/` + `crates/storage/src/workspace.rs` |
| acceptance + invariants block 总数 | 9 |
| 已覆盖 | 9 |
| `cargo test -p cli-ux` | PASS（14 tests：6 golden + 7 property + 1 smoke）|

## Intent 覆盖表

| Intent / Safety | 层级 | Task | 实现位置 | Test | 状态 |
|---|---|---|---|---|---|
| `InitShowsNextStep` | acceptance | T-CU-0001 | `SelfHostDomain::init_workspace` | `intent_properties::init_shows_next_step` | ✅ |
| `IssueStartCreatesRun` | acceptance | T-CU-0002 | `TsvWorkspace::start_issue` | `intent_properties` + `smoke_workflow` | ✅ |
| `DocCommandWritesArtifact` | acceptance | T-CU-0003 | `TsvWorkspace::create_doc` | `intent_properties` + `smoke_workflow` | ✅ |
| `StageAdvanceReflectsState` | acceptance | T-CU-0004 | `TsvWorkspace::complete_stage` | `intent_properties` + `smoke_workflow` | ✅ |
| `ErrorsAreActionable` | acceptance | T-CU-0005 | `CliError` / `ws_err` | `intent_properties::errors_are_actionable` | ✅ |
| `AdminCommandsAreExplicit` | acceptance | T-CU-0006 | `parse_args` AdminCommand | `intent_properties::admin_commands_are_explicit` | ✅ |
| `RemovedCommandsStayRemoved` | acceptance | T-CU-0007 | `REMOVED_TOP_LEVEL_COMMANDS` | `golden_006` + property | ✅ |
| `SelfHostedWorkflowSmokePasses` | acceptance | T-CU-0008 | `SelfHostDomain` + `main.rs` | `smoke_workflow::self_host_workflow_smoke_passes` | ✅ |
| `BinaryProvenanceVisible` | acceptance | T-CU-0008 | `binary_provenance` / `doctor` | `smoke_workflow` + `golden-008-doctor.sh` | ✅ |
| `WorkflowSmokeDoesNotDependOnParentBinary` | invariants | T-CU-0008 | `binary_provenance` flags | doctor output | ✅ |

## File Manifest 对账

| 路径 | 责任 | 状态 |
|---|---|---|
| `crates/cli-ux/src/self_host.rs` | TSV workspace + doctor + tool run | ✅ |
| `crates/cli-ux/src/main.rs` | `SelfHostDomain` binary entry | ✅ |
| `crates/storage/src/workspace.rs` | `WorkspaceStore` trait（Phase 2 PROJ-11）| ✅ |
| `.popsicle/self-host/state.tsv` | Phase 1 issue/run/doc index | ✅ runtime |

## cargo test

```
cargo test -p cli-ux → 14 passed (6 golden + 7 property + 1 smoke)
```

## 待办（Phase 2 — PROJ-11）

| 项 | 跟进 |
|---|---|
| SQLite `popsicle.db` 新 schema | `SqliteWorkspaceStore` impl `WorkspaceStore` |
| Legacy `popsicle.db` 读取 | 不做（不兼容旧 popsicle）|
| SaaS billing dogfood 重跑 | smoke 通过后人工触发 |

## 检查清单

- [x] 每个 acceptance block 有行
- [x] File Manifest 与 ADR-009 一致
- [x] cargo test 实跑 exit 0
- [x] Phase 2 未冒充 ✅
