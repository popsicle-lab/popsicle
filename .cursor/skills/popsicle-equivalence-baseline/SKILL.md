---
name: popsicle-equivalence-baseline
description: 为当前迁移 slice 建立 legacy vs new golden 对账：脚本、baseline 快照、 migration/traceability.md 行、divergence 登记。CONTRIBUTING 切流门禁： ≥5 条 golden diff 为空，或 divergence 有 Accepted ADR。slice-delivery 第二棒。 (from module: intent-coder)
---

> This skill is provided by the **intent-coder** module.

Perform the "equivalence-baseline" step in the Popsicle pipeline.

## Workflow

- **Initial state**: `inventory`
- **Final state(s)**: `completed`
- **Transitions**:
  - `inventory` → `running` via `inventoried` (guard: `checklist_complete:Golden Inventory`)
  - `running` → `reporting` via `run-complete`
  - `reporting` → `review` via `submit` (guard: `has_sections:Summary,Golden 清单,运行结果,Traceability,Divergence,门禁;checklist_complete:检查清单`)
  - `review` → `completed` via `approve`
  - `review` → `running` via `revise`

## Inputs (upstream dependencies)

- `implementation-coverage` from skill `shadow-implementer` (required)
- `fact-extraction-report` from skill `fact-extractor` (optional)

## Prerequisites

An active pipeline run MUST exist before executing this skill. If `popsicle pipeline status` shows no active run, you MUST first create an Issue (`popsicle issue create`) then start it (`popsicle issue start <key>`). NEVER execute this skill outside of a pipeline run.

## Commands

```bash
# Verify an active pipeline run exists and this skill is the current step
popsicle pipeline next --format json

# Get enriched prompt with historical references and project context
popsicle prompt equivalence-baseline --run <run-id> --related --format json

# Create the document
popsicle doc create equivalence-baseline --title "<title>" --run <run-id>

# View the created document
popsicle doc show <doc-id>

# ⚠ STOP — Do NOT auto-complete stages
# After creating all documents for a stage, STOP and show the user:
#   1. What documents were created
#   2. The stage completion command below
# Let the user review and decide when to complete the stage.
popsicle pipeline stage complete <stage-name>

```

## Writing Guide

# equivalence-baseline 使用指南

为单个 slice 建立 **legacy vs new 等价性基线**，是 Strangler Fig 切流前的硬门禁。

```
shadow-implementer（代码 + coverage）
    → equivalence-baseline（本 skill）
    → cutover-author
```

## 切流门禁（与 CONTRIBUTING §4 对齐）

满足其一即可进入 cutover：

1. **≥5 条 golden** 同输入 legacy/new **diff 为空**
2. 或：未通过的项均在 **Divergence** 表中有 **Accepted ADR** 登记

## 产物布局

```
docs/baseline/<YYYY-MM-DD>/<slice>/
  README.md
  golden-001-*.sh          # 或 rust integration test
  fixtures/
```

## traceability

本 skill 起草 `migration/traceability.md` 行；**cutover ADR Accepted 后**
由 cutover-author 正式写入并标 `in-shadow` → `cutover-done`。

## 常见 divergence

| 现象 | 处理 |
|---|---|
| intent 要求 `body' == body`，legacy `trim_start` | 登记 divergence + 切流 ADR 说明「新语义为准」|
| extractor 简化实现 vs legacy regex | golden 比「kind + title 集合」而非字节 |
| 尚无 cli-ux | golden 直接调 lib API，CLI 对账留给 slice-3 |

## 红线

- 不臆造 pass——必须实跑脚本
- 不把 divergence 静默吞掉
- 不修改 `.intent`（修 spec 走 intent-spec-writer + PDR）
