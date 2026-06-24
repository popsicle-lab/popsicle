---
id: fc54f504-8fc5-497a-8770-93f42ddd9aa9
doc_type: equivalence-report
title: artifact-system equivalence report
status: final
skill_name: equivalence-baseline
pipeline_run_id: 249be474-d00c-4a2e-a8fc-c64b8aca08d9
spec_id: cf6e7062-b75f-4925-b633-82a04e775a76
version: 1
parent_doc_id: null
tags: []
metadata: null
created_at: 2026-06-09T09:58:21.681367Z
updated_at: 2026-06-09T09:58:21.681367Z
---

---
artifact: equivalence-report
slug: artifact-system-equivalence-report
generated_by: equivalence-baseline
slice: artifact-system
last_updated: 2026-06-09
golden_total: 6
golden_pass: 6
golden_fail: 0
divergence_count: 2
equivalence_gate_pass: true
baseline_dir: docs/baseline/2026-06-09/artifact-system/
query_anchors:
  - "legacy 和 new 行为一致吗？"
  - "golden 对账过了几条？"
---

# 等价性基线报告 — artifact-system-equivalence-report

> 由 `equivalence-baseline` 产出。legacy pin 见 `LEGACY_PIN.md`。

## Summary

| 指标 | 值 |
|---|---|
| Slice | artifact-system |
| Legacy pin | `c76d729db91c59009f0fa8f7c6f1e499eb0c7eb1`（见 LEGACY_PIN.md）|
| Golden 总数 | 6 |
| ✅ pass（lib-level）| 6 |
| ❌ fail | 0 |
| ⚠️ divergence（已 ADR 登记）| 2（scope 外/intent-driven，见下表）|
| **equivalence_gate_pass** | true |

门禁：`golden_pass >= 5` → pass（6/6 lib-level golden）。

## Golden Inventory

- [x] G-001 document roundtrip baseline recorded
- [x] G-002 guard checks baseline recorded
- [x] G-003 context assembly baseline recorded
- [x] G-004 extractor baseline recorded
- [x] G-005 task_chunk rename baseline recorded
- [x] G-006 upstream port baseline recorded

## Golden 清单

| ID | 描述 | 脚本 | Legacy | New | 结果 | diff 摘要 |
|---|---|---|---|---|---|---|
| G-001 | document file-content roundtrip | `golden-001-document-roundtrip.sh` | `model/document.rs` semantics | `document::Document` | PASS | — |
| G-002 | guard has_sections + checklist | `golden-002-guard-checks.sh` | `engine/guard.rs` | `guard::check_guard` | PASS | — |
| G-003 | context assembly ordering | `golden-003-context-assembly.sh` | `engine/context.rs` / `context_layer.rs` | `context::assemble_layers` | PASS | — |
| G-004 | extractor kind + totality | `golden-004-extractors.sh` | `engine/extractor.rs` | `extractor::*` | PASS | — |
| G-005 | work_item -> task_chunk rename | `golden-005-task-chunk-rename.sh` | `model/work_item.rs` | `task_chunk::*` | PASS | — |
| G-006 | upstream approval port | `golden-006-upstream-port.sh` | `engine/guard.rs` registry/run lookup | `guard::UpstreamApprovalChecker` | PASS | — |

## 运行结果

```
bash docs/baseline/2026-06-09/artifact-system/run-all.sh
→ exit 0 — All artifact-system golden baselines passed.
```

## Traceability（拟写入 migration/traceability.md）

| Legacy 路径 | 新位置 | 责任 Spec | 切流 ADR | 等价性 baseline | 状态 |
|---|---|---|---|---|---|
| `crates/popsicle-core/src/model/document.rs` | `crates/artifact-system/src/document.rs` | slice-2-artifact-system | ADR-006 | G-001 | in-shadow |
| `crates/popsicle-core/src/engine/guard.rs` | `crates/artifact-system/src/guard.rs` | slice-2-artifact-system | ADR-004 + ADR-006 | G-002/G-006 | in-shadow |
| `crates/popsicle-core/src/engine/context.rs` / `engine/context_layer.rs` | `crates/artifact-system/src/context.rs` | slice-2-artifact-system | ADR-004 + ADR-006 | G-003 | in-shadow |
| `crates/popsicle-core/src/engine/extractor.rs` | `crates/artifact-system/src/extractor.rs` | slice-2-artifact-system | ADR-004 + ADR-006 | G-004 | in-shadow |
| `crates/popsicle-core/src/model/work_item.rs` | `crates/artifact-system/src/task_chunk.rs` | slice-2-artifact-system | ADR-006 | G-005 | in-shadow |
| `crates/popsicle-core/src/storage/index.rs`（documents row shape）| `crates/storage/src/document_row.rs` | slice-2-artifact-system | ADR-004 + ADR-006 | — | in-shadow |

## Divergence

| ID | 行为 | Legacy | New | 原因 | ADR |
|---|---|---|---|---|---|
| D-001 | Document body parse | `from_file_content` trims leading body whitespace | body-preserving round-trip | formal `DocumentRoundTrips` requires `body' == body` for any body | ADR-006 |
| D-002 | CLI byte parity | legacy `doc` / `prompt` / `extract` commands | lib API only | CLI ownership deferred to `cli-ux` slice | ADR-006 |

## 门禁判定

- [x] ≥5 golden pass，或
- [x] 全部 fail 已列入 Divergence 且 ADR Accepted
- [x] baseline 目录已创建且 README 可复现
- [x] traceability 草稿已写

## 检查清单

- [x] 每条 golden 有实跑证据
- [x] pass/fail 数字与 Summary 一致
- [x] divergence 未隐瞒
- [x] equivalence_gate_pass 可复算
