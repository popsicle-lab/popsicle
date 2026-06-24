---
id: 4666b920-eb7f-4d35-9d32-72a9f81c4d41
doc_type: implementation-coverage
title: cli-ux implementation coverage
status: final
skill_name: shadow-implementer
pipeline_run_id: faff72be-0378-49e0-8114-f050c2b3a2e0
spec_id: 308c231f-0e91-4922-87f0-22b63be857b6
version: 1
parent_doc_id: null
tags: []
metadata: null
created_at: 2026-06-10T03:38:08.084525Z
updated_at: 2026-06-10T04:00:00Z
---

---
artifact: implementation-coverage
slug: cli-ux-implementation-coverage
generated_by: shadow-implementer
slice: cli-ux
last_updated: 2026-06-10
crate: crates/cli-ux/
cargo_test_exit: 0
intent_blocks_total: 8
intent_blocks_covered: 8
query_anchors:
  - "这个 slice 的 intent 哪些已经有代码了？"
  - "还缺哪些 fn 或 test？"
---

# 实现覆盖报告 — cli-ux

> 由 `shadow-implementer` 产出。把 `products/cli-ux/intents/` 的每个
> acceptance/invariants 块映射到 `crates/cli-ux/` 的具体实现与测试。

## Scope Checklist

- [x] target slice 已确认：`cli-ux`
- [x] 全部 intent block 已枚举：acceptance 6 + invariants 2
- [x] File Manifest 路径清单已列出：ADR-007 / ARCHITECTURE.md
- [x] 每个 block 已标 已实现/部分/缺失
- [x] 跨界端口归属已核对：CLI shell 调用 `skill-runtime` / `artifact-system` / `storage`

## Summary

| 指标 | 值 |
|---|---|
| Slice | cli-ux |
| Crate | `crates/cli-ux/` |
| acceptance + invariants block 总数 | 8 |
| 已覆盖（fn 或 test）| 8 |
| `cargo test -p cli-ux` | PASS（exit 0）|

一句话结论：cli-ux 的 semantic shell 已 in-shadow 落地；6 条 golden 与 7 条 intent property 全绿，覆盖 PDR-001 的命令效果而非 legacy stdout byte parity。

## Intent 覆盖表

| Intent / Safety | 层级 | Task ID | 实现位置 | Test | 状态 |
|---|---|---|---|---|---|
| `InitShowsNextStep` | acceptance | T-CU-0001 | `lib.rs::run_command` (`Command::Init`) | `intent_properties.rs::init_shows_next_step` | ✅ |
| `IssueStartCreatesRun` | acceptance | T-CU-0002 | `lib.rs::parse_args`, `lib.rs::start_issue_run`, `lib.rs::run_command` (`Command::IssueStart`) | `intent_properties.rs::issue_start_creates_run`; `golden.rs::golden_002_issue_start_returns_run_id_and_lock_signal` | ✅ |
| `DocCommandWritesArtifact` | acceptance | T-CU-0003 | `lib.rs::create_document_artifact`, `lib.rs::run_command` (`Command::DocCreate`) | `intent_properties.rs::doc_command_writes_artifact_and_row`; `golden.rs::golden_003_doc_create_writes_artifact_and_document_row` | ✅ |
| `StageAdvanceReflectsState` | acceptance | T-CU-0004 | `lib.rs::complete_pipeline_stage`, `lib.rs::run_command` (`Command::StageComplete`) | `intent_properties.rs::stage_advance_reflects_state`; `golden.rs::golden_004_stage_complete_requires_confirm_then_advances` | ✅ |
| `ErrorsAreActionable` | acceptance | T-CU-0005 | `lib.rs::CliError::actionable`, `lib.rs::CliError::has_category_object_and_next_step`, `lib.rs::parse_args` | `intent_properties.rs::errors_are_actionable`; `golden.rs::golden_006_removed_commands_return_actionable_errors` | ✅ |
| `AdminCommandsAreExplicit` | acceptance | T-CU-0006 | `lib.rs::parse_args`, `lib.rs::admin_response`, `lib.rs::run_command` (`Command::Admin`) | `intent_properties.rs::admin_commands_are_explicit`; `golden.rs::golden_005_admin_commands_are_nested_under_admin` | ✅ |
| `RemovedCommandsStayRemoved` | invariants/safety | T-CU-0007 | `lib.rs::REMOVED_TOP_LEVEL_COMMANDS`, `lib.rs::contains_removed_top_level_command`, `lib.rs::removed_command_next_step` | `intent_properties.rs::render_top_level_help_keeps_removed_commands_removed`; `golden.rs::golden_001_help_exposes_idd_main_path_without_removed_commands` | ✅ |
| `RenderTopLevelHelp` | invariants/intent | T-CU-0007 | `lib.rs::TOP_LEVEL_COMMANDS`, `lib.rs::top_level_help`, `lib.rs::contains_removed_top_level_command` | `intent_properties.rs::render_top_level_help_keeps_removed_commands_removed`; `golden.rs::golden_001_help_exposes_idd_main_path_without_removed_commands` | ✅ |

状态：✅ 已覆盖 / ⚠️ 部分 / ❌ 缺失

## File Manifest 对账

| ADR/RFC 路径 | 预期责任 | 磁盘存在 | 备注 |
|---|---|---|---|
| `crates/cli-ux/` | thin IO shell：argv parsing、command dispatch、formatting、actionable errors | ✅ | `Cargo.toml` + `src/lib.rs` + tests 已存在 |
| `products/cli-ux/intents/contracts.intent` | `CliShellDelegatesToDomainCrates` unlocked by ADR-007 | ✅ | Status: ADR-007 Accepted |
| `products/cli-ux/decisions/adr/ADR-007-cli-ux-io-shell-boundary.md` | IO shell boundary | ✅ | Status: Accepted |
| `products/cli-ux/ARCHITECTURE.md` | File Manifest / dependency boundary living doc | ✅ | 内容仍显示 spec/planned，交 living-doc-author 刷新 |

## cargo test

```
cargo test -p cli-ux
exit 0

Running tests/golden.rs
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

Running tests/intent_properties.rs
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## 待办

| 项 | 类型 | 跟进 |
|---|---|---|
| `src/main.rs` / installed `popsicle` binary entrypoint | CLI wiring | 目前 coverage 锁定 semantic shell；真实 binary exposure 由 cutover/living-doc 阶段决定 |
| legacy stdout byte parity | scope decision | PDR-001 明确不锁 legacy 文案字节；equivalence baseline 以 semantic golden 为准 |
| `PRODUCT.md` / `ARCHITECTURE.md` / `tasks/README.md` 状态刷新 | living docs | 交 `living-doc-author`，不要在 implementation artifact 中冒充完成 |

## 检查清单

- [x] 每个 acceptance block 在表中有行
- [x] 每个 ✅ 行有具体 `文件::符号` 与 test 名
- [x] File Manifest 与 ADR Consequences 一致
- [x] cargo test 已实跑并记录 exit code
- [x] 待办项未冒充 ✅
