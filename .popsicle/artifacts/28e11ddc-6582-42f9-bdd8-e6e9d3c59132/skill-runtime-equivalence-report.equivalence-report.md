---
id: 3795c4e4-dbe9-4ffc-8b70-7dc61521c7c7
doc_type: equivalence-report
title: skill-runtime equivalence report
status: final
skill_name: equivalence-baseline
pipeline_run_id: 28e11ddc-6582-42f9-bdd8-e6e9d3c59132
spec_id: df11b6f9-e504-42b7-8d27-700d529c2346
version: 1
parent_doc_id: null
tags: []
metadata: null
created_at: 2026-06-09T09:10:30Z
updated_at: 2026-06-09T09:08:49.158291Z
---

---
artifact: equivalence-report
slug: skill-runtime-equivalence-report
generated_by: equivalence-baseline
slice: skill-runtime
last_updated: 2026-06-09
golden_total: 6
golden_pass: 6
golden_fail: 0
divergence_count: 2
equivalence_gate_pass: true
baseline_dir: docs/baseline/2026-06-09/skill-runtime/
query_anchors:
  - "legacy 和 new 行为一致吗？"
  - "golden 对账过了几条？"
---

# 等价性基线报告 — skill-runtime-equivalence-report

> 由 `equivalence-baseline` 产出。legacy pin 见 `LEGACY_PIN.md`。

## Summary

| 指标 | 值 |
|---|---|
| Slice | skill-runtime |
| Legacy pin | `c76d729db91c59009f0fa8f7c6f1e499eb0c7eb1`（见 LEGACY_PIN.md）|
| Golden 总数 | 6 |
| ✅ pass（lib-level）| 6 |
| ❌ fail | 0 |
| ⚠️ divergence（已 ADR 登记）| 2（scope 外，见下表）|
| **equivalence_gate_pass** | true |

门禁：`golden_pass >= 5` → pass（6/6 lib-level golden）。

## Golden 清单

| ID | 描述 | 脚本 | Legacy | New | 结果 | diff 摘要 |
|---|---|---|---|---|---|---|
| G-001 | load project-init skill | `golden-001-load-project-init.sh` | popsicle-core loader | `loader::load_skill` | PASS | — |
| G-002 | load migration-bootstrap pipeline | `golden-002-migration-bootstrap.sh` | popsicle-core | `loader::load_pipeline` | PASS | — |
| G-003 | load slice-delivery pipeline | `golden-003-slice-delivery.sh` | popsicle-core | `loader::load_pipeline` | PASS | — |
| G-004 | skill registry count | `golden-004-skill-registry.sh` | popsicle-core registry | `SkillRegistry` | PASS | — |
| G-005 | canonical state machine | `golden-005-state-machine.sh` | popsicle-core | `state_machine` | PASS | — |
| G-006 | pipeline session advance | `golden-006-pipeline-session.sh` | popsicle-core | `pipeline_session` | PASS | — |

## 运行结果

```
bash docs/baseline/2026-06-09/skill-runtime/run-all.sh
→ exit 0 — All skill-runtime golden baselines passed.
```

## Traceability（拟写入 migration/traceability.md）

| Legacy 路径 | 新位置 | 责任 Spec | 切流 ADR | 等价性 baseline | 状态 |
|---|---|---|---|---|---|
| `crates/popsicle-core/src/registry/loader.rs` | `crates/skill-runtime/src/loader.rs` | slice-1-skill-runtime | ADR-005 | `docs/baseline/2026-06-09/skill-runtime/` | in-shadow |
| `crates/popsicle-core/src/model/pipeline.rs` | `crates/skill-runtime/src/pipeline_session.rs` | slice-1-skill-runtime | ADR-005 | G-006 | in-shadow |
| `crates/popsicle-core/src/storage/index.rs` | `crates/storage/src/document_row.rs` | slice-1-skill-runtime | ADR-004+005 | — | in-shadow |

## Divergence

| ID | 行为 | Legacy | New | 原因 | ADR |
|---|---|---|---|---|---|
| D-001 | SQLite IndexDb | 完整 SQLite | `MemoryDocumentStore` 占位 | storage slice 未切流 | ADR-005 § Divergence |
| D-002 | CLI 字节对账 | legacy `popsicle` binary | lib API only | cli-ux 未起 | ADR-005 § Divergence |

## 门禁判定

- [x] ≥5 golden pass
- [x] baseline 目录已创建且 README 可复现
- [x] traceability 草稿已写（`migration/traceability.md`）

## 检查清单

- [x] 每条 golden 有实跑证据
- [x] pass/fail 数字与 Summary 一致
- [x] divergence 未隐瞒
- [x] equivalence_gate_pass 可复算
