---
id: 8007be46-6ee3-4c46-ab08-c21f3b3c7318
doc_type: implementation-coverage
title: artifact-system implementation coverage
status: final
skill_name: shadow-implementer
pipeline_run_id: 249be474-d00c-4a2e-a8fc-c64b8aca08d9
spec_id: cf6e7062-b75f-4925-b633-82a04e775a76
version: 1
parent_doc_id: null
tags: []
metadata: null
created_at: 2026-06-09T09:56:25.608687Z
updated_at: 2026-06-09T09:56:25.608687Z
---

---
artifact: implementation-coverage
slug: artifact-system-implementation-coverage
generated_by: shadow-implementer
slice: artifact-system
last_updated: 2026-06-09
crate: crates/artifact-system/
cargo_test_exit: 0
intent_blocks_total: 6
intent_blocks_covered: 6
query_anchors:
  - "这个 slice 的 intent 哪些已经有代码了？"
  - "还缺哪些 fn 或 test？"
---

# 实现覆盖报告 — artifact-system-implementation-coverage

> 由 `shadow-implementer` 产出。把 `products/artifact-system/intents/` 的每个
> acceptance/invariants 块映射到 `crates/artifact-system/` 的具体实现与测试。

## Scope Checklist

- [x] 已读取 `products/artifact-system/intents/acceptance.intent`
- [x] 已读取 `products/artifact-system/intents/invariants.intent`
- [x] 已读取 `products/artifact-system/intents/contracts.intent`
- [x] 已对齐 ADR-004 / ADR-006 File Manifest
- [x] 已实跑 `cargo test -p artifact-system`

## Summary

| 指标 | 值 |
|---|---|
| Slice | artifact-system |
| Crate | `crates/artifact-system/` |
| acceptance + invariants block 总数 | 6 |
| 已覆盖（fn 或 test）| 6 |
| `cargo test -p artifact-system` | PASS（exit 0，15 intent_properties + 6 golden）|

一句话结论：acceptance 5 intent + invariants 1 safety/intent 均已映射到 `crates/artifact-system/` 纯函数与 property/golden tests；ADR-004 三个跨界 contract 由 guard port、extractor total 化、context layer ordering 覆盖。

## Intent 覆盖表

| Intent / Safety | 层级 | Task ID | 实现位置 | Test | 状态 |
|---|---|---|---|---|---|
| `DocumentRoundTrips` | acceptance | T-AS-0002 | `document::Document::{to_file_content,from_file_content,new_revision}` | `intent_properties::document_round_trips_preserves_body_id_version`; `golden_001_document_roundtrip` | ✅ |
| `GuardChecklistCompleteIffNoUnchecked` | acceptance | T-AS-0003 | `guard::{check_guard,checklist_outcome,count_checkboxes}` | `intent_properties::guard_checklist_complete_*`; `golden_002_guard_sections_and_checklist` | ✅ |
| `ContextAssemblyOrdersByRelevance` | acceptance | T-AS-0004 | `context::{context_includes_full_text,assemble_layers}` | `intent_properties::context_assembly_orders_by_relevance`; `golden_003_context_assembly_order` | ✅ |
| `ExtractPreservesKind` | acceptance | T-AS-0005 | `extractor::{extract_user_stories,extract_test_cases,extract_bugs}` | `intent_properties::extract_preserves_kind`; `golden_004_extractors_preserve_kind` | ✅ |
| `RenameWorkItemToTaskChunk` | acceptance | T-AS-0006 | `task_chunk::rename_work_item_to_task_chunk` | `intent_properties::rename_work_item_to_task_chunk_preserves_kind_and_fields`; `golden_005_task_chunk_rename_preserves_fields` | ✅ |
| `UnknownGuardIsInvalid` / `EvaluateGuard` | invariants | T-AS-0005 | `guard::{guard_recognized,guard_outcome_for,check_guard}` | `intent_properties::unknown_guard_is_invalid_and_total`; `golden_006_upstream_port_requires_checker` | ✅ |
| contracts: upstream guard port | contracts | — | `guard::UpstreamApprovalChecker` | `intent_properties::upstream_approved_requires_injected_checker`; `golden_006_upstream_port_requires_checker` | ✅ |
| contracts: extractor totality | contracts | — | `extractor::h3_titles` line scanner | `intent_properties::extract_is_total_on_noisy_input` | ✅ |
| contracts: context layer ordering | contracts | — | `context::ContextLayer` + `assemble_layers` | `intent_properties::assemble_layers_is_independent_of_registration_order` | ✅ |

状态：✅ 已覆盖 / ⚠️ 部分 / ❌ 缺失

## File Manifest 对账

| ADR/RFC 路径 | 预期责任 | 磁盘存在 | 备注 |
|---|---|---|---|
| `crates/artifact-system/src/document.rs` | Document artifact model + file round-trip | ✅ | T-AS-0001/0002 |
| `crates/artifact-system/src/guard.rs` | pure document guards + `UpstreamApprovalChecker` port | ✅ | ADR-004 contract 1 |
| `crates/artifact-system/src/context.rs` | ContextLayer trait + deterministic assembly | ✅ | ADR-004 contract 3 |
| `crates/artifact-system/src/extractor.rs` | total structured extraction | ✅ | ADR-004 contract 2 |
| `crates/artifact-system/src/task_chunk.rs` | `work_item` -> `task_chunk_entity` rename | ✅ | T-AS-0006 |
| `crates/storage/src/document_row.rs` | storage-facing DocumentRow mirror | ✅ | ADR-004; shared by skill-runtime |

## cargo test

```
cargo test -p artifact-system
test result: ok. 15 passed (intent_properties); 0 failed

bash docs/baseline/2026-06-09/artifact-system/run-all.sh
All artifact-system golden baselines passed.（6/6）
```

## 待办

| 项 | 类型 | 跟进 |
|---|---|---|
| CLI 字节对账 | cli-ux slice | legacy `doc`/`prompt`/`extract` 命令 vs new binary |
| Full legacy YAML wire parity | divergence | new Document 采用最小 deterministic frontmatter；ADR-006 登记 |
| SQLite IndexDb 全量替换 | storage/cli-ux | 当前只下沉 `DocumentRow`，持久化 wiring 延后 |

## 检查清单

- [x] 每个 acceptance block 在表中有行
- [x] 每个 ✅ 行有具体 `文件::符号` 与 test 名
- [x] File Manifest 与 ADR Consequences 一致
- [x] cargo test 已实跑并记录 exit code
- [x] 待办项未冒充 ✅
